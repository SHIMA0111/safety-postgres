use crate::generator::base::{Aggregation, ConditionOperator, GeneratorPlaceholder, GeneratorPlaceholderWrapper, MainGenerator, Parameters, ReferenceValue};
use crate::Column;

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

    pub(crate) fn add_group_condition(&mut self, group_condition: GroupCondition<'a>) {
        self.group_conditions.push(group_condition);
    }
}

impl GeneratorPlaceholderWrapper for GroupConditions<'_> {
    fn get_total_statement(&self, start_placeholder: u16) -> String {
        let mut statement_vec = vec!["HAVING".to_string()];

        let mut index = start_placeholder;

        for condition in &self.group_conditions {
            statement_vec.push(condition.get_statement(index));
            index += condition.get_parameters_number();
        }

        statement_vec.join(" ")
    }

    fn get_all_params(&self) -> Parameters {
        let mut params = Parameters::new();

        for grouping_condition in &self.group_conditions {
            params += grouping_condition.get_params();
        }

        params
    }

    fn len(&self) -> usize {
        self.group_conditions.len()
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
}

impl GeneratorPlaceholder for GroupCondition<'_> {
    fn get_statement(&self, start_placeholder_number: u16) -> String {
        match &self.ref_value {
            ReferenceValue::Variable(_) => format!("{} {} ${}", self.aggregation, self.condition_operator, start_placeholder_number),
            ReferenceValue::SubQueryAggregation(query) => query.get_statement()
        }
    }

    fn get_params(&self) -> Parameters {
        self.ref_value.get_parameters()
    }

    fn get_parameters_number(&self) -> u16 {
        self.ref_value.get_parameter_num()
    }

    fn get_table_name(&self) -> String {
        self.aggregation.get_table_name()
    }
}
