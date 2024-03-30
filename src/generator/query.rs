use std::collections::HashSet;
use crate::generator::base::{ChainMethod, ConditionOperator, Generator};
use crate::utils::errors::GeneratorError;
use crate::utils::helpers::Variable;

struct QueryGenerator<'a> {
    main_schema: Option<&'a str>,
    main_table: &'a str,
    main_query_columns: QueryColumns<'a>,
    join_tables: Vec<JoinTable<'a>>,
    conditions: Vec<Condition<'a>>,
    include_tables: HashSet<String>,
}

impl<'a> QueryGenerator<'a> {
    pub fn new(
        main_table: &'a str,
        query_columns: QueryColumns<'a>,
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
    fn get_statement(&self) -> String {
        todo!()
    }
    fn get_params(&self) -> Vec<String> {
        todo!()
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

struct JoinColumn<'a> {
    main_joined_column: &'a str,
    joined_column: &'a str,
    operator: ConditionOperator,
    bind_method: ChainMethod
}

pub enum QueryColumns<'a> {
    AllColumns,
    SpecifyColumns(&'a[&'a str])
}

pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
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
