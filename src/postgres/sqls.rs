use crate::postgres::errors::{InsertValueError, InsertValueErrorGenerator, QueryColumnError, QueryColumnErrorGenerator, StatementError, UpdateSetError, UpdateSetErrorGenerator};
use crate::postgres::validators::validate_string;

#[derive(Clone)]
enum SqlType {
    Select(QueryColumns),
    Update(UpdateSets),
    Insert(InsertValues),
    Delete,
}

#[derive(Clone)]
pub struct QueryColumns {
    all_columns: bool,
    columns: Vec<QueryColumn>,
}

#[derive(Clone)]
struct QueryColumn {
    schema_name: String,
    table_name: String,
    column: Vec<String>,
}

impl QueryColumns {
    fn new(all_columns: bool) -> Self {
        Self {
            all_columns,
            columns: Vec::new(),
        }
    }

    fn add_column(&mut self, schema_name: String, table_name: String, column: &[&str]) -> Result<Self, QueryColumnError> {
        if self.all_columns {
            return Err(QueryColumnError::InputInconsistentError("'all_columns' flag is true so all columns will queried so you can't set column. Please check your input.".to_string()));
        }
        validate_string(schema_name.as_str(), "schema_name", &QueryColumnErrorGenerator)?;
        validate_string(table_name.as_str(), "table_name", &QueryColumnErrorGenerator)?;
        let res: Result<Vec<_>, _> = column.iter()
            .map(|column_name| {
                validate_string(column_name, "column", &QueryColumnErrorGenerator)
            }).collect();

        match res {
            Ok(_) => {},
            Err(e) => return Err(e),
        }

        let column_vec:Vec<String> = column.iter().map(|str| str.to_string()).collect();

        let query_column = QueryColumn {
            schema_name,
            table_name,
            column: column_vec,
        };

        self.columns.push(query_column);
        Ok(self.clone())
    }
}

#[derive(Clone)]
struct UpdateSets {
    update_sets: Vec<UpdateSet>
}

#[derive(Clone)]
struct UpdateSet {
    column: String,
    value: String,
}

impl UpdateSets {
    fn new() -> Self {
        Self {
            update_sets: Vec::new(),
        }
    }

    fn add_set(&mut self, column: &str, value: &str) -> Result<Self, UpdateSetError> {
        validate_string(column, "column", &UpdateSetErrorGenerator)?;
        validate_string(value, "value", &UpdateSetErrorGenerator)?;

        let update_set = UpdateSet {
            column: column.to_string(),
            value: value.to_string(),
        };
        self.update_sets.push(update_set);

        Ok(self.clone())
    }
}

#[derive(Clone)]
struct InsertValue {
    column: String,
    values: Vec<String>,
}

#[derive(Clone)]
struct InsertValues {
    insert_values: Vec<InsertValue>,
}


impl InsertValues {
    fn new() -> Self {
        Self {
            insert_values: Vec::new(),
        }
    }

    fn add_value(&mut self, column: &str, values: &[&str]) -> Result<Self, InsertValueError> {
        validate_string(column, "column", &InsertValueErrorGenerator)?;
        let res: Result<Vec<_>, _> = values.iter()
            .map(|value| {
                validate_string(value, "vales", &InsertValueErrorGenerator)
            }).collect();
        match res {
            Ok(_) => {},
            Err(e) => return Err(e),
        }

        let insert_value = InsertValue {
            column: column.to_string(),
            values: values.iter().map(|value| value.to_string()).collect(),
        };

        self.insert_values.push(insert_value);

        Ok(self.clone())
    }
}
