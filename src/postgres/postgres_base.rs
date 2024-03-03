use tokio;
use tokio_postgres::{NoTls, Error as PGError, row::Row, Client, Statement};
use tokio_postgres::types::ToSql;
use chrono::NaiveDate;
use crate::postgres::app_config::AppConfig;
use crate::postgres::conditions::{ComparisonOperator, Conditions, IsJoin, LogicalOperator};
use crate::postgres::join_tables::JoinTables;
use crate::postgres::sqls::{InsertRecords, QueryColumns, SqlType, UpdateSets};
use crate::postgres::validators::{parameter_validator, validate_alphanumeric_name};

pub(crate) struct PostgresBase {
    username: String,
    password: String,
    hostname: String,
    port: u32,
    dbname: String,
    table_name: String,
    schema_name: String,
    client: Option<Client>
}

enum Param {
    Text(String),
    Int(i32),
    Float(f32),
    Date(NaiveDate),
}

impl PostgresBase {
    pub(crate) fn new(table_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let valid_table_name;
        if !validate_alphanumeric_name(table_name, "") {
            return Err(format!("{} is invalid name. Please confirm the rule of the 'table_name'", table_name).into());
        }
        else {
            valid_table_name = table_name;
        }

        let config = match AppConfig::new() {
            Ok(config) => config,
            Err(e) => return Err(e.into()),
        };
        let schema_name: String;
        let table_name_w_schema = match std::env::var("WORKTIME_DB_SCHEMA") {
            Ok(schema) => {

                if !validate_alphanumeric_name(&schema, "_") {
                    eprintln!("{} is invalid schema name. The schema is ignored so if you need to add schema please use 'set_schema' method.", schema);
                    schema_name = "".to_string();
                    valid_table_name.to_string()
                } else {
                    schema_name = schema.clone();
                    format!("{}.{}", schema, valid_table_name)
                }
            },
            Err(_) => {
                schema_name = "".to_string();
                valid_table_name.to_string()
            }
        };

        Ok(Self {
            username: config.db_username,
            password: config.db_password,
            hostname: config.db_hostname,
            port: config.db_port,
            dbname: config.db_name,
            table_name: table_name_w_schema,
            schema_name,
            client: None
        })
    }

    pub(crate) async fn connect(&mut self) -> Result<(), PGError> {
        let (client, connection) = tokio_postgres::Config::new()
            .user(self.username.as_str())
            .password(self.password.as_str())
            .host(self.hostname.as_str())
            .port(self.port as u16)
            .dbname(self.dbname.as_str())
            .connect(NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        self.client = Some(client);
        Ok(())
    }

    pub(crate) async fn query_raw(&self, query_columns: QueryColumns) -> Result<(), Box<dyn std::error::Error>> {
        self.query_inner_join_conditions(query_columns, JoinTables::new(), Conditions::new(IsJoin::False)).await
    }

    pub(crate) async fn query_condition_raw(&self, query_column: QueryColumns, conditions: Conditions) -> Result<(), Box<dyn std::error::Error>> {
        self.query_inner_join_conditions(query_column, JoinTables::new(), conditions).await
    }

    pub(crate) async fn query_inner_join_conditions(&self, query_columns: QueryColumns, join_tables: JoinTables, conditions: Conditions) -> Result<(), Box<dyn std::error::Error>> {
        let query_statement: String = SqlType::Select(query_columns).sql_build(self.table_name.as_str());
        let mut statement_vec: Vec<String> = vec![query_statement];

        if !join_tables.is_tables_empty() {
            let join_statement = join_tables.generate_statement_text(self.table_name.as_str())?;
            statement_vec.push(join_statement);
        }
        if !conditions.is_empty() {
            let condition_statement = conditions.generate_statement_text(0)?;
            statement_vec.push(condition_statement);
        }

        let statement = statement_vec.join(" ");
        println!("{}", statement);

        Ok(())
    }

    pub(crate) async fn insert(&self, insert_records: InsertRecords) -> Result<(), Box<dyn std::error::Error>> {
        let statement = SqlType::Insert(insert_records).sql_build(self.table_name.as_str());
        println!("{}", statement);
        Ok(())
    }

    pub(crate) async fn update(&self, update_set: UpdateSets, allow_all_update: bool) -> Result<(), Box<dyn std::error::Error>> {
        if allow_all_update {
            self.update_condition(update_set, Conditions::new(IsJoin::False)).await
        }
        else {
            Err("'update' method will update all records in the specified table so please consider to use 'update_condition' instead of this.".into())
        }
    }

    pub(crate) async fn update_condition(&self, update_set: UpdateSets, conditions: Conditions) -> Result<(), Box<dyn std::error::Error>> {
        let statement_base = SqlType::Update(update_set.clone()).sql_build(self.table_name.as_str());
        let mut statement_vec = vec![statement_base];
        let set_num = update_set.get_num_values();

        if !conditions.is_empty() {
            let statement_condition = conditions.generate_statement_text(set_num)?;
            statement_vec.push(statement_condition);
        }

        println!("{}", statement_vec.join(" "));
        Ok(())
    }

    pub(crate) async fn delete(&self, conditions: Conditions) -> Result<(), Box<dyn std::error::Error>> {
        if conditions.is_empty() {
            return Err("'delete' method unsupports deleting records without any condition.".into())
        }

        let statement_base = SqlType::Delete.sql_build(self.table_name.as_str());
        let mut  statement_vec = vec![statement_base];
        statement_vec.push(conditions.generate_statement_text(0)?);

        let statement = statement_vec.join(" ");
        println!("{}", statement);

        Ok(())
    }

    pub(crate) async fn query_row_sql(&self, statement: &str, params: &[&str]) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
        if !validate_alphanumeric_name(statement, "$_,.=* ") {
            return Err("SQL statement is allowed only alphabets and number and allowed symbols but got invalid chars. Please check your input.".into());
        }
        let state_placeholders = statement.matches("$").count();
        if state_placeholders != params.len() {
            return Err("The number of 'statement' placeholders should be match with params number.".into());
        }


        let box_params = Self::params_generator(params);
        let params_ref = Self::params_ref_generator(&box_params);

        let client = match self.client.as_ref() {
            Some(client) => client,
            None => return Err("Client does not exist. Please connect the PostgreSQL first via connect method.".into()),
        };
        let statement = client.prepare(statement).await?;

        match client.query(&statement, &params_ref).await {
            Ok(res) => Ok(res),
            Err(e) => return Err(format!("query failed due to {}.", e).into())
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn set_dbname(&mut self, dbname: &str) -> Self {
        if !validate_alphanumeric_name(dbname, "_") {
            eprintln!("Unexpected dbname inputted so the change is rejected.");
            return self.self_generator();
        }
        self.dbname = dbname.to_string();
        self.self_generator()
    }

    #[allow(dead_code)]
    pub(crate) fn set_schema(&mut self, schema_name: &str) -> Self {
        if !validate_alphanumeric_name(schema_name, "_") {
            eprintln!("Unexpected dbname inputted so the change is rejected.");
            return self.self_generator();
        }

        let table_name: String;
        if self.table_name.contains(".") {
            let origin_table_param = self.table_name.split(".").collect::<Vec<&str>>();
            table_name = origin_table_param[1].to_string();
        }
        else {
            table_name = self.table_name.clone();
        }

        if schema_name.is_empty() {
            self.table_name = table_name;
        }
        else {
            self.table_name = format!("{}.{}", schema_name, table_name);
        }
        self.schema_name = schema_name.to_string();
        self.self_generator()
    }

    #[allow(dead_code)]
    pub(crate) fn set_port(&mut self, port: u32) -> Self {
        self.port = port;
        self.self_generator()
    }

    #[allow(dead_code)]
    pub(crate) fn get_config(&self) -> String {
        let mut schema_name: Option<&str> = None;

        if self.table_name.contains(".") {
            let schema_table: Vec<&str> = self.table_name.split(".").collect();
            schema_name = Some(schema_table[0]);
        }

        if let Some(schema) = schema_name {
            format!("postgresql://{}:{}@{}:{}/{}?options=--search_path={}", self.username, self.password, self.hostname, self.port, self.dbname, schema)
        } else {
            format!("postgresql://{}:{}@{}:{}/{}", self.username, self.password, self.hostname, self.port, self.dbname)
        }

    }

    async fn execute(&self, statement: &String, params: &[Box<dyn ToSql + Sync>]) -> Result<u64, Box<dyn std::error::Error>> {
        let client = match self.client.as_ref() {
            Some(client) => client,
            None => return Err("Client does not exist. Please connect the PostgreSQL first via connect method.".into()),
        };
        let statement: Statement = client.prepare(statement).await?;
        let params_ref: Vec<&(dyn ToSql + Sync)> = Self::params_ref_generator(&params);

        match client.execute(&statement, &params_ref).await {
            Ok(res) => Ok(res),
            Err(e) => Err(format!("SQL executor failed due to {}", e).into()),
        }
    }

    fn self_generator(&self) -> Self {
        Self {
            username: self.username.clone(),
            password: self.password.clone(),
            hostname: self.hostname.clone(),
            port: self.port,
            dbname: self.dbname.clone(),
            table_name: self.table_name.clone(),
            schema_name: self.schema_name.clone(),
            client: None
        }
    }

    fn params_generator(row_params: &[&str]) -> Vec<Box<dyn ToSql + Sync>> {
        let mut params: Vec<Param> = Vec::new();
        for row_param in row_params {
            let str_param = row_param.to_string();
            if let Ok(i) = str_param.parse::<i32>() {
                params.push(Param::Int(i));
            }
            else if let Ok(f) = str_param.parse::<f32>() {
                params.push(Param::Float(f));
            }
            else if let Ok(d) = NaiveDate::parse_from_str(str_param.as_str(), "%Y-%m-%d") {
                params.push(Param::Date(d));
            }
            else {
                params.push(Param::Text(str_param));
            }
        }

        let mut box_param:Vec<Box<dyn ToSql + Sync>> = Vec::new();
        for param in params {
            match param {
                Param::Float(f) => box_param.push(Box::new(f) as Box<dyn ToSql + Sync>),
                Param::Int(i) => box_param.push(Box::new(i) as Box<dyn ToSql + Sync>),
                Param::Text(t) => box_param.push(Box::new(t) as Box<dyn ToSql + Sync>),
                Param::Date(d) => box_param.push(Box::new(d) as Box<dyn ToSql + Sync>),
            };
        }
        box_param
    }

    fn params_ref_generator<'a>(box_params: &'a[Box<dyn ToSql + Sync>]) -> Vec<&'a(dyn ToSql + Sync)> {
        box_params.iter().map(AsRef::as_ref).collect()
    }
}
