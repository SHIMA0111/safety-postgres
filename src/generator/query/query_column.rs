use crate::generator::base::{Aggregation, Parameters};
use crate::utils::errors::GeneratorError;
use crate::{Column, Table};

pub enum QueryColumns<'a> {
    AllColumns(&'a Table<'a>),
    SpecifyColumns(Vec<QueryColumn<'a>>)
}

impl <'a> QueryColumns<'a> {
    pub fn create_all_columns(table: &'a Table<'a>) -> QueryColumns<'a> {
        QueryColumns::AllColumns(table)
    }

    pub fn create_specify_columns() -> QueryColumns<'a> {
        QueryColumns::SpecifyColumns(Vec::<QueryColumn>::new())
    }

    pub fn add_as_is_column(&mut self, column: &'a Column<'a>) -> Result<(), GeneratorError> {
        self.validate_self()?;
        if let QueryColumns::SpecifyColumns(vec) = self {
            vec.push(QueryColumn::AsIs(column));
        }
        Ok(())
    }

    pub fn add_aggregation_column(&mut self, aggregation_column: &'a Aggregation<'a>) -> Result<(), GeneratorError> {
        self.validate_self()?;
        if let QueryColumns::SpecifyColumns(vec) = self {
            vec.push(QueryColumn::Aggregation(aggregation_column));
        }
        Ok(())
    }

    fn validate_self(&self) -> Result<(), GeneratorError> {
        if let QueryColumns::AllColumns(_) = self {
            return Err(
                GeneratorError::InconsistentConfigError(
                    "This QueryColumns specify all columns so you can't add column.".to_string()))
        }
        Ok(())
    }

    pub(crate) fn get_query_columns_statement(&self) -> String {
        match self {
            QueryColumns::AllColumns(table) => format!("{}.*", table.get_table_name()),
            QueryColumns::SpecifyColumns(columns) => {
                let mut query_columns_vec = Vec::new();

                for column in columns {
                    query_columns_vec.push(column.get_statement());
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
    fn get_statement(&self) -> String {
        match self {
            Self::AsIs(column) => format!("{}", column),
            Self::Aggregation(column) => format!("{}", column),
        }
    }

    fn get_params(&self) -> Parameters {
        Parameters::new()
    }
}