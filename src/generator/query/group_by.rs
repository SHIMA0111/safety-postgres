use crate::generator::base::{Aggregation, ConditionOperator, ReferenceValue};
use crate::{Column, Variable};

pub(crate) struct Groupings<'a> {
    groupings: Vec<&'a Column<'a>>,
}

impl <'a> Groupings <'a> {
    pub(crate) fn new() -> Groupings<'a> {
        Self {
            groupings: Vec::<&'a Column<'a>>::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.groupings.len()
    }

    pub(crate) fn add_grouping(&mut self, grouping_column: &'a Column<'a>) {
        self.groupings.push(grouping_column);
    }

    pub(crate) fn get_grouping_statement(&self) -> String {
        let grouping_statement = self.groupings
            .iter()
            .map(|grouping| format!("{}", grouping))
            .collect::<Vec<String>>()
            .join(", ");

        format!("{} {}", "GROUP BY", grouping_statement)
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

    pub(crate) fn get_grouping_condition_statement(&self, start_placeholder_num: u16) -> String {
        let mut statement_vec = vec!["HAVING".to_string()];

        for (idx, condition) in self.group_conditions.iter().enumerate() {
            statement_vec.push(condition.get_grouping_condition(idx as u16 + start_placeholder_num));
        }

        statement_vec.join(" ")
    }
}

pub struct GroupCondition<'a> {
    aggregation: &'a Aggregation<'a>,
    ref_value: ReferenceValue<'a>,
    condition_operator: ConditionOperator,
}

impl <'a> GroupCondition<'a> {
    pub fn new(aggregation: &'a Aggregation<'a>,
               condition_operator: ConditionOperator,
               ref_value: ReferenceValue<'a>) -> GroupCondition<'a> {
        Self {
            aggregation,
            ref_value,
            condition_operator,
        }
    }

    pub(crate) fn get_table_name(&self) -> String {
        self.aggregation.get_table_name()
    }

    pub(crate) fn get_grouping_condition(&self, placeholder_num: u16) -> String {
        todo!()
    }

    pub(crate) fn get_grouping_condition_parameter(&self) -> Variable {
        todo!()
    }

    pub fn get_parameter_num(&self) -> u16 {
        self.ref_value.get_parameter_num()
    }
}
