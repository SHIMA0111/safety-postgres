use crate::legacy::errors::*;
use crate::legacy::validators::validate_string;

/// Represents the different types of SQL statements.
#[derive(Clone)]
pub(super) enum SqlType <'a> {
    Select(&'a QueryColumns),
    Update(&'a UpdateSets),
    Insert(&'a InsertRecords),
    Delete,
}

/// Trait for building SQL statements.
///
/// This trait defines a method for building SQL statements based on a given table name.
trait SqlBuilder {
    fn build_sql(&self, table_name: &str) -> String;
}

/// Represents a collection of query columns.
///
/// # Example
///
/// ```rust
/// use safety_postgres::legacy::sql_base::QueryColumns;
///
/// let mut query_columns = QueryColumns::new(false);
/// query_columns.add_column("schema_name", "table_name", "column_name").unwrap();
///
/// let query_text = query_columns.get_query_text();
///
/// assert_eq!(query_text, "SELECT schema_name.table_name.column_name FROM main_table_name");
/// ```
#[derive(Clone)]
pub struct QueryColumns {
    all_columns: bool,
    columns: Vec<QueryColumn>,
}

/// Represents a single query column.
#[derive(Clone)]
struct QueryColumn {
    schema_name: String,
    table_name: String,
    column: String,
}

impl QueryColumns {
    /// Creates a new instance of `QueryColumns` struct.
    ///
    /// # Arguments
    ///
    /// * `all_columns` - A boolean value indicating whether all columns should be selected.
    ///
    pub fn new(all_columns: bool) -> Self {
        Self {
            all_columns,
            columns: Vec::new(),
        }
    }

    /// Adds a query selected column to the query.
    ///
    /// # Arguments
    ///
    /// * `schema_name` - The name of the schema (input "" if there is no schema name or in the same table).
    /// * `table_name` - The name of the table (input "" if there is no table name or in the same table).
    /// * `column` - The name of the column.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to `Self` (the query builder) on success, or a `QueryColumnError` on failure.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    /// * The `all_columns` flag is set to true, indicating that all columns will be queried,
    /// so setting a specific column is not allowed.
    /// * The `schema_name`, `table_name`, or `column` is an invalid string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use safety_postgres::legacy::sql_base::QueryColumns;
    ///
    /// let mut query_columns = QueryColumns::new(false);
    /// query_columns.add_column("", "", "id").unwrap().add_column("", "", "username").unwrap();
    /// ```
    pub fn add_column(&mut self, schema_name: &str, table_name: &str, column: &str) -> Result<&mut Self, QueryColumnError> {
        if self.all_columns {
            return Err(QueryColumnError::InputInconsistentError("'all_columns' flag is true so all columns will queried so you can't set column. Please check your input.".to_string()));
        }

        validate_string(schema_name, "schema_name", &QueryColumnErrorGenerator)?;
        validate_string(table_name, "table_name", &QueryColumnErrorGenerator)?;
        validate_string(column, "column_name", &QueryColumnErrorGenerator)?;

        let query_column = QueryColumn {
            schema_name: schema_name.to_string(),
            table_name: table_name.to_string(),
            column: column.to_string(),
        };

        self.columns.push(query_column);
        Ok(self)
    }

    /// Retrieves the query text for the current instance.
    ///
    /// # Returns
    ///
    /// A `String` representing the query text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use safety_postgres::legacy::sql_base::QueryColumns;
    ///
    /// let obj = QueryColumns::new(true);
    /// let query_text = obj.get_query_text();
    /// println!("Query Text: {}", query_text);
    /// ```
    pub fn get_query_text(&self) -> String {
        self.build_sql("main_table_name")
    }
}

impl SqlBuilder for QueryColumns {
    /// Builds an SQL query based on the given parameters.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to query.
    ///
    /// # Returns
    ///
    /// A string representing the SQL query.
    fn build_sql(&self, table_name: &str) -> String {
        let mut sql_vec: Vec<String> = Vec::new();
        sql_vec.push("SELECT".to_string());
        if self.all_columns {
            sql_vec.push("*".to_string());
        }
        else {
            let mut columns: Vec<String> = Vec::new();
            for query_column in &self.columns {
                let mut column_condition: Vec<String> = Vec::new();
                if !query_column.schema_name.is_empty() {
                    column_condition.push(query_column.schema_name.clone());
                }
                if !query_column.table_name.is_empty() {
                    column_condition.push(query_column.table_name.clone());
                }
                column_condition.push(query_column.column.clone());
                columns.push(column_condition.join("."));
            }
            sql_vec.push(columns.join(", "));
        }
        sql_vec.push(format!("FROM {}", table_name));
        sql_vec.join(" ")
    }
}

/// Represents a collection of update sets.
///
/// Update sets are used to define the values to be updated in a database table.
///
/// # Example
///
/// ```rust
/// use safety_postgres::legacy::sql_base::UpdateSets;
///
/// let mut update_sets = UpdateSets::new();
///
/// update_sets.add_set("column1", "value1").expect("adding update set failed");
/// update_sets.add_set("column2", "value2").expect("adding update set failed");
///
/// let update_set_text = update_sets.get_update_text();
///
/// assert_eq!(update_set_text, "UPDATE main_table_name SET column1 = value1, column2 = value2");
/// ```
///
#[derive(Clone)]
pub struct UpdateSets {
    update_sets: Vec<UpdateSet>
}

/// Represents a single column-value pair used in an update statement.
///
/// This struct is used to specify the column and its corresponding value for an update operation.
#[derive(Clone)]
struct UpdateSet {
    column: String,
    value: String,
}

impl UpdateSets {
    /// Creates a new instance of the `UpdateSets` struct.
    pub fn new() -> Self {
        Self {
            update_sets: Vec::new(),
        }
    }

    /// Adds a set of column-value pair to the update sets of the struct.
    ///
    /// # Arguments
    ///
    /// * `column` - The name of the column to be updated.
    /// * `value` - The new value for the column.
    ///
    /// # Errors
    ///
    /// Returns an `UpdateSetError` if the `column` is not a valid string.
    ///
    /// # Returns
    ///
    /// A mutable reference to `Self (UpdateSets)` on success.
    ///
    /// # Example
    ///
    /// ```rust
    /// use safety_postgres::legacy::sql_base::UpdateSets;
    ///
    /// let mut update_sets = UpdateSets::new();
    /// update_sets.add_set("name", "John").expect("adding update set failed");
    /// ```
    pub fn add_set(&mut self, column: &str, value: &str) -> Result<&mut Self, UpdateSetError> {
        validate_string(column, "column", &UpdateSetErrorGenerator)?;

        let update_set = UpdateSet {
            column: column.to_string(),
            value: value.to_string(),
        };
        self.update_sets.push(update_set);

        Ok(self)
    }

    /// Retrieves all the values from the update sets as flatten vector.
    ///
    /// # Returns
    ///
    /// A vector of strings containing the values.
    pub(super) fn get_flat_values(&self) -> Vec<String> {
        let mut flat_values = Vec::new();
        for update_set in &self.update_sets {
            flat_values.push(update_set.value.clone());
        }
        flat_values
    }

    /// Returns the update text for the set parameters.
    pub fn get_update_text(&self) -> String {
        let mut update_text = self.build_sql("main_table_name");
        let values = self.get_flat_values();
        for (index, value) in values.iter().enumerate() {
            update_text = update_text.replace(&format!("${}", index + 1), value);
        }

        update_text
    }

    /// Returns the number of values in the update sets.
    ///
    /// # Returns
    ///
    /// The number of values in the update sets.
    pub fn get_num_values(&self) -> usize {
        self.update_sets.len()
    }
}

impl SqlBuilder for UpdateSets {
    /// Builds an SQL UPDATE statement with the provided table name and set values.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to update.
    ///
    /// # Returns
    ///
    /// Returns the generated SQL statement as a string.
    fn build_sql(&self, table_name: &str) -> String {
        let mut sql_vec: Vec<String> = Vec::new();

        sql_vec.push("UPDATE".to_string());
        sql_vec.push(table_name.to_string());

        let mut set_vec: Vec<String> = Vec::new();
        for (index, update_set) in self.update_sets.iter().enumerate() {
            set_vec.push(format!("{} = ${}", update_set.column, index + 1));
        }
        sql_vec.push(format!("SET {}", set_vec.join(", ")));

        sql_vec.join(" ")
    }
}

/// Represents a collection of insert records.
///
/// # Fields
///
/// - `keys`: A vector of strings representing the columns for the insert records.
/// - `insert_records`: A vector of `InsertRecord` objects.
///
/// # Example
///
/// ```rust
/// use safety_postgres::legacy::sql_base::InsertRecords;
///
/// let mut insert_records = InsertRecords::new(&["str_column1", "int_column2", "float_column3"]);
///
/// let record1 = vec!["value1", "2", "3.1"];
/// let record2 = vec!["value3", "10", "0.7"];
/// insert_records.add_record(&record1).expect("adding insert record failed");
/// insert_records.add_record(&record2).expect("adding insert record failed");
///
/// let insert_query = insert_records.get_insert_text();
///
/// assert_eq!(
///     insert_query,
///     "INSERT INTO main_table_name (str_column1, int_column2, float_column3) VALUES (value1, 2, 3.1), (value3, 10, 0.7)"
/// );
/// ```
#[derive(Clone)]
pub struct InsertRecords {
    keys: Vec<String>,
    insert_records: Vec<InsertRecord>,
}

/// Represents the values of one record to be inserted into a table.
#[derive(Clone)]
struct InsertRecord {
    values: Vec<String>,
}

impl InsertRecords {
    /// Creates a new instance of the `Table` struct.
    ///
    /// # Arguments
    ///
    /// * `columns` - A slice of strings representing the column names to insert.
    pub fn new(columns: &[&str]) -> Self {
        let keys = columns.iter().map(|column| column.to_string()).collect::<Vec<String>>();

        Self {
            keys,
            insert_records: Vec::new()
        }
    }

    /// Adds a record to insert the database.
    ///
    /// # Arguments
    ///
    /// * `record` - A slice of strings representing the values to be inserted.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the `Self` type. Returns an error of type `InsertValueError`
    /// if there is an error during the insertion process.
    ///
    /// # Errors
    ///
    /// Returns an `InsertValueError` if some error occurred during process.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use safety_postgres::legacy::sql_base::InsertRecords;
    ///
    /// let mut insert_records = InsertRecords::new(&["first_name", "last_name", "age"]);
    ///
    /// let record = vec!["John", "Doe", "25"];
    /// insert_records.add_record(&record).unwrap();
    /// ```
    pub fn add_record(&mut self, record: &[&str]) -> Result<&mut Self, InsertValueError> {
        if self.insert_records.is_empty() {
            self.keys.iter().map(|key| validate_string(key.as_str(), "columns", &InsertValueErrorGenerator)).collect::<Result<_, InsertValueError>>()?;
        }
        if record.len() != self.keys.len() {
            return Err(InsertValueError::InputInconsistentError("'values' should match with the 'columns' number. Please input data.".to_string()));
        }

        let insert_record = InsertRecord {
            values: record.iter().map(|value| value.to_string()).collect(),
        };

        self.insert_records.push(insert_record);

        Ok(self)
    }

    /// Retrieves the insert text for the SQL statement.
    ///
    /// # Returns
    ///
    /// The insert text for the SQL statement.
    ///
    /// # Example
    ///
    /// ```
    /// use safety_postgres::legacy::sql_base::InsertRecords;
    ///
    /// let columns = vec!["column1", "column2"];
    /// let mut insert_records = InsertRecords::new(&columns);
    ///
    /// let record = vec!["value1", "value2"];
    /// insert_records.add_record(&record).unwrap();
    ///
    /// let insert_text = insert_records.get_insert_text();
    /// println!("Insert Text: {}", insert_text);
    /// ```
    pub fn get_insert_text(&self) -> String {
        let mut insert_text = self.build_sql("main_table_name");
        let values = self.get_flat_values();

        for (index, value) in values.iter().enumerate() {
            insert_text = insert_text.replace(&format!("${}", index + 1), value);
        }

        insert_text
    }

    /// Returns a vector of all the values from the insert records in a flattened vector.
    ///
    /// # Returns
    ///
    /// - `Vec<String>` - A vector containing all the values from the insert records.
    pub(super) fn get_flat_values(&self) -> Vec<String> {
        let mut flat_values = Vec::new();
        for record in &self.insert_records {
            flat_values.extend(record.values.clone());
        }
        flat_values
    }
}

impl SqlBuilder for InsertRecords {
    /// Builds an SQL statement for inserting records into a specified table.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to insert the records into.
    ///
    /// # Returns
    ///
    /// A string containing the SQL statement for inserting the records.
    fn build_sql(&self, table_name: &str) -> String {
        let mut sql_vec: Vec<String> = Vec::new();

        sql_vec.extend(vec!["INSERT INTO".to_string(), table_name.to_string()]);
        let number_elements = self.keys.len() * self.insert_records.len();
        let mut values_placeholder_vec: Vec<String> = Vec::new();
        for placeholder_index in 1..=number_elements {
            match placeholder_index % self.keys.len() {
                0 => values_placeholder_vec.push(format!("${})", placeholder_index)),
                1 => values_placeholder_vec.push(format!("(${}", placeholder_index)),
                _ => values_placeholder_vec.push(format!("${}", placeholder_index))
            }
        }
        sql_vec.push(format!("({}) VALUES {}", self.keys.join(", "), values_placeholder_vec.join(", ")));
        sql_vec.join(" ")
    }

}

impl SqlType<'_> {
    /// Function to build an SQL query based on the provided SqlType enum.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to perform the query on.
    ///
    /// # Returns
    ///
    /// A string representing the built SQL query.
    pub(super) fn sql_build(&self, table_name: &str) -> String {
        match self {
            SqlType::Select(query_columns) => query_columns.build_sql(table_name),
            SqlType::Insert(insert_values) => insert_values.build_sql(table_name),
            SqlType::Update(update_sets) => update_sets.build_sql(table_name),
            SqlType::Delete => format!("DELETE FROM {}", table_name),
        }
    }
}
