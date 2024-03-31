use crate::generator::base::ConditionOperator;
use crate::generator::query::QueryGenerator;
use crate::utils::helpers::Variable;

pub enum Condition<'a> {
    OnMainTable {
        column: &'a str,
        ref_value: Variable,
        operator: ConditionOperator,
    },
    SameSchemaTable {
        table_name: &'a str,
        column: &'a str,
        ref_value: Variable,
        operator: ConditionOperator,
    },
    AnotherSchemaTable {
        schema_name: &'a str,
        table_name: &'a str,
        column: &'a str,
        ref_value: Variable,
        operator: ConditionOperator,
    },
    SubQueryCondition {
        sub_query: QueryGenerator<'a>,
        ref_value: Variable,
        operator: ConditionOperator,
    }
}

impl Condition<'_> {
    pub(crate) fn get_table_name(&self) -> String {
        match self {
            Condition::OnMainTable {..} => "main".to_string(),
            Condition::SameSchemaTable { table_name, .. } => format!("{}", table_name),
            Condition::AnotherSchemaTable {
                schema_name,
                table_name, ..} => format!("{}.{}", schema_name, table_name),
            Condition::SubQueryCondition {..} => "subquery".to_string(),
        }
    }
}
