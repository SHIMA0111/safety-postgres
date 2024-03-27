use crate::legacies::errors::{JoinTableError, JoinTableErrorGenerator};
use crate::legacies::validators::{validate_alphanumeric_name, validate_string};

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
/// use safety_postgres::legacies::join_tables::JoinTables;
///
/// let mut join_tables = JoinTables::new();
///
/// join_tables.add_join_table(
///     "",
///     "joined_table",
///     &vec!["joined_table_c1"],
///     &vec!["main_table_c1"]).expect("add joined table failed");
///
/// let join_text = join_tables.get_joined_text();
/// let expected_text =
///     "INNER JOIN joined_table ON main_table_name.main_table_c1 = joined_table.joined_table_c1";
///
/// assert_eq!(join_text, expected_text.to_string());
/// ```
#[derive(Clone)]
pub struct JoinTables {
    tables: Vec<JoinTable>,
}

impl JoinTables {
    /// Create a new instance of JoinTables.
    pub fn new() -> Self {
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
    /// use safety_postgres::legacies::join_tables::JoinTables;
    ///
    /// let mut join_tables = JoinTables::new();
    ///
    /// join_tables.add_join_table("public", "users", &["id"], &["user_id"]).expect("adding join table failed");
    /// let joined_text = join_tables.get_joined_text();
    ///
    /// assert_eq!(joined_text, "INNER JOIN public.users ON main_table_name.user_id = public.users.id");
    /// ```
    pub fn add_join_table(&mut self, schema: &str, table_name: &str, join_columns: &[&str], destination_columns: &[&str]) -> Result<&mut Self, JoinTableError> {
        validate_string(table_name, "table_name", &JoinTableErrorGenerator)?;
        validate_string(schema, "schema", &JoinTableErrorGenerator)?;
        Self::validate_column_collection_pare(join_columns, destination_columns)?;

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
    /// use safety_postgres::legacies::join_tables::JoinTables;
    ///
    /// let mut obj = JoinTables::new();
    /// obj.add_join_table("", "category", &["id"], &["cid"])
    ///     .expect("adding join table failed");
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
    pub fn is_tables_empty(&self) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the successful addition of a `JoinTable`.
    #[test]
    fn test_add_join_table() {
        let mut join_tables = JoinTables::new();
        join_tables.add_join_table("", "users", &["id"], &["user_id"]).unwrap();

        assert_eq!(join_tables.tables.len(), 1);
        assert_eq!(join_tables.tables[0].table_name, "users");
        assert_eq!(join_tables.tables[0].join_columns, vec!["id".to_string()]);
        assert_eq!(join_tables.tables[0].destination_columns, vec!["user_id".to_string()]);
    }

    /// Ensures that a correct SQL INNER JOIN statement is generated.
    #[test]
    fn test_generate_statement_text() {
        let mut join_tables = JoinTables::new();
        join_tables.add_join_table("", "users", &["id"], &["user_id"]).unwrap();
        join_tables.add_join_table("schema", "teams", &["id"], &["team_id"]).unwrap();

        let stmt = join_tables.generate_statement_text("main");
        assert!(stmt.contains("INNER JOIN users ON main.user_id = users.id INNER JOIN schema.teams ON main.team_id = schema.teams.id"));
    }

    /// Checks whether the tables collection is empty.
    #[test]
    fn test_is_tables_empty() {
        let join_tables = JoinTables::new();

        assert!(join_tables.is_tables_empty());
    }

    /// Validates the proper generation of a SQL query from a `JoinTable`.
    #[test]
    fn test_join_table_generate_statement_text() {
        let join_table = JoinTable {
            schema: "".to_string(),
            table_name: "users".to_string(),
            join_columns: vec!["id".to_string()],
            destination_columns: vec!["user_id".to_string()],
        };

        let stmt = join_table.generate_statement_text("main".to_string());
        assert!(stmt.contains("INNER JOIN users ON main.user_id = users.id"))
    }

    /// Tests that the tables vector is initially empty on new `JoinTables` creation.
    #[test]
    fn test_join_tables_empty_constructor() {
        let join_tables = JoinTables::new();
        assert_eq!(join_tables.tables.len(), 0);
    }

    /// Validates the error on `add_join_table` with an invalid schema name is used.
    #[test]
    fn test_invalid_schema_name() {
        let mut join_tables = JoinTables::new();
        let Err(e) = join_tables.add_join_table("schema!", "table", &["id"], &["table_id"]) else { panic!() };
        assert_eq!(e, JoinTableError::InputInvalidError(format!("'{}' has invalid characters. '{}' allows alphabets, numbers and under bar only.", "schema!", "schema")));
    }

    /// Checks error handling when invalid characters are used in a table name.
    #[test]
    fn test_invalid_table_name() {
        let mut join_tables = JoinTables::new();
        let Err(e) = join_tables.add_join_table("", "tabl+e", &["id"], &["table_id"]) else { panic!() };
        assert_eq!(e, JoinTableError::InputInvalidError("'tabl+e' has invalid characters. 'table_name' allows alphabets, numbers and under bar only.".to_string()));
    }

    /// Confirms error when either 'join_columns' or 'destination_columns' contains invalid characters.
    #[test]
    fn test_invalid_char_contains_columns() {
        let ok_columns = vec!["id", "team", "data"];
        let ng_columns = vec!["id", "te;am", "date"];

        let mut join_tables = JoinTables::new();
        let Err(e) = join_tables.add_join_table("", "table", &ng_columns, &ok_columns) else { panic!() };

        assert_eq!(e, JoinTableError::InputInvalidError("'join_columns' includes invalid name. Please check your input.".to_string()));

        let Err(e) = join_tables.add_join_table("", "table", &ok_columns, &ng_columns) else { panic!() };
        assert_eq!(e, JoinTableError::InputInvalidError("'destination_columns' includes invalid name. Please check your input.".to_string()))
    }

    /// Ensures error when 'join_columns' and 'destination_columns' collections' number of elements don't match.
    #[test]
    fn test_inconsistent_number_columns() {
        let mut join_tables = JoinTables::new();
        let Err(e) = join_tables.add_join_table("", "table", &["id"], &["user_name", "id"]) else { panic!() };

        assert_eq!(e, JoinTableError::InputInconsistentError("'join_columns' and 'destination_columns' will be join key in SQL so these should have match number of elements.".to_string()))
    }
}
