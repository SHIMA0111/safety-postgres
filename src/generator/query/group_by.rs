use crate::generator::base::{Aggregation, ConditionOperator};
use crate::utils::helpers::Variable;

pub(crate) struct Groupings<'a> {
    groupings: Vec<Grouping<'a>>
}

impl <'a> Groupings <'a> {
    pub(crate) fn new() -> Groupings<'a> {
        Self {
            groupings: Vec::<Grouping<'a>>::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.groupings.len()
    }

    pub(crate) fn add_grouping(&mut self, grouping: Grouping<'a>) {
        self.groupings.push(grouping);
    }

    pub(crate) fn get_grouping_statement(&self) -> String {
        let grouping_statement = self.groupings
            .iter()
            .map(|grouping| grouping.get_table_name())
            .collect::<Vec<String>>()
            .join(", ");

        format!("{} {}", "GROUP BY", grouping_statement)
    }
}

pub enum Grouping<'a> {
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

impl Grouping<'_> {
    pub(crate) fn get_table_name(&self) -> String {
        match self {
            Grouping::OnMainTable { .. } => "main".to_string(),
            Grouping::SameSchemaTable { table_name, .. } => table_name.to_string(),
            Grouping::AnotherSchemaTable {
                schema_name,
                table_name, .. } => format!("{}.{}", schema_name, table_name),
        }
    }
}


pub(crate) struct GroupConditions<'a> {
    group_conditions: Vec<GroupCondition<'a>>,
}

impl <'a> GroupConditions<'a> {
    pub(crate) fn new() -> GroupConditions<'a> {
        Self {
            group_conditions: Vec::<GroupCondition<'a>>::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.group_conditions.len()
    }

    pub(crate) fn add_group_condition(&mut self, group_condition: GroupCondition<'a>) {
        self.group_conditions.push(group_condition);
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

    pub(crate) fn get_table_name(&self) -> String {
        self.aggregation.get_table_name()
    }

    pub(crate) fn get_grouping_condition_statement(&self) -> String {
        todo!()
    }
}
