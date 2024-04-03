use std::fmt::Display;
use crate::generator::base::{BindMethod, ConditionOperator};
use crate::generator::query::query_column::QueryColumns;
use crate::utils::helpers::{Column, Pair, Table};

pub(crate) struct JoinTables<'a> {
    join_tables: Vec<&'a JoinTable<'a>>,
}

impl <'a> JoinTables<'a> {
    pub(crate) fn new() -> JoinTables<'a> {
        Self {
            join_tables: Vec::<&'a JoinTable<'a>>::new()
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.join_tables.len()
    }

    pub(crate) fn add_join_table(&mut self, join_table: &'a JoinTable<'a>) {
        self.join_tables.push(join_table);
    }

    pub(crate) fn get_join_statement(&self) -> String {
        let mut statement = Vec::<String>::new();

        for join_table in &self.join_tables {
            statement.push(join_table.get_join_statement());
        }

        statement.join(" ")
    }

    pub(crate) fn get_query_columns(&self) -> String {
        self.join_tables.iter()
            .map(|join_table| join_table.query_columns.get_query_columns_statement())
            .collect::<Vec<String>>().join(", ")
    }
}

pub struct JoinTable<'a> {
    table: &'a Table<'a>,
    query_columns: &'a QueryColumns<'a>,
    join_columns: Vec<JoinColumn<'a>>,
    join_type: JoinType,
}

pub struct JoinColumn<'a> {
    columns: Pair<&'a Column<'a>>,
    operator: ConditionOperator,
    bind_method: BindMethod,
}

impl <'a> JoinColumn<'a> {
    pub fn new(columns: Pair<&'a Column<'a>>, operator: ConditionOperator, bind_method: BindMethod) -> JoinColumn<'a> {
        Self {
            columns,
            operator,
            bind_method
        }
    }
}

impl <'a> JoinTable<'a> {
    pub fn new(
        table: &'a Table<'a>,
        query_columns: &'a QueryColumns<'a>,
        join_type: JoinType
    ) -> JoinTable<'a> {
        Self {
            table,
            query_columns,
            join_columns: Vec::<JoinColumn<'a>>::new(),
            join_type }
    }

    pub fn add_join_columns(
        &mut self, src_dist_column: Pair<&'a Column<'a>>,
        join_condition: ConditionOperator,
        bind_method: BindMethod) {

        self.join_columns.push(
            JoinColumn::new(src_dist_column, join_condition, bind_method)
        );
    }

    pub(crate) fn get_table_name(&self) -> String {
        self.table.get_table_name()
    }

    pub(crate) fn get_join_dist_table_names(&self) -> Vec<String> {
        self.join_columns.iter()
            .map(|join_column| join_column.columns.get_first().get_table_name())
            .collect()
    }

    fn get_join_statement(&self) -> String {
        let join_type_text = match self.join_type {
            JoinType::Inner => "JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Full => "FULL JOIN"
        };

        let mut join_columns_vec = Vec::new();
        for join_column in &self.join_columns {
            if join_columns_vec.len() != 0 {
                join_columns_vec.push(format!("{}", join_column.bind_method))
            }
            let (src_column, dist_column) = join_column.columns.get_values();

            join_columns_vec.push(format!("{} {} {}", src_column, join_column.operator, dist_column));
        }
        let join_columns = join_columns_vec.join(" ");

        format!("{} {} ON {}", join_type_text, self.table, join_columns)
    }
}

pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}