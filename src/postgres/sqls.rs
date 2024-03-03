use tokio_postgres::types::ToSql;
use crate::postgres::errors::*;
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
    schema_name: Option<String>,
    table_name: Option<String>,
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
        let schema = if schema_name.is_empty() { Some(schema_name.clone()) } else { None };
        let table = if table_name.is_empty() { Some(table_name.clone()) } else { None };

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
            schema_name: schema,
            table_name: table,
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
    records: u32,
}


impl InsertValues {
    fn new(records: u32) -> Self {
        Self {
            insert_values: Vec::new(),
            records
        }
    }

    fn add_value(&mut self, column: &str, values: &[&str]) -> Result<Self, InsertValueError> {
        validate_string(column, "column", &InsertValueErrorGenerator)?;
        if self.records != (values.len() as u32) {
            return Err(InsertValueError::InputInvalidError("'values' should have match number with the 'records' setting.".to_string()));
        }

        let insert_value = InsertValue {
            column: column.to_string(),
            values: values.iter().map(|value| value.to_string()).collect(),
        };

        self.insert_values.push(insert_value);

        Ok(self.clone())
    }
}

impl SqlType {
    fn sql_build(&self, table_name: String) -> String {
        let mut sql_vec: Vec<String> = Vec::new();
        match self {
            SqlType::Select(query_columns) => {
                sql_vec.push("SELECT".to_string());
                if query_columns.all_columns {
                    sql_vec.push("*".to_string());
                }
                else {
                    let mut columns: Vec<String> = Vec::new();
                    for query_column in &query_columns.columns {
                        let mut column_condition: Vec<String> = Vec::new();
                        for column_name in &query_column.column {
                            if let Some(schema) = &query_column.schema_name {
                                column_condition.push(schema.to_string());
                            }
                            if let Some(table) = &query_column.table_name {
                                column_condition.push(table.to_string());
                            }
                            column_condition.push(column_name.to_string());
                            columns.push(column_condition.join("."));
                        }
                    }
                    sql_vec.push(columns.join(", "));
                }
                sql_vec.push(format!("FROM {}", table_name));
                sql_vec.join(" ")
            },
            SqlType::Insert(insert_values) => {
                sql_vec.extend(vec!["INSERT INTO".to_string(), table_name]);
                let mut columns_vec: Vec<String> = Vec::new();
                let number_elements = insert_values.records * (insert_values.insert_values.len() as u32);
                for insert_value in &insert_values.insert_values {
                    columns_vec.push(insert_value.column.to_string());
                }
                let mut values_placeholder_vec: Vec<String> = Vec::new();
                for placeholder_index in 1..=number_elements {
                    match placeholder_index % insert_values.records {
                        0 => values_placeholder_vec.push(format!("${})", placeholder_index)),
                        1 => values_placeholder_vec.push(format!("(${}", placeholder_index)),
                        _ => values_placeholder_vec.push(format!("${}", placeholder_index))
                    }
                }
                sql_vec.push(format!("({}) VALUES ({})", columns_vec.join(", "), values_placeholder_vec.join(", ")));
                sql_vec.join(" ")
            },
            SqlType::Update(update_sets) => {
                sql_vec.push("UPDATE".to_string());
                sql_vec.push(table_name);

                let mut set_vec: Vec<String> = Vec::new();
                for (index, update_set) in update_sets.update_sets.iter().enumerate() {
                    set_vec.push(format!("{} = ${}", update_set.column, index + 1));
                }
                sql_vec.push(format!("SET {}", set_vec.join(", ")));

                sql_vec.join(" ")
            },
            SqlType::Delete => {
                format!("DELETE {}", table_name)
            }
        }
    }
}
