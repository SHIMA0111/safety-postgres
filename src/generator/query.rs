use std::collections::HashSet;
use crate::generator::base::{Aggregation, Generator, SortRule};
use crate::generator::base::condition::Condition;
use crate::generator::base::join_table::JoinTable;
use crate::generator::query::from::FromPhrase;
use crate::generator::query::group_by::{GroupBy, GroupCondition};
use crate::utils::errors::GeneratorError;

mod group_by;
mod from;

pub struct QueryGenerator<'a> {
    base_table: FromPhrase<'a>,
    main_query_columns: QueryColumns<'a>,
    join_tables: Vec<JoinTable<'a>>,
    conditions: Vec<Condition<'a>>,
    groupings: Vec<GroupBy<'a>>,
    group_conditions: Vec<GroupCondition<'a>>,
    sort_rules: Vec<SortRule<'a>>,
    include_tables: HashSet<String>,
}

impl<'a> QueryGenerator<'a> {
    pub fn new(
        base_table: FromPhrase<'a>,
        query_columns: QueryColumns<'a>) -> QueryGenerator<'a> {

        Self {
            base_table,
            main_query_columns: query_columns,
            join_tables: Vec::new(),
            conditions: Vec::new(),
            groupings: Vec::new(),
            group_conditions: Vec::new(),
            sort_rules: Vec::new(),
            include_tables: HashSet::from_iter(vec!["main".to_string()]),
        }
    }

    pub fn add_join_table(&mut self, join_table: JoinTable<'a>) {
        let table = join_table.get_table_name();
        self.include_tables.insert(table);
        self.join_tables.push(join_table);
    }

    pub fn add_condition(&mut self, condition: Condition<'a>) -> Result<(), GeneratorError> {
        let table_name = condition.get_table_name();

        match self.table_validation(table_name.as_str()) {
            Ok(_) => self.conditions.push(condition),
            Err(e) => return Err(e)
        }
        Ok(())
    }

    pub fn add_grouping(&mut self, group_by: GroupBy<'a>) -> Result<(), GeneratorError> {
        let table_name = group_by.get_table_name();

        match self.table_validation(table_name.as_str()) {
            Ok(_) => self.groupings.push(group_by),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    pub fn add_aggregation_condition(&mut self, aggregation_condition: GroupCondition<'a>) {
        self.group_conditions.push(aggregation_condition);
    }

    pub fn add_sort_rule(&mut self, sort_rule: SortRule<'a>) -> Result<(), GeneratorError> {
        let table_name = sort_rule.get_table_name();

        match self.table_validation(table_name.as_str()) {
            Ok(_) => self.sort_rules.push(sort_rule),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn table_validation(&self, table_name: &str) -> Result<(), GeneratorError> {
        if !self.include_tables.contains(table_name) {
            return Err(
                GeneratorError::InvalidTableNameError(
                    format!("'{}' doesn't exist in main table and joined tables. \
                    Please set the table as JoinTable first.", table_name)))
        }
        Ok(())
    }
}

impl Generator for QueryGenerator<'_> {
    fn get_statement(&self) -> String {
        todo!()
    }
    fn get_params(&self) -> Vec<String> {
        todo!()
    }
}

pub enum QueryColumns<'a> {
    AllColumns,
    SpecifyColumns(&'a[QueryColumn<'a>])
}

pub enum QueryColumn<'a> {
    AsIs(&'a str),
    Aggregation(Aggregation<'a>),
}
