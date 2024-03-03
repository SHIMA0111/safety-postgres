use crate::postgres::errors::*;
use crate::postgres::validators::validate_string;

#[derive(Clone)]
pub(crate) enum SqlType {
    Select(QueryColumns),
    Update(UpdateSets),
    Insert(InsertRecords),
    Delete,
}

trait SqlBuilder {
    fn build_sql(&self, table_name: &str) -> String;
}

#[derive(Clone)]
pub(crate) struct QueryColumns {
    all_columns: bool,
    columns: Vec<QueryColumn>,
}

#[derive(Clone)]
struct QueryColumn {
    schema_name: String,
    table_name: String,
    column: String,
}

impl QueryColumns {
    pub(crate) fn new(all_columns: bool) -> Self {
        Self {
            all_columns,
            columns: Vec::new(),
        }
    }

    pub(crate) fn add_column(&mut self, schema_name: &str, table_name: &str, column: &str) -> Result<&mut Self, QueryColumnError> {
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

#[derive(Clone)]
pub(crate) struct UpdateSets {
    update_sets: Vec<UpdateSet>
}

#[derive(Clone)]
struct UpdateSet {
    column: String,
    value: String,
}

impl UpdateSets {
    pub(crate) fn new() -> Self {
        Self {
            update_sets: Vec::new(),
        }
    }

    pub(crate) fn add_set(&mut self, column: &str, value: &str) -> Result<&mut Self, UpdateSetError> {
        validate_string(column, "column", &UpdateSetErrorGenerator)?;

        let update_set = UpdateSet {
            column: column.to_string(),
            value: value.to_string(),
        };
        self.update_sets.push(update_set);

        Ok(self)
    }

    pub(super) fn get_flat_values(&self) -> Vec<String> {
        let mut flat_values = Vec::new();
        for update_set in &self.update_sets {
            flat_values.push(update_set.value.clone());
        }
        flat_values
    }

    pub(crate) fn get_num_values(&self) -> usize {
        self.update_sets.len()
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
struct InsertRecord {
    values: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct InsertRecords {
    keys: Vec<String>,
    insert_records: Vec<InsertRecord>,
}


impl InsertRecords {
    pub(crate) fn new(columns: &[&str]) -> Self {
        let keys = columns.iter().map(|column| column.to_string()).collect::<Vec<String>>();

        Self {
            keys,
            insert_records: Vec::new()
        }
    }

    pub(crate) fn add_value(&mut self, values: &[&str]) -> Result<&mut Self, InsertValueError> {
        self.keys.iter().map(|key| validate_string(key.as_str(), "columns", &InsertValueErrorGenerator)).collect::<Result<_, InsertValueError>>()?;
        if values.len() != self.keys.len() {
            return Err(InsertValueError::InputInconsistentError("'values' should match with the 'columns' number. Please input data.".to_string()));
        }

        let insert_record = InsertRecord {
            values: values.iter().map(|value| value.to_string()).collect(),
        };

        self.insert_records.push(insert_record);

        Ok(self)
    }

    pub(super) fn get_flat_values(&self) -> Vec<String> {
        let mut flat_values = Vec::new();
        for record in &self.insert_records {
            flat_values.extend(record.values.clone());
        }
        flat_values
    }
}

impl SqlBuilder for InsertRecords {
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

impl SqlType {
    pub(super) fn sql_build(&self, table_name: &str) -> String {
        match self {
            SqlType::Select(query_columns) => query_columns.build_sql(table_name),
            SqlType::Insert(insert_values) => insert_values.build_sql(table_name),
            SqlType::Update(update_sets) => update_sets.build_sql(table_name),
            SqlType::Delete => format!("DELETE {}", table_name),
        }
    }
}
