use std::fmt::Display;
use crate::generator::base::{BindMethod, ConditionOperator, GeneratorPlaceholder, GeneratorPlaceholderWrapper, Parameters};
use crate::generator::query::query_column::QueryColumns;
use crate::utils::helpers::Pair;
use crate::{Column, Table};

pub(crate) struct JoinTables<'a> {
    join_tables: Vec<JoinTable<'a>>,
}

impl <'a> JoinTables<'a> {
    pub(crate) fn new() -> JoinTables<'a> {
        Self {
            join_tables: Vec::<JoinTable<'a>>::new()
        }
    }

    pub(crate) fn add_join_table(&mut self, join_table: JoinTable<'a>)  {
        self.join_tables.push(join_table)
    }

    pub(crate) fn get_query_columns(&self) -> String {
        self.join_tables.iter()
            .map(|join_table| join_table.query_columns.get_query_columns_statement())
            .collect::<Vec<String>>().join(", ")
    }
}

impl GeneratorPlaceholderWrapper for JoinTables<'_> {
    fn get_total_statement(&self, start_placeholder: u16) -> String {
        let mut statement = Vec::<String>::new();

        let mut index = start_placeholder;

        for join_table in &self.join_tables {
            statement.push(join_table.get_statement(start_placeholder));
            index += join_table.get_parameters_number();
        }

        statement.join(" ")
    }

    fn get_all_params(&self) -> Parameters {
        let mut params = Parameters::new();
        for join_table in &self.join_tables {
            params += join_table.get_params();
        }
        params
    }

    fn len(&self) -> usize {
        self.join_tables.len()
    }
}

pub struct JoinTable<'a> {
    table: &'a Table<'a>,
    query_columns: &'a QueryColumns<'a>,
    join_columns: Vec<JoinColumn<'a>>,
    join_type: JoinType,
}

impl <'a> JoinTable<'a> {
    pub fn new(table: &'a Table<'a>,
               query_columns: &'a QueryColumns<'a>,
               join_type: JoinType) -> JoinTable<'a> {
        Self {
            table,
            query_columns,
            join_columns: Vec::<JoinColumn<'a>>::new(),
            join_type }
    }

    pub fn add_join_columns(&mut self, src_dist_column: Pair<&'a Column<'a>>,
                            join_condition: ConditionOperator,
                            bind_method: BindMethod) {
        self.join_columns.push(
            JoinColumn::new(src_dist_column, join_condition, bind_method)
        );
    }

    pub(crate) fn get_join_dist_table_names(&self) -> Vec<String> {
        self.join_columns.iter()
            .map(|join_column| join_column.columns.get_first().get_table_name())
            .collect()
    }
}

impl GeneratorPlaceholder for JoinTable<'_> {
    fn get_statement(&self, start_placeholder_numbers: u16) -> String {
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

    fn get_params(&self) -> Parameters {
        self.table.get_parameters()
    }

    fn get_parameters_number(&self) -> u16 {
        self.table.get_parameter_num()
    }

    fn get_table_name(&self) -> String {
        self.table.get_table_name()
    }
}

pub struct JoinColumn<'a> {
    columns: Pair<&'a Column<'a>>,
    operator: ConditionOperator,
    bind_method: BindMethod,
}

impl <'a> JoinColumn<'a> {
    pub fn new(columns: Pair<&'a Column<'a>>,
               operator: ConditionOperator,
               bind_method: BindMethod) -> JoinColumn<'a> {
        Self {
            columns,
            operator,
            bind_method
        }
    }
}


pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}