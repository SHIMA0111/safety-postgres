use crate::generator::base::{Aggregation, ConditionOperator};
use crate::utils::helpers::Variable;

pub enum GroupBy<'a> {
    OnMainTable {
        column_name: &'a str,
    },
    SameSchemaTable {
        table_name: &'a str,
        column_name: &'a str,
    },
    AnotherSchemaTable {
        schema_name: &'a str,
        table_name: &'a str,
        column_name: &'a str,
    }
}

impl GroupBy<'_> {
    pub(in crate::generator) fn get_table_name(&self) -> String {
        match self {
            GroupBy::OnMainTable { .. } => "main".to_string(),
            GroupBy::SameSchemaTable { table_name, .. } => table_name.to_string(),
            GroupBy::AnotherSchemaTable {
                schema_name,
                table_name, .. } => format!("{}.{}", schema_name, table_name),
        }
    }
}

pub struct GroupCondition<'a> {
    aggregation: Aggregation<'a>,
    ref_value: Variable,
    condition_operator: ConditionOperator,
}

impl <'a> GroupCondition<'a> {
    pub fn new(aggregation: Aggregation<'a>,
               condition_operator: ConditionOperator,
               ref_value: Variable) -> GroupCondition<'a> {
        Self {
            aggregation,
            ref_value,
            condition_operator,
        }
    }
}
