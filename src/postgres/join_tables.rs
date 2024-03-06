use crate::postgres::errors::{JoinTableError, JoinTableErrorGenerator};
use crate::postgres::validators::{validate_alphanumeric_name, validate_string};

#[derive(Clone)]
struct JoinTable {
    schema: String,
    table_name: String,
    join_columns: Vec<String>,
    destination_columns: Vec<String>,
}

#[derive(Clone)]
pub(super) struct JoinTables {
    pub(super) tables: Vec<JoinTable>,
}

impl JoinTables {
    pub(crate) fn new() -> Self {
        Self {
            tables: Vec::new(),
        }
    }

    pub(super) fn add_join_table(&mut self, table_name: &str, schema: &str, join_columns: &[&str], destination_columns: &[&str]) -> Result<&mut Self, JoinTableError> {
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

    pub(super) fn generate_statement_text(&self, main_table: &str) -> String {
        let mut statement_texts:Vec<String> = Vec::new();

        for table in &self.tables {
            let statement_text = table.generate_statement_text(main_table.to_string());
            statement_texts.push(statement_text);
        }
        statement_texts.join(" ")
    }

    pub(super) fn is_tables_empty(&self) -> bool {
        self.tables.is_empty()
    }

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
