use std::borrow::Cow;
use crate::postgres::errors::*;
use crate::postgres::validators::validate_string;

#[derive(Clone)]
enum SqlType {
    Select(QueryColumns),
    Update(UpdateSets),
    Insert(InsertValues),
    Delete,
}

trait SqlBuilder {
    fn build_sql(&self, table_name: &str) -> String;
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

    fn add_column(&mut self, schema_name: Option<Cow<'_, str>>, table_name: Option<Cow<'_, str>>, column: &[&str]) -> Result<&mut Self, QueryColumnError> {
        if self.all_columns {
            return Err(QueryColumnError::InputInconsistentError("'all_columns' flag is true so all columns will queried so you can't set column. Please check your input.".to_string()));
        }

        if let Some(schema) = schema_name.as_deref() {
            validate_string(schema, "schema_name", &QueryColumnErrorGenerator)?;
        }
        if let Some(table) = table_name.as_deref() {
            validate_string(table, "table_name", &QueryColumnErrorGenerator)?;
        }
        let column_vec = column.iter()
            .map(|column_name| {
                validate_string(column_name, "column", &QueryColumnErrorGenerator)
                    .map(|_| column_name.to_string())
            }).collect::<Result<Vec<String>, QueryColumnError>>()?;

        let schema_name = schema_name.map(Cow::into_owned);
        let table_name = table_name.map(Cow::into_owned);

        let query_column = QueryColumn {
            schema_name,
            table_name,
            column: column_vec,
        };

        self.columns.push(query_column);
        Ok(self)
    }
}

impl SqlBuilder for QueryColumns {
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

    fn add_set(&mut self, column: &str, value: &str) -> Result<&mut Self, UpdateSetError> {
        validate_string(column, "column", &UpdateSetErrorGenerator)?;

        let update_set = UpdateSet {
            column: column.to_string(),
            value: value.to_string(),
        };
        self.update_sets.push(update_set);

        Ok(self)
    }
}

impl SqlBuilder for UpdateSets {
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

    fn add_value(&mut self, column: &str, values: &[&str]) -> Result<&mut Self, InsertValueError> {
        validate_string(column, "column", &InsertValueErrorGenerator)?;
        if self.records != (values.len() as u32) {
            return Err(InsertValueError::InputInvalidError("'values' should have match number with the 'records' setting.".to_string()));
        }

        let insert_value = InsertValue {
            column: column.to_string(),
            values: values.iter().map(|value| value.to_string()).collect(),
        };

        self.insert_values.push(insert_value);

        Ok(self)
    }
}

impl SqlBuilder for InsertValues {
    fn build_sql(&self, table_name: &str) -> String {
        let mut sql_vec: Vec<String> = Vec::new();

        sql_vec.extend(vec!["INSERT INTO".to_string(), table_name.to_string()]);
        let mut columns_vec: Vec<String> = Vec::new();
        let number_elements = self.records * (self.insert_values.len() as u32);
        for insert_value in &self.insert_values {
            columns_vec.push(insert_value.column.to_string());
        }
        let mut values_placeholder_vec: Vec<String> = Vec::new();
        for placeholder_index in 1..=number_elements {
            match placeholder_index % self.records {
                0 => values_placeholder_vec.push(format!("${})", placeholder_index)),
                1 => values_placeholder_vec.push(format!("(${}", placeholder_index)),
                _ => values_placeholder_vec.push(format!("${}", placeholder_index))
            }
        }
        sql_vec.push(format!("({}) VALUES ({})", columns_vec.join(", "), values_placeholder_vec.join(", ")));
        sql_vec.join(" ")
    }

}

impl SqlType {
    fn sql_build(&self, table_name: &str) -> String {
        match self {
            SqlType::Select(query_columns) => query_columns.build_sql(table_name),
            SqlType::Insert(insert_values) => insert_values.build_sql(table_name),
            SqlType::Update(update_sets) => update_sets.build_sql(table_name),
            SqlType::Delete => format!("DELETE {}", table_name),
        }
    }
}
