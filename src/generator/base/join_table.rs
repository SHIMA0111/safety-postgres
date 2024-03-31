use crate::generator::base::{ChainMethod, ConditionOperator};
use crate::generator::query::QueryColumns;

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

impl JoinTable<'_> {
    pub(crate) fn get_table_name(&self) -> String {
        match self {
            JoinTable::SameSchemaTable {
                table_name, ..} => format!("{}", table_name),
            JoinTable::AnotherSchemaTable {
                schema_name,
                table_name, ..} => format!("{}.{}", schema_name, table_name),
        }
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

    pub(crate) fn get_join_statement(&self, main_table_name: &str) -> String {
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
                        join_columns_vec.push(column.bind_method.get_string())
                    }
                    let one_join = format!(
                        "{}.{} {} {}.{}",
                        main_table_name, &column.main_joined_column,
                        column.operator.get_symbol(),
                        table_name,
                        &column.joined_column
                    );
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
    main_joined_column: &'a str,
    joined_column: &'a str,
    operator: ConditionOperator,
    bind_method: ChainMethod
}

pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}