use std::collections::HashSet;
use crate::generator::base::{ConditionOperator, Generator};
use crate::utils::errors::GeneratorError;
use crate::utils::helpers::{Pair, Variable};

struct QueryGenerator<'a> {
    main_schema: Option<&'a str>,
    main_table: &'a str,
    main_query_columns: &'a[&'a str],
    join_tables: Vec<JoinTable<'a>>,
    conditions: Vec<Condition<'a>>,
    include_tables: HashSet<String>,
}

pub enum JoinTable<'a> {
    SameSchemaTable{
        table_name: &'a str,
        query_columns: &'a[&'a str],
        join_columns: Vec<Pair<&'a str>>,
    },
    AnotherSchemaTable {
        schema_name: &'a str,
        table_name: &'a str,
        query_columns: &'a[&'a str],
        join_columns: Vec<Pair<&'a str>>,
    }
}

pub enum Condition<'a> {
    OnMainTable {
        column: &'a str,
        value: Variable,
        operator: ConditionOperator,
    },
    SameSchemaTable {
        table_name: &'a str,
        column: &'a str,
        value: Variable,
        operator: ConditionOperator,
    },
    AnotherSchemaTable {
        schema_name: &'a str,
        table_name: &'a str,
        column: &'a str,
        value: Variable,
        operator: ConditionOperator,
    }
}

impl<'a> QueryGenerator<'a> {
    pub fn new(
        main_table: &'a str,
        query_columns: &'a [&'a str],
        main_schema: Option<&'a str>) -> QueryGenerator<'a> {
        Self {
            main_schema,
            main_table,
            main_query_columns: query_columns,
            join_tables: Vec::new(),
            conditions: Vec::new(),
            include_tables: HashSet::from_iter(vec!["main".to_string()]),
        }
    }

    pub fn add_join_table(&mut self, join_table: JoinTable<'a>) {
        let table = join_table.get_table_name();
        self.include_tables.insert(table);
        self.join_tables.push(join_table);
    }

    pub fn add_condition(&mut self, condition: Condition<'a>) -> Result<(), GeneratorError> {
        let table = condition.get_table_name();

        if !self.include_tables.contains(&table) {
            return Err(
                GeneratorError::InvalidTableNameError(
                    format!("'{}' doesn't exist in main table and joined tables. \
                    Please set the table as JoinTable first.", table)))
        }
        self.conditions.push(condition);
        Ok(())
    }
}

impl Generator for QueryGenerator<'_> {
    fn get_statement() -> String {
        todo!()
    }
    fn get_params() -> Vec<String> {
        todo!()
    }
}

impl JoinTable<'_> {
    fn get_table_name(&self) -> String {
        match self {
            JoinTable::SameSchemaTable {
                table_name, ..} => format!("{}", table_name),
            JoinTable::AnotherSchemaTable {
                schema_name,
                table_name, ..} => format!("{}.{}", schema_name, table_name),
        }
    }
}

impl Condition<'_> {
    fn get_table_name(&self) -> String {
        match self {
            Condition::OnMainTable {..} => "main".to_string(),
            Condition::SameSchemaTable { table_name, .. } => format!("{}", table_name),
            Condition::AnotherSchemaTable {
                schema_name,
                table_name, ..} => format!("{}.{}", schema_name, table_name),
        }
    }
}
