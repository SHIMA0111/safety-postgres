use std::collections::HashSet;
use crate::generator::base::{Aggregation, Generator, SortRule};
use crate::generator::base::condition::{Condition, Conditions};
use crate::generator::base::join_table::{JoinTable, JoinTables};
use crate::generator::query::from::FromPhrase;
use crate::generator::query::group_by::{Grouping, GroupCondition, Groupings, GroupConditions};
use crate::utils::errors::GeneratorError;
use crate::utils::helpers::Column;

mod group_by;
mod from;

pub struct QueryGenerator<'a> {
    base_table: FromPhrase<'a>,
    main_query_columns: QueryColumns<'a>,
    join_tables: JoinTables<'a>,
    conditions: Conditions<'a>,
    groupings: Groupings<'a>,
    group_conditions: GroupConditions<'a>,
    sort_rules: Vec<SortRule<'a>>,
    include_tables: HashSet<String>,
}

impl<'a> QueryGenerator<'a> {
    pub fn new(
        base_table: FromPhrase<'a>,
        query_columns: QueryColumns<'a>) -> QueryGenerator<'a> {

        let table_name = base_table.get_table_name();

        Self {
            base_table,
            main_query_columns: query_columns,
            join_tables: JoinTables::new(table_name),
            conditions: Conditions::new(),
            groupings: Groupings::new(),
            group_conditions: GroupConditions::new(),
            sort_rules: Vec::new(),
            include_tables: HashSet::from_iter(vec!["main".to_string()]),
        }
    }

    pub fn add_join_table(&mut self, join_table: JoinTable<'a>) -> Result<(), GeneratorError> {
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

    pub fn add_condition(&mut self, condition: Condition<'a>) -> Result<(), GeneratorError> {
        let table_name = condition.get_table_name();

        match self.table_validation(table_name.as_str()) {
            Ok(_) => self.conditions.add_condition(condition)?,
            Err(e) => return Err(e)
        }
        Ok(())
    }

    pub fn add_grouping(&mut self, grouping: Grouping<'a>) -> Result<(), GeneratorError> {
        let table_name = grouping.get_table_name();

        match self.table_validation(table_name.as_str()) {
            Ok(_) => self.groupings.add_grouping(grouping),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    pub fn add_aggregation_condition(&mut self, aggregation_condition: GroupCondition<'a>) -> Result<(), GeneratorError> {
        let table_name = aggregation_condition.get_table_name();

        match self.table_validation(table_name.as_str()) {
            Ok(_) => self.group_conditions.add_group_condition(aggregation_condition),
            Err(e) => return Err(e),
        }
        Ok(())
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
    SpecifyColumns(Vec<QueryColumn<'a>>)
}

pub enum QueryColumn<'a> {
    AsIs(Column<'a>),
    Aggregation(Aggregation<'a>),
}

impl QueryColumns<'_> {
    fn get_query_columns_statement(&self) -> String {
        match self {
            QueryColumns::AllColumns => "*".to_string(),
            QueryColumns::SpecifyColumns(..) => {
                todo!()
            }
        }
    }
}

impl QueryColumn<'_> {
    fn get_column_statement(&self) -> String {
        todo!()
    }
}