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