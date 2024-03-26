use std::fmt::{Debug, Formatter};
use tokio;
use tokio_postgres::{NoTls, Error as PGError, row::Row, Client, Statement};
use tokio_postgres::types::ToSql;
use crate::access::app_config::AppConfig;
use crate::access::conditions::Conditions;
use crate::access::errors::PostgresBaseError;
use crate::access::generate_params::{box_param_generator, params_ref_generator};
use crate::access::join_tables::JoinTables;
use crate::access::json_parser::row_to_json;
use crate::access::sql_base::{InsertRecords, QueryColumns, SqlType, UpdateSets};
use crate::access::validators::validate_alphanumeric_name;

/// Represents a connection config to a PostgreSQL database.
///
/// # Example
/// ```rust
/// use safety_postgres::access::postgres::PostgresBase;
/// use safety_postgres::access::sql_base::QueryColumns;
///
/// async fn postgres_query() {
///     let mut postgres = PostgresBase::new("table_name")
///         .expect("postgres base init failed");
///     postgres.connect().await.expect("connect failed");
///
///     let query_columns = QueryColumns::new(true);
///
///     postgres.query_raw(&query_columns).await.expect("query failed");
/// }
/// ```
pub struct PostgresBase {
    username: String,
    password: String,
    hostname: String,
    port: u32,
    dbname: String,
    table_name: String,
    schema_name: String,
    client: Option<Client>
}

/// Represents the type of execution.
///
/// This enum is used to determine the type of SQL execution to be performed.
/// It can be either `Execute` or `Query`.
enum ExecuteType {
    Execute,
    Query,
}

/// Represents the result of an execution.
///
/// This enum is used to represent the result of an SQL execution operation.
///
/// # Variants
///
/// - `Execute(u64)`: Represents the result of an execution operation that returns a single value of type `u64`.
///
/// - `Query(Vec<Row>)`: Represents the result of a query operation that returns multiple rows of type `Vec<Row>`.
enum ExecuteResult {
    Execute(u64),
    Query(Vec<Row>),
}

impl PostgresBase {
    /// Creates a new instance of `PostgresBase` for interacting with a PostgreSQL database.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to interact with.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the new `PostgresBase` instance if successful,
    /// or a `PostgresBaseError` if an error occurs.
    ///
    /// # Example
    /// ```rust
    /// use safety_postgres::access::postgres::PostgresBase;
    /// # std::env::set_var("DB_USER", "username");
    /// # std::env::set_var("DB_PASSWORD", "password");
    /// # std::env::set_var("DB_HOST", "localhost");
    ///
    /// let mut postgres = PostgresBase::new("table_name").expect("PostgresBase init failed");
    /// ```
    pub fn new(table_name: &str) -> Result<Self, PostgresBaseError> {
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

        let (username, password, hostname, port, dbname) = config.get_values();

        Ok(PostgresBase {
            username: username.to_string(),
            password: password.to_string(),
            hostname: hostname.to_string(),
            port,
            dbname: dbname.to_string(),
            table_name: table_name_w_schema,
            schema_name,
            client: None,
        })
    }

    /// Connects to a PostgreSQL database using the provided configuration.
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
    /// use safety_postgres::access::postgres::PostgresBase;
    ///
    /// async fn postgres_connect() {
    ///     let mut postgres = PostgresBase::new("your_table_name").expect("PostgresBase struct return error");
    ///     let _ = postgres.connect().await.expect("connect failed");
    /// }
    /// ```
    pub async fn connect(&mut self) -> Result<(), PGError> {
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

    /// Executes a raw query on the database and returns the result.
    ///
    /// # Arguments
    ///
    /// * `query_columns` - A `QueryColumns` struct reference specifying the columns to query.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Row>)` - Get the values if the query was successful.
    /// * `Err(PostgresBaseError)` - If an error occurred during the query process.
    ///
    /// # Errors
    ///
    /// Returns a `PostgresBaseError` if there was an error executing the query.
    pub async fn query_raw(&self, query_columns: &QueryColumns) -> Result<Vec<Row>, PostgresBaseError> {
        let empty_join_table = JoinTables::new();
        let empty_condition = Conditions::new();
        self.query_inner_join_conditions(query_columns, &empty_join_table, &empty_condition).await
    }

    /// Queries the database for data based on the provided query column and conditions.
    ///
    /// # Arguments
    ///
    /// * `query_column` - The columns using reference of the `QueryColumns` struct to query.
    /// * `conditions` - The conditions using reference of the `Conditions` to apply to the query.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Row>)` - Get the values if the query was successful.
    /// * `Err(PostgresBaseError)` - If an error occurred during the query process.
    ///
    /// # Errors
    ///
    /// Returns a `PostgresBaseError` if there was an error querying the database.
    pub async fn query_condition_raw(&self, query_column: &QueryColumns, conditions: &Conditions) -> Result<Vec<Row>, PostgresBaseError> {
        let join_tables = JoinTables::new();
        self.query_inner_join_conditions(query_column, &join_tables, conditions).await
    }

    /// Queries the database with inner join and conditions.
    ///
    /// # Arguments
    ///
    /// * `query_columns` - The columns using reference of the `QueryColumns` struct to query.
    /// * `join_tables` - The tables collection using reference of the `JoinTables` to join.
    /// * `conditions` - The conditions using reference of the `Conditions` to apply to the query.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Row>)` - Get the values if the query was successful
    /// * `Err(PostgresBaseError)` - If an error occurred during the query process.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use safety_postgres::access::conditions::Conditions;
    /// use safety_postgres::access::join_tables::JoinTables;
    /// use safety_postgres::access::postgres::PostgresBase;
    /// use safety_postgres::access::sql_base::QueryColumns;
    ///
    /// async fn postgres_query() {
    ///     let mut db = PostgresBase::new("table_name").unwrap();
    ///     db.connect().await.expect("connection failed");
    ///
    ///     let query_column = QueryColumns::new(true);
    ///     let join_tables = JoinTables::new();
    ///     let conditions = Conditions::new();
    ///
    ///     /**
    ///     * Your code....
    ///     */
    ///
    ///     let result = db.query_condition_raw(&query_column, &conditions).await;
    ///     match result {
    ///         Ok(rows) => {
    ///             for row in rows {
    ///                 // Do something with the row
    ///             }
    ///         }
    ///         Err(error) => {
    ///             // Handle the error
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn query_inner_join_conditions(&self, query_columns: &QueryColumns, join_tables: &JoinTables, conditions: &Conditions) -> Result<Vec<Row>, PostgresBaseError> {
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

    pub async fn query_json(&self, query_columns: &QueryColumns) -> Result<String, PostgresBaseError> {
        let empty_join_table = JoinTables::new();
        let empty_condition = Conditions::new();
        self.query_inner_join_conditions_json(query_columns, &empty_join_table, &empty_condition).await
    }

    pub async fn query_condition_json(&self, query_columns: &QueryColumns, conditions: &Conditions) -> Result<String, PostgresBaseError> {
        let empty_join_table = JoinTables::new();
        self.query_inner_join_conditions_json(query_columns, &empty_join_table, conditions).await
    }

    pub async fn query_inner_join_conditions_json(&self, query_columns: &QueryColumns, join_tables: &JoinTables, conditions: &Conditions) -> Result<String, PostgresBaseError> {
        let query_result = self.query_inner_join_conditions(query_columns, join_tables, conditions).await?;
        let json_result = match row_to_json(&query_result) {
            Ok(json) => json,
            Err(e) => return Err(PostgresBaseError::SerializeError(e.to_string())),
        };

        Ok(json_result)
    }

    /// Inserts records into the database table.
    ///
    /// # Arguments
    ///
    /// * `insert_records` - An `InsertRecords` object reference containing the records to be inserted.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the records were inserted successfully.
    /// * `Err(PostgresBaseError)` - If an error occurred during the insertion process.
    ///
    /// # Examples
    ///
    /// ```
    /// use safety_postgres::access::postgres::PostgresBase;
    /// use safety_postgres::access::sql_base::InsertRecords;
    ///
    /// async fn postgres_insert() {
    ///     let mut db = PostgresBase::new("my_table").expect("db struct init failed");
    ///     db.connect().await.expect("connection failed");
    ///     let mut insert_records = InsertRecords::new(&["column1", "column2"]);
    ///     insert_records.add_record(&["value1", "value2"]).expect("add record failed");
    ///
    ///     let result = db.insert(&insert_records).await.expect("insert failed");
    /// }
    /// ```
    pub async fn insert(&self, insert_records: &InsertRecords) -> Result<(), PostgresBaseError> {
        let params_values = insert_records.get_flat_values();
        let insert = SqlType::Insert(insert_records);
        let statement = insert.sql_build(self.table_name.as_str());
        let res = self.execute(&statement, &params_values).await?;
        println!("{} record(s) are inserted.", res);
        Ok(())
    }

    /// Updates records in the specified table based on the given update sets.
    ///
    /// # Arguments
    ///
    /// - `update_set`: An `UpdateSets` object reference which containing the fields to update.
    /// - `allow_all_update`: A boolean flag indicating whether updating all records is allowed.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the update is successful.
    /// - `Err(PostgresBaseError)` if an error occurs during the update.
    pub async fn update(&self, update_set: &UpdateSets, allow_all_update: bool) -> Result<(), PostgresBaseError> {
        if allow_all_update {
            let condition = Conditions::new();
            self.update_condition(update_set, &condition).await
        }
        else {
            Err(PostgresBaseError::UnsafeExecutionError("'update' method will update all records in the specified table so please consider to use 'update_condition' instead of this.".to_string()))
        }
    }

    /// Updates records in the table based on the specified update set and conditions.
    ///
    /// # Arguments
    ///
    /// * `update_set` - The `UpdateSets` reference specifying the columns and values to update.
    /// * `conditions` - The `Conditions` reference specifying the records to update.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update operation is successful.
    /// * `Err(PostgresBaseError)` - If an error occurs during the update operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use safety_postgres::access::conditions::{Conditions, IsInJoinedTable};
    /// use safety_postgres::access::postgres::PostgresBase;
    /// use safety_postgres::access::sql_base::UpdateSets;
    ///
    /// async fn postgres_update() {
    ///     let mut database = PostgresBase::new("my_table").expect("postgres base init failed");
    ///     database.connect().await.expect("connection failed");
    ///
    ///     let mut update_set = UpdateSets::new();
    ///     update_set.add_set("column1", "value1").unwrap();
    ///
    ///     let mut conditions = Conditions::new();
    ///     conditions.add_condition_from_str(
    ///         "column1",
    ///         "value1",
    ///         "eq",
    ///         "",
    ///         IsInJoinedTable::No)
    ///         .expect("adding condition failed");
    ///
    ///     database.update_condition(&update_set, &conditions).await.expect("update failed");
    /// }
    /// ```
    pub async fn update_condition(&self, update_set: &UpdateSets, conditions: &Conditions) -> Result<(), PostgresBaseError> {
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

    /// Delete records from the database table based on given conditions.
    ///
    /// # Arguments
    ///
    /// * `conditions` - The reference of the conditions used to filter the records to be deleted.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Returns this if the deletion is successful
    /// * `PostgresBaseError` - Returns an error of type `PostgresBaseError` when deletion process failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use safety_postgres::access::conditions::ComparisonOperator::Grater;
    /// use safety_postgres::access::conditions::{Conditions, IsInJoinedTable};
    /// use safety_postgres::access::conditions::LogicalOperator::FirstCondition;
    /// use safety_postgres::access::postgres::PostgresBase;
    ///
    /// async fn postgres_delete() {
    ///     let mut database = PostgresBase::new("my_table").expect("db init failed");
    ///     database.connect().await.expect("connecting failed");
    ///
    ///     let mut conditions = Conditions::new();
    ///     conditions.add_condition(
    ///         "column1",
    ///         "value1",
    ///         Grater,
    ///         FirstCondition,
    ///         IsInJoinedTable::No).expect("adding condition failed");
    ///
    ///     database.delete(&conditions).await.expect("delete failed");
    /// }
    /// ```
    pub async fn delete(&self, conditions: &Conditions) -> Result<(), PostgresBaseError> {
        if conditions.is_empty() {
            return Err(PostgresBaseError::UnsafeExecutionError("'delete' method unsupported deleting records without any condition.".to_string()))
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

    /// Sets the name of the database.
    ///
    /// This method validates the given `dbname` parameter to ensure it consists only of alphanumeric characters and underscores.
    /// If the validation fails, an error message is printed to the standard error output and the change is rejected.
    ///
    /// # Arguments
    ///
    /// * `dbname` - The new name of the database.
    ///
    /// # Returns
    ///
    /// The updated `self` object.
    pub fn set_dbname(&mut self, dbname: &str) -> &mut Self {
        if !validate_alphanumeric_name(dbname, "_") {
            eprintln!("Unexpected dbname inputted so the change is rejected.");
            return self;
        }
        self.dbname = dbname.to_string();
        self
    }

    /// Sets the schema for the database table.
    ///
    /// # Arguments
    ///
    /// * `schema_name` - The new name of the schema to set.
    ///
    /// # Returns
    ///
    /// The modified `Self` object.
    pub fn set_schema(&mut self, schema_name: &str) -> &mut Self {
        if !validate_alphanumeric_name(schema_name, "_") {
            eprintln!("Unexpected dbname inputted so the change is rejected.");
            return self;
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
        self
    }

    /// Sets the port for the postgresql.
    ///
    /// # Arguments
    ///
    /// * `port` - The new port setting for the postgresql
    ///
    /// # Returns
    ///
    /// The modified `self` object.
    pub fn set_port(&mut self, port: u32) -> &mut Self {
        self.port = port;
        self
    }

    /// Executes a query statement with the given parameters and returns a vector of rows as the result.
    ///
    /// # Arguments
    ///
    /// - `statement_str`: A reference to a `String` containing the query statement to be executed.
    /// - `params`: A slice of `String` containing the parameters to be used in the query.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Row>)` - Returns vector of `Row` if the query was executed successfully.
    /// * `Err(PostgresBaseError)` - Returns `PostgresBaseError` if an error occurred during this process.
    ///
    /// # Errors
    ///
    /// This function can return a `PostgresBaseError` in the following cases:
    ///
    /// - If an internal execution error occurs, an `UnexpectedError` variant of `PostgresBaseError` will be returned.
    async fn query(&self, statement_str: &String, params: &[String]) -> Result<Vec<Row>, PostgresBaseError> {
        let result = self.execute_core(statement_str, params, ExecuteType::Query).await?;
        match result {
            ExecuteResult::Query(res) => Ok(res),
            _ => return Err(PostgresBaseError::UnexpectedError("Execution internal error occurred, please contact the developer.".to_string())),        }
    }

    /// Executes a database statement with parameters asynchronously.
    ///
    /// # Arguments
    ///
    /// * `statement_str` - The database statement to execute.
    /// * `params` - The parameters to bind to the statement.
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` - Returns the number of rows affected by the statement if successful
    /// * `Err(PostgresBaseError)` - Returns an error if an unexpected error occurred
    ///
    /// # Errors
    ///
    /// Returns an `PostgresBaseError` if an unexpected error occurred while executing the statement.
    async fn execute(&self, statement_str: &String, params: &[String]) -> Result<u64, PostgresBaseError> {
        let result = self.execute_core(statement_str, params, ExecuteType::Execute).await?;
        match result {
            ExecuteResult::Execute(res) => Ok(res),
            _ => return Err(PostgresBaseError::UnexpectedError("Execution internal error occurred, please contact the developer.".to_string())),
        }
    }

    /// Executes a PostgreSQL statement with the given parameters and return the result.
    ///
    /// # Arguments
    ///
    /// * `statement_str` - The statement string to execute.
    /// * `params` - The parameters to bind to the statement.
    /// * `execute_type` - The type of execution (Execute or Query).
    ///
    /// # Returns
    ///
    /// * Ok(ExecuteResult) - Returns result valiant containing the execution result
    /// * Err(PostgresBaseError) - Returns an error if the execution failed
    async fn execute_core(&self, statement_str: &String, params: &[String], execute_type: ExecuteType) -> Result<ExecuteResult, PostgresBaseError> {
        let client = match self.client.as_ref() {
            Some(client) => client,
            None => return Err(PostgresBaseError::ConnectionNotFoundError("Client does not exist. Please connect the PostgreSQL first via connect method.".to_string())),
        };

        let box_params_res = box_param_generator(params);
        let box_params = match box_params_res {
            Ok(box_params) => box_params,
            Err(e) => return Err(PostgresBaseError::SQLExecutionError(e.to_string())),
        };
        let params_ref: Vec<&(dyn ToSql + Sync)> = params_ref_generator(&box_params);

        let statement: Statement = match client.prepare(statement_str).await {
            Ok(statement) => statement,
            Err(e) => return Err(PostgresBaseError::TokioPostgresError(e.to_string())),
        };

        match execute_type {
            ExecuteType::Execute => {
                match client.execute(&statement, &params_ref).await {
                    Ok(res) => Ok(ExecuteResult::Execute(res)),
                    Err(e) => return Err(PostgresBaseError::SQLExecutionError(e.to_string())),
                }
            }
            ExecuteType::Query => {
                match client.query(&statement, &params_ref).await {
                    Ok(res) => Ok(ExecuteResult::Query(res)),
                    Err(e) => return Err(PostgresBaseError::SQLExecutionError(e.to_string())),
                }
            }
        }
    }
}

impl Debug for PostgresBase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut schema_name: Option<&str> = None;

        if self.table_name.contains(".") {
            let schema_table: Vec<&str> = self.table_name.split(".").collect();
            schema_name = Some(schema_table[0]);
        }
        if let Some(schema) = schema_name {
            write!(f, "postgresql://{}:****@{}:{}/{}?options=--search_path={}", self.username, self.hostname, self.port, self.dbname, schema)
        } else {
            write!(f, "postgresql://{}:****@{}:{}/{}", self.username, self.hostname, self.port, self.dbname)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::access::errors::PostgresBaseError;
    use crate::access::postgres::PostgresBase;

    #[test]
    fn test_set_and_get_connect_conf() {
        std::env::set_var("DB_USER", "username");
        std::env::set_var("DB_PASSWORD", "password");
        std::env::set_var("DB_HOST", "localhost");

        let mut postgres = PostgresBase::new("test_table").unwrap();

        let config = format!("{:?}", postgres);

        assert_eq!(config, "postgresql://username:****@localhost:5432/postgres");

        postgres.set_dbname("test");
        let config = format!("{:?}", postgres);

        assert_eq!(config, "postgresql://username:****@localhost:5432/test");

        postgres.set_port(12345);
        let config = format!("{:?}", postgres);

        assert_eq!(config, "postgresql://username:****@localhost:12345/test");

        postgres.set_schema("schema");
        let config = format!("{:?}", postgres);

        assert_eq!(config, "postgresql://username:****@localhost:12345/test?options=--search_path=schema");
    }

    #[test]
    fn test_new_get_invalid_value() {
        std::env::set_var("DB_USER", "username");
        std::env::set_var("DB_PASSWORD", "password");
        std::env::set_var("DB_HOST", "localhost");

        let Err(e) = PostgresBase::new("tab;le") else { panic!() };
        assert_eq!(e, PostgresBaseError::InputInvalidError(format!("{} is invalid name. Please confirm the rule of the 'table_name'", "tab;le")));
    }
}
