use crate::generator::base::Aggregation;
use crate::utils::errors::GeneratorError;
use crate::utils::helpers::{Column, Table};

pub enum QueryColumns<'a> {
    AllColumns(&'a Table<'a>),
    SpecifyColumns(Vec<QueryColumn<'a>>)
}

impl <'a> QueryColumns<'a> {
    pub fn new(all_column: bool, table: Option<&'a Table<'a>>) -> Result<QueryColumns<'a>, GeneratorError> {
        let query_columns = if all_column {
            match table {
                Some(table) => QueryColumns::AllColumns(table),
                None => return Err(GeneratorError::InvalidInputError("When 'all_column' is enabled, you need to input valid Table reference as table.".to_string()))
            }
        } else {
            QueryColumns::SpecifyColumns(Vec::new())
        };

        Ok(query_columns)
    }

    pub fn add_query_column(&mut self, query_column: QueryColumn<'a>) -> Result<(), GeneratorError> {
        match self {
            Self::AllColumns(_) => return Err(
                GeneratorError::InconsistentConfigError(
                    "This QueryColumns specify all columns so you can't add column.".to_string())),
            Self::SpecifyColumns(columns) => columns.push(query_column),
        }
        Ok(())
    }

    pub(crate) fn get_query_columns_statement(&self) -> String {
        match self {
            QueryColumns::AllColumns(table) => format!("{}.*", table.get_table_name()),
            QueryColumns::SpecifyColumns(columns) => {
                let mut query_columns_vec = Vec::new();

                for column in columns {
                    query_columns_vec.push(column.get_column_statement());
                }
                query_columns_vec.join(", ")
            }
        }
    }
}

pub enum QueryColumn<'a> {
    AsIs(&'a Column<'a>),
    Aggregation(&'a Aggregation<'a>),
}

impl QueryColumn<'_> {
    fn get_column_statement(&self) -> String {
        match self {
            Self::AsIs(column) => format!("{}", column),
            Self::Aggregation(column) => format!("{}", column),
        }
    }
}