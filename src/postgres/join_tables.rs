use crate::postgres::errors::{JoinTableError, JoinTableErrorGenerator};
use crate::postgres::validators::{validate_alphanumeric_name, validate_string};

/// Represents a join table in a database.
#[derive(Clone)]
struct JoinTable {
    schema: String,
    table_name: String,
    join_columns: Vec<String>,
    destination_columns: Vec<String>,
}

/// Represents a collection of join tables in a database.
///
/// # Example
/// ```rust
/// let join_tables = JoinTables::new();
///
/// join_tables.add_join_table("schema_name"(there is no schema name, input ""),
///     "table_name", &vec!["column1_from_joined_table", ...],
///     &vec!["column1_in_main_table", ...])
///
/// let join_text = join_tables.get_joined_text();
/// assert_eq!(join_text,
/// "INNER JOIN main_table_name ON
/// main_table_name.column1_in_main_table = schema_name.table_name.column1_from_joined_table
/// AND ...");
/// ```
#[derive(Clone)]
pub(super) struct JoinTables {
    pub(super) tables: Vec<JoinTable>,
}

impl JoinTables {
    /// Create a new instance of JoinTables.
    pub(crate) fn new() -> Self {
        Self {
            tables: Vec::new(),
        }
    }

    /// Adds a join table to the instance.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema name for the new join table (input "" if there is no schema_name).
    /// * `table_name` - The table name for the new join table.
    /// * `join_columns` - The names of the columns in the joined table.
    /// * `destination_columns` - The names of the columns in the main(base) table.
    ///
    /// # Errors
    ///
    /// Returns a `JoinTableError` if there is an error adding the join table.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut join_tables = JoinTables::new();
    ///
    /// join_tables.add_join_table("public", "users", &["id"], &["user_id"])?;
    /// let joined_text = join_tables.get_joined_text()
    ///
    /// assert_eq!(joined_text, "INNER JOIN main_table_name ON main_table_name.user_id = public.users.id");
    /// ```
    pub(super) fn add_join_table(&mut self, schema: &str, table_name: &str, join_columns: &[&str], destination_columns: &[&str]) -> Result<&mut Self, JoinTableError> {
        validate_string(table_name, "table_name", &JoinTableErrorGenerator)?;
        validate_string(schema, "schema", &JoinTableErrorGenerator)?;
        let _ = Self::validate_column_collection_pare(join_columns, destination_columns)?;

        fn convert_vec(input: &[&str]) -> Vec<String> {
            input.iter().map(|str| str.to_string()).collect()
        }

        let join_table = JoinTable {
            schema: schema.to_string(),
            table_name: table_name.to_string(),
            join_columns: convert_vec(join_columns),
            destination_columns: convert_vec(destination_columns),
        };

        self.tables.push(join_table);

        Ok(self)
    }

    /// Generate the statement text for the given main table.
    ///
    /// # Arguments
    ///
    /// * `main_table` - The name of the main(base) table.
    ///
    /// # Returns
    ///
    /// The generated statement text as a `String`.
    ///
    pub(super) fn generate_statement_text(&self, main_table: &str) -> String {
        let mut statement_texts:Vec<String> = Vec::new();

        for table in &self.tables {
            let statement_text = table.generate_statement_text(main_table.to_string());
            statement_texts.push(statement_text);
        }
        statement_texts.join(" ")
    }

    /// Returns the joined text generated from the given join information.
    ///
    /// # Examples
    ///
    /// ```
    /// let obj = JoinTables::new();
    /// obj.add_table("", "category", &["id"], &["cid"]);
    /// let joined_text = obj.get_joined_text();
    /// println!("Joined Text: {}", joined_text);
    /// // This will display:
    /// // "Joined Text: INNER JOIN main_table_name ON main_table_name.cid = category.id"
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a `String` that represents the joined text generated.
    pub fn get_joined_text(&self) -> String {
        self.generate_statement_text("main_table_name")
    }

    /// Checks if the tables collection is empty.
    ///
    /// # Returns
    ///
    /// Returns `true` if the tables collection is empty, `false` otherwise.
    pub(super) fn is_tables_empty(&self) -> bool {
        self.tables.is_empty()
    }

    /// Validates the column collections for joining tables.
    ///
    /// This function takes two slices of strings representing join and
    /// destination columns respectively,
    /// and checks for the validity of the column names according to the following rules:
    ///
    /// - All column names must be alphanumeric or contain underscores.
    /// - There should be a matching number of join and destination columns.
    ///
    /// # Arguments
    ///
    /// * `join_columns` - A slice of strings representing join columns from joined table.
    /// * `destination_columns` - A slice of strings representing destination columns from main table.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` with the following possible outcomes:
    ///
    /// * `Ok(())` - If the column collections pass the validation.
    /// * `Err(JoinTableError)` - If there are any validation errors. The error type provides a detailed message.
    fn validate_column_collection_pare(join_columns: &[&str], destination_columns: &[&str]) -> Result<(), JoinTableError> {
        if !join_columns.iter().all(|column| validate_alphanumeric_name(column, "_")) {
            return Err(JoinTableError::InputInvalidError("'join_columns' includes invalid name. Please check your input.".to_string()));
        }
        if !destination_columns.iter().all(|column| validate_alphanumeric_name(column, "_")) {
            return Err(JoinTableError::InputInvalidError("'destination_columns' includes invalid name. Please check your input.".to_string()));
        }

        if join_columns.len() != destination_columns.len() {
            return Err(JoinTableError::InputInconsistentError("'join_columns' and 'destination_columns' will be join key in SQL so these should have match number of elements.".to_string()));
        }

        Ok(())
    }
}

impl JoinTable {
    /// Generates a SQL statement for joining a main table with a secondary table.
    ///
    /// # Arguments
    ///
    /// * `main_table` - The name of the main table to join.
    ///
    /// # Returns
    ///
    /// The generated inner join SQL statement as a `String`.
    fn generate_statement_text(&self, main_table: String) -> String {
        let table_with_schema = if self.schema.is_empty() {
            self.table_name.clone()
        } else {
            format!("{}.{}", self.schema, self.table_name)
        };
        let mut statement = format!("INNER JOIN {} ON", table_with_schema);
        for (index, (join_column, destination_column)) in self.join_columns.iter().zip(&self.destination_columns).enumerate() {
            statement += format!(" {}.{} = {}.{}", main_table, destination_column, table_with_schema, join_column).as_str();
            if index + 1 < self.join_columns.len() {
                statement += " AND";
            }
        }
        statement
    }
}
