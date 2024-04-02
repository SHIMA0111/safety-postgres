use std::collections::HashSet;
use crate::generator::base::{BindMethod, Generator, SortRule, SortRules};
use crate::generator::base::condition::{Condition, Conditions};
use crate::generator::base::join_table::{JoinTable, JoinTables};
use crate::generator::query::group_by::{GroupCondition, Groupings, GroupConditions};
use crate::generator::query::query_column::QueryColumns;
use crate::utils::errors::GeneratorError;
use crate::utils::helpers::{Column, Table};

pub mod group_by;
pub mod query_column;

pub struct QueryGenerator<'a> {
    base_table: &'a Table<'a>,
    main_query_columns: &'a QueryColumns<'a>,
    join_tables: JoinTables<'a>,
    conditions: Conditions<'a>,
    groupings: Groupings<'a>,
    group_conditions: GroupConditions<'a>,
    sort_rules: SortRules<'a>,
    include_tables: HashSet<String>,
}

impl<'a> QueryGenerator<'a> {
    pub fn new(
        base_table: &'a Table<'a>,
        query_columns: &'a QueryColumns<'a>) -> QueryGenerator<'a> {

        let main_table = base_table.get_table_name();

        Self {
            base_table,
            main_query_columns: query_columns,
            join_tables: JoinTables::new(),
            conditions: Conditions::new(),
            groupings: Groupings::new(),
            group_conditions: GroupConditions::new(),
            sort_rules: SortRules::new(),
            include_tables: HashSet::from_iter(vec![main_table]),
        }
    }

    pub fn add_join_table(&mut self, join_table: &'a JoinTable<'a>) -> Result<(), GeneratorError> {
        let table = join_table.get_table_name();

        let join_dist_tables = join_table.get_join_dist_table_names();

        for join_dist_table in join_dist_tables {
            if let Err(e) = self.table_validation(join_dist_table.as_str()) {
                return Err(e)
            }
        }

        self.include_tables.insert(table);
        self.join_tables.add_join_table(join_table);
        Ok(())
    }

    pub fn add_condition(&mut self, condition: &'a Condition<'a>, bind_method: BindMethod) -> Result<(), GeneratorError> {
        let table_name = condition.get_table_name();

        match self.table_validation(table_name.as_str()) {
            Ok(_) => self.conditions.add_condition(condition, bind_method)?,
            Err(e) => return Err(e)
        }
        Ok(())
    }

    pub fn add_grouping(&mut self, grouping_column: &'a Column<'a>) -> Result<(), GeneratorError> {
        let table_name = grouping_column.get_table_name();

        match self.table_validation(table_name.as_str()) {
            Ok(_) => self.groupings.add_grouping(grouping_column),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    pub fn add_aggregation_condition(&mut self, aggregation_condition: &'a GroupCondition<'a>) -> Result<(), GeneratorError> {
        let table_name = aggregation_condition.get_table_name();

        match self.table_validation(table_name.as_str()) {
            Ok(_) => self.group_conditions.add_group_condition(aggregation_condition),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    pub fn add_sort_rule(&mut self, sort_rule: &'a SortRule<'a>) -> Result<(), GeneratorError> {
        let table_name = sort_rule.get_table_name();

        match self.table_validation(table_name.as_str()) {
            Ok(_) => self.sort_rules.add_sort_rule(sort_rule),
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
        let mut base_vec = vec!["SELECT".to_string()];
        let (query_columns, join_tables) = {
            let mut columns_vec = vec![self.main_query_columns.get_query_columns_statement()];
            let mut join_tables_vec = Vec::<String>::new();
            if self.join_tables.len() != 0 {
                columns_vec.push(self.join_tables.get_query_columns());
                join_tables_vec.push(self.join_tables.get_join_statement());
            }
            (columns_vec.join(" ,"), join_tables_vec.join(" "))
        };
        let from_statement = format!("FROM {}", self.base_table);

        base_vec.push(query_columns);
        base_vec.push(from_statement);

        if self.join_tables.len() != 0 {
            base_vec.push(join_tables);
        }
        if self.conditions.len() != 0 {
            base_vec.push(self.conditions.get_condition_statement());
        }
        if self.groupings.len() != 0 {
            base_vec.push(self.groupings.get_grouping_statement());
        }
        if self.group_conditions.len() != 0 {
            base_vec.push(self.group_conditions.get_grouping_condition_statement());
        }
        if self.sort_rules.len() != 0 {
            base_vec.push(self.sort_rules.get_sort_rule_statement());
        }

        base_vec.join(" ")
    }
    fn get_params(&self) -> Vec<String> {
        todo!()
    }
}