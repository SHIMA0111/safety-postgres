use std::fmt::Display;
use crate::generator::base::{BindMethod, ConditionOperator};
use crate::generator::query::QueryColumns;
use crate::utils::helpers::Column;

pub(crate) struct JoinTables<'a> {
    main_table_name: String,
    join_tables: Vec<JoinTable<'a>>,
}

impl <'a> JoinTables<'a> {
    pub(crate) fn new(main_table_name: String) -> JoinTables<'a> {
        Self {
            main_table_name,
            join_tables: Vec::<JoinTable<'a>>::new()
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.join_tables.len()
    }

    pub(crate) fn add_join_table(&mut self, join_table: JoinTable<'a>) {
        self.join_tables.push(join_table);
    }

    pub(crate) fn get_join_statement(&self) -> String {
        let mut statement = Vec::<String>::new();

        for join_table in &self.join_tables {
            statement.push(join_table.get_join_statement(self.main_table_name.as_str()));
        }

        statement.join(" ")
    }
}

pub enum JoinTable<'a> {
    SameSchemaTable{
        table_name: &'a str,
        query_columns: QueryColumns<'a>,
        join_columns: JoinColumns<'a>,
        join_method: JoinType,
    },
    AnotherSchemaTable {
        schema_name: &'a str,
        table_name: &'a str,
        query_columns: QueryColumns<'a>,
        join_columns: JoinColumns<'a>,
        join_method: JoinType,
    }
}

impl <'a> JoinTable<'a> {
    pub(crate) fn get_table_name(&self) -> String {
        match self {
            JoinTable::SameSchemaTable {
                table_name, ..} => format!("{}", table_name),
            JoinTable::AnotherSchemaTable {
                schema_name,
                table_name, ..} => format!("{}.{}", schema_name, table_name),
        }
    }

    pub(crate) fn get_join_dist_table_names(&self) -> Vec<String> {
        let join_columns =  match self {
            Self::SameSchemaTable {join_columns, ..} => join_columns,
            Self::AnotherSchemaTable {join_columns, ..} => join_columns,
        };
        let join_dist_table = if let JoinColumns::SpecifyColumns(columns) = join_columns {
            columns.iter().map(|column| {
                format!("{}", column.destination_joined_column)
            }).collect::<Vec<String>>()
        }
        else {
            Vec::new()
        };
        join_dist_table
    }

    fn get_join_type(&self) -> &JoinType {
        match self {
            JoinTable::AnotherSchemaTable {join_method, ..} => join_method,
            JoinTable::SameSchemaTable {join_method, ..} => join_method,
        }
    }

    fn get_join_columns(&self) -> &JoinColumns {
        match self {
            JoinTable::AnotherSchemaTable { join_columns, .. } => join_columns,
            JoinTable::SameSchemaTable { join_columns, .. } => join_columns,
        }
    }

    fn get_join_statement(&self, main_table_name: &str) -> String {
        let mut join_statement_vec: Vec<String> = Vec::new();

        let join_method = match self.get_join_type() {
            JoinType::Inner => "JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Full => "FULL JOIN"
        };
        join_statement_vec.push(join_method.to_string());

        let table_name = self.get_table_name();
        join_statement_vec.push(table_name.to_string());

        let join_columns = match self.get_join_columns() {
            JoinColumns::AllColumns => "TRUE".to_string(),
            JoinColumns::SpecifyColumns(columns) => {
                let mut join_columns_vec = Vec::new();
                for column in columns {
                    if join_columns_vec.len() != 0 {
                        join_columns_vec.push(format!("{}", column.bind_method))
                    }
                    let one_join = if let Column::OnMainTable {..} = &column.destination_joined_column {
                        format!(
                            "{}.{} {} {}.{}",
                            main_table_name,
                            &column.destination_joined_column,
                            &column.operator,
                            table_name,
                            &column.joined_column
                        )
                    }
                    else {
                        format!(
                            "{} {} {}.{}",
                            &column.destination_joined_column,
                            &column.operator,
                            table_name,
                            &column.joined_column)
                    };
                    join_columns_vec.push(one_join);
                }
                join_columns_vec.join(" ")
            }
        };
        join_statement_vec.push(join_columns);

        join_statement_vec.join(" ")
    }
}

pub enum JoinColumns<'a> {
    AllColumns,
    SpecifyColumns(Vec<JoinColumn<'a>>)
}

pub struct JoinColumn<'a> {
    joined_column: &'a str,
    destination_joined_column: Column<'a>,
    operator: ConditionOperator,
    bind_method: BindMethod
}

pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}