use crate::generator::query::QueryGenerator;

pub enum FromPhrase<'a> {
    NonSchemaTable {
        table_name: &'a str,
    },
    WithSchemaTable {
        schema_name: &'a str,
        table_name: &'a str,
    },
    SubQueryAsTable {
        sub_query: Box<QueryGenerator<'a>>,
    }
}

impl FromPhrase<'_> {
    pub(crate) fn get_table_name(&self) -> String {
        match self {
            FromPhrase::NonSchemaTable { table_name } => table_name.to_string(),
            FromPhrase::WithSchemaTable {
                schema_name, table_name } => format!("{}.{}", schema_name, table_name),
            FromPhrase::SubQueryAsTable { .. } => "subquery_table".to_string(),
        }
    }
    pub(crate) fn get_from_statement(&self) -> String {
        match self {
            FromPhrase::NonSchemaTable {..} | FromPhrase::WithSchemaTable {..} =>
                format!("FROM {}", self.get_table_name()),
            FromPhrase::SubQueryAsTable { .. } => todo!(),
        }
    }
}
