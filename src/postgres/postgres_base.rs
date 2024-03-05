use tokio;
use tokio_postgres::{NoTls, Error as PGError, row::Row, Client, Statement};
use tokio_postgres::types::ToSql;
use crate::postgres::app_config::AppConfig;
use crate::postgres::conditions::Conditions;
use crate::postgres::errors::PostgresBaseError;
use crate::postgres::generate_params::{box_param_generator, params_ref_generator};
use crate::postgres::join_tables::JoinTables;
use crate::postgres::sql_base::{InsertRecords, QueryColumns, SqlType, UpdateSets};
use crate::postgres::validators::validate_alphanumeric_name;

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

enum ExecuteType {
    Execute,
    Query,
}

enum ExecuteResult {
    Execute(u64),
    Query(Vec<Row>),
}

impl PostgresBase {
    /// Create a new instance of the `PostgresBase` struct.
    ///
    /// # Arguments
    ///
    /// * `table_name` - A string slice that represents the name of the table you want to connect.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the new `PostgresBase` instance, or a boxed `dyn std::error::Error`.
    ///
    /// # Errors
    ///
    /// An error is returned if the table name is invalid. The table name should only contain alphanumeric characters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::error::Error;
    ///
    /// let table = match new("users") {
    ///     Ok(postgres) => postgres,
    ///     Err(e) => panic!("Error creating table: {}", e),
    /// };
    /// ```
    pub(crate) fn new(table_name: &str) -> Result<Self, PostgresBaseError> {
        let valid_table_name;
        if !validate_alphanumeric_name(table_name, "_") {
            return Err(PostgresBaseError::InputInvalidError(format!("{} is invalid name. Please confirm the rule of the 'table_name'", table_name)));
        }
        else {
            valid_table_name = table_name;
        }

        let config = match AppConfig::new() {
            Ok(config) => config,
            Err(e) => return Err(PostgresBaseError::ConfigNotDefinedError(e)),
        };
        let schema_name: String;
        let table_name_w_schema = match std::env::var("DB_SCHEMA") {
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

    /// Connects to a PostgreSQL database using the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the current instance of the struct.
    ///
    /// # Returns
    ///
    /// Returns a result indicating whether the connection was successful or an error occurred.
    /// If the connection is successful, `Ok(())` is returned.
    /// If an error occurs, `Err(PGError)` is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut postgres = PostgresBase::new("your_table_name");
    /// let _ = postgres.connect().await?;
    /// ```
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

    /// Executes a raw database query and returns the resulting rows.
    ///
    /// ## Parameters
    ///
    /// - `query_columns`: The `QueryColumns` instance including columns to query.
    ///
    /// ## Returns
    ///
    /// Returns a `Result` indicating success or failure the query. If successful, it contains
    /// a `Vec` of `Row` objects representing the query results. If an error occurs,
    /// it contains a `Box<dyn std::error::Error>`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// let query_columns = QueryColumns::new(false);
    /// query_columns.add_column("your_schema_name (this can use empty string slice)", "your_table_name (this can use empty string slice)", "your_column_name");
    /// let postgres = PostgresBase::new();
    /// let res = postgres.query_raw(query_columns);
    /// ```
    ///
    /// ## Errors
    ///
    /// This function can return an error if there is a problem executing the raw query.
    /// The error type is `Box<dyn std::error::Error>`.
    pub(crate) async fn query_raw(&self, query_columns: QueryColumns) -> Result<Vec<Row>, PostgresBaseError> {
        self.query_inner_join_conditions(query_columns, JoinTables::new(), Conditions::new()).await
    }

    pub(crate) async fn query_condition_raw(&self, query_column: QueryColumns, conditions: Conditions) -> Result<Vec<Row>, PostgresBaseError> {
        self.query_inner_join_conditions(query_column, JoinTables::new(), conditions).await
    }

    pub(crate) async fn query_inner_join_conditions(&self, query_columns: QueryColumns, join_tables: JoinTables, conditions: Conditions) -> Result<Vec<Row>, PostgresBaseError> {
        let query_statement: String = SqlType::Select(query_columns).sql_build(self.table_name.as_str());
        let mut statement_vec: Vec<String> = vec![query_statement];

        if !join_tables.is_tables_empty() {
            let join_statement = join_tables.generate_statement_text(self.table_name.as_str());
            statement_vec.push(join_statement);
        }

        let params_values = conditions.get_flat_values();
        if !conditions.is_empty() {
            let condition_statement = conditions.generate_statement_text(0);
            statement_vec.push(condition_statement);
        }

        let statement = statement_vec.join(" ");
        let res = self.query(&statement, &params_values).await?;
        Ok(res)
    }

    pub(crate) async fn insert(&self, insert_records: InsertRecords) -> Result<(), PostgresBaseError> {
        let params_values = insert_records.get_flat_values();
        let insert = SqlType::Insert(insert_records);
        let statement = insert.sql_build(self.table_name.as_str());
        let res = self.execute(&statement, &params_values).await?;
        println!("{} record(s) are inserted.", res);
        Ok(())
    }

    pub(crate) async fn update(&self, update_set: UpdateSets, allow_all_update: bool) -> Result<(), PostgresBaseError> {
        if allow_all_update {
            self.update_condition(update_set, Conditions::new()).await
        }
        else {
            Err(PostgresBaseError::UnsafeExecutionError("'update' method will update all records in the specified table so please consider to use 'update_condition' instead of this.".to_string()))
        }
    }

    pub(crate) async fn update_condition(&self, update_set: UpdateSets, conditions: Conditions) -> Result<(), PostgresBaseError> {
        let set_num = update_set.get_num_values();
        let mut params_values = update_set.get_flat_values();
        let statement_base = SqlType::Update(update_set).sql_build(self.table_name.as_str());
        let mut statement_vec = vec![statement_base];

        params_values.extend(conditions.get_flat_values());
        if !conditions.is_empty() {
            let statement_condition = conditions.generate_statement_text(set_num);
            statement_vec.push(statement_condition);
        }
        let statement = statement_vec.join(" ");

        let res = self.execute(&statement, &params_values).await?;
        println!("{} record(s) are updated.", res);
        Ok(())
    }

    pub(crate) async fn delete(&self, conditions: Conditions) -> Result<(), PostgresBaseError> {
        if conditions.is_empty() {
            return Err(PostgresBaseError::UnsafeExecutionError("'delete' method unsupports deleting records without any condition.".to_string()))
        }

        let statement_base = SqlType::Delete.sql_build(self.table_name.as_str());
        let mut  statement_vec = vec![statement_base];
        let params_values = conditions.get_flat_values();
        statement_vec.push(conditions.generate_statement_text(0));

        let statement = statement_vec.join(" ");
        let res = self.execute(&statement, &params_values).await?;
        println!("{} record(s) are deleted.", res);

        Ok(())
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

    async fn query(&self, statement_str: &String, params: &[String]) -> Result<Vec<Row>, PostgresBaseError> {
        let result = self.execute_core(statement_str, params, ExecuteType::Query).await?;
        match result {
            ExecuteResult::Query(res) => Ok(res),
            _ => return Err(PostgresBaseError::UnexpectedError("Execution internal error occurred, please contact the developer.".to_string())),        }
    }

    async fn execute(&self, statement_str: &String, params: &[String]) -> Result<u64, PostgresBaseError> {
        let result = self.execute_core(statement_str, params, ExecuteType::Execute).await?;
        match result {
            ExecuteResult::Execute(res) => Ok(res),
            _ => return Err(PostgresBaseError::UnexpectedError("Execution internal error occurred, please contact the developer.".to_string())),
        }
    }

    async fn execute_core(&self, statement_str: &String, params: &[String], execute_type: ExecuteType) -> Result<ExecuteResult, PostgresBaseError> {
        let client = match self.client.as_ref() {
            Some(client) => client,
            None => return Err(PostgresBaseError::ConnectionNotFoundError("Client does not exist. Please connect the PostgreSQL first via connect method.".to_string())),
        };

        let box_params = box_param_generator(params);
        let params_ref: Vec<&(dyn ToSql + Sync)> = params_ref_generator(&box_params);

        let statement: Statement = match client.prepare(statement_str).await {
            Ok(statement) => statement,
            Err(e) => return Err(PostgresBaseError::TokioPostgresError(format!("Prepare statement generation failed in tokio-postgres like {}", e))),
        };

        match execute_type {
            ExecuteType::Execute => {
                match client.execute(&statement, &params_ref).await {
                    Ok(res) => Ok(ExecuteResult::Execute(res)),
                    Err(e) => return Err(PostgresBaseError::SQLExecutionError(format!("SQL executor failed due to {}", e))),
                }
            }
            ExecuteType::Query => {
                match client.query(&statement, &params_ref).await {
                    Ok(res) => Ok(ExecuteResult::Query(res)),
                    Err(e) => return Err(PostgresBaseError::SQLExecutionError(format!("SQL executor failed due to {}", e))),
                }
            }
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
}
