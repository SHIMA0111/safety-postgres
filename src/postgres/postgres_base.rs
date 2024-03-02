use tokio;
use tokio_postgres::{NoTls, Error as PGError, row::Row, Client, Statement};
use tokio_postgres::types::ToSql;
use chrono::NaiveDate;
use crate::postgres::app_config::AppConfig;
use crate::postgres::validators::{parameter_validator, validate_alphanumeric_name};

pub(super) struct PostgresBase {
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

pub(crate) enum ConditionOperator {
    And,
    Or,
    Parameter,
}

pub(crate) struct JoinTable {
    schema: String,
    table_name: String,
    key_column: Vec<String>,
    destination_column: Vec<String>,
}

pub(crate) struct JoinTables {
    tables: Vec<JoinTable>,
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

    pub(crate) async fn query_raw(&self, query_column: &[&str]) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
        if !query_column.is_empty() && !query_column.iter().all(|str| validate_alphanumeric_name(str, "_")) {
            return Err("Invalid column name inputted. Please check the input again.".into());
        }

        self.query_condition_raw(query_column, &[], &[], ConditionOperator::And).await
    }

    pub(crate) async fn query_condition_raw(&self, query_column: &[&str], condition_key: &[&str], condition_values: &[&str], condition_operator: ConditionOperator) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
        if let Err(e) = parameter_validator(condition_key, condition_values) {
            return Err(e.into());
        }
        let query_param: String;
        if query_column.is_empty() {
            query_param = "*".to_string();
        }
        else {
            query_param = query_column.join(", ");
        }

        let statement: String;
        if condition_key.is_empty() {
            statement = format!("SELECT {} FROM {}", query_param, self.table_name);
        }
        else {
            let condition_key_str = Self::statement_parameter_builder(condition_key, 0, condition_operator);
            statement = format!("SELECT {} FROM {} WHERE {}", query_param, self.table_name, condition_key_str);
        }
        Ok(self.query_row_sql(statement.as_str(), condition_values).await?)
    }

    async fn query_inner_join_conditions(&self, join_tables: JoinTables, condition_keys: &[&str], condition_values: &[&str], condition_operator: ConditionOperator) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
        if join_tables.tables.is_empty() {
            return Err("'join_tables' doesn't have any joined tables if you want to query from only one table, please use 'query_condition_raw' method.".into());
        }
        return unimplemented!()

    }

    pub(crate) async fn insert(&self, keys: &[&str], values: &Vec<Vec<&str>>) -> Result<(), Box<dyn std::error::Error>> {
        if !keys.iter().all(|key| validate_alphanumeric_name(key, "_")) {
            return Err("Invalid key name. Please use only alphabetic characters and underscores.".into());
        }
        for (index, value) in values.iter().enumerate() {
            if keys.len() != value.len() {
                return Err(format!("'keys' and all 'values' elements should have the same length but index {} has {} elements", index, value.len()).into())
            }
        }

        let num_values = keys.len();
        let mut placeholder = "".to_string();
        for index in 1..=num_values {
            placeholder += format!("${}", index).as_str();
            if index == num_values {
                placeholder += "";
            }
            else {
                placeholder += ", ";
            }
        }

        let insert_statement = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table_name,
            keys.join(", "),
            placeholder
        );

        let mut  error_record_index = Vec::new();
        let mut complete_insert_num = 0;
        for (index, value) in values.iter().enumerate() {
            let params = Self::params_generator(value);
            let res = match self.execute(&insert_statement, &params).await {
                Ok(res) => res,
                Err(e) => {
                    error_record_index.push(index);
                    eprintln!("Data index: {} insert failed due to {}", index, e);
                    0
                }
            };
            complete_insert_num += res;
        }
        if complete_insert_num == (values.len() as u64) {
            Ok(())
        } else {
            let failed_indexes = error_record_index.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ");
            Err(format!("Input Index: {} were failed due to insert error.", failed_indexes).into())
        }
    }

    pub(crate) async fn update(&self, keys: &[&str], values: &[&str], allow_all_update: bool) -> Result<(), Box<dyn std::error::Error>> {
        if allow_all_update {
            self.update_condition(keys, values, &[], &[], ConditionOperator::And).await
        }
        else {
            Err("'update' method will update all records in the specified table so please consider to use 'update_condition' instead of this.".into())
        }
    }

    pub(crate) async fn update_condition(&self, keys: &[&str], values: &[&str], condition_keys: &[&str], condition_values: &[&str], condition_operator: ConditionOperator) -> Result<(), Box<dyn std::error::Error>> {
        if let Err(e) = parameter_validator(keys, values) {
            return Err(e.into());
        }
        if let Err(e) = parameter_validator(condition_keys, condition_values) {
            return Err(e.into());
        }

        if keys.is_empty() {
            return Err("'update' related methods require number of keys and values at least 1.".into());
        }

        let update_statement: String = Self::statement_parameter_builder(keys, 0, ConditionOperator::Parameter);

        let statement: String;
        if condition_keys.is_empty() {
            statement = format!("UPDATE {} SET {}", self.table_name, update_statement);
        }
        else {
            let condition_statement: String = Self::statement_parameter_builder(condition_keys, keys.len(), condition_operator);
            statement = format!("UPDATE {} SET {} WHERE {}", self.table_name, update_statement, condition_statement);
        }

        let mut params_raw: Vec<&str> = Vec::new();
        params_raw.extend_from_slice(values);
        params_raw.extend_from_slice(condition_values);

        let params = Self::params_generator(&params_raw);
        match self.execute(&statement, &params).await {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("UPDATE failed due to {}", e).into()),
        }
    }

    pub(crate) async fn delete(&self, condition_keys: &[&str], condition_values: &[&str], condition_operator: ConditionOperator) -> Result<(), Box<dyn std::error::Error>> {
        if let Err(e) = parameter_validator(condition_keys, condition_values) {
            return Err(e.into());
        };
        if condition_keys.is_empty() {
            return Err("'delete' method can't execute without conditions. Please check your input.".into());
        };
        let condition_statement = Self::statement_parameter_builder(condition_keys, 0, condition_operator);
        let statement = format!("DELETE {} WHERE {}", self.table_name, condition_statement);

        let params = Self::params_generator(condition_values);
        match self.execute(&statement, &params).await {
            Ok(_) => Ok(()),
            Err(e) => return Err(format!("DELETE failed due to {}", e).into()),
        }
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

    fn statement_parameter_builder(keys: &[&str], prev_index: usize, condition_operator: ConditionOperator) -> String {
        let mut statement_parameter = "".to_string();
        for (index, key) in keys.iter().enumerate() {
            statement_parameter += format!("{} = ${}", key, index + prev_index + 1).as_str();
            if index + 1 != keys.len() {
                match condition_operator {
                    ConditionOperator::And => statement_parameter += " AND ",
                    ConditionOperator::Or => statement_parameter += " OR ",
                    ConditionOperator::Parameter => statement_parameter += ", ",
                }
            }
        }
        statement_parameter
    }
}
