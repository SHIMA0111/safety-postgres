use crate::generator::base::{BindMethod, ConditionOperator, ReferenceValue};
use crate::utils::errors::GeneratorError;
use crate::{Column, Variable};

pub(crate) struct Conditions<'a> {
    conditions: Vec<Condition<'a>>,
    bind_methods: Vec<BindMethod>,
}

impl <'a> Conditions<'a> {
    pub(crate) fn new() -> Conditions<'a> {
        Self {
            conditions: Vec::<Condition<'a>>::new(),
            bind_methods: Vec::<BindMethod>::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.conditions.len()
    }

    pub(crate) fn add_condition(&mut self,
                                condition: Condition<'a>,
                                bind_method: BindMethod) -> Result<(), GeneratorError> {

        if bind_method == BindMethod::FirstCondition && self.conditions.len() != 0 {
            return Err(GeneratorError::InconsistentConfigError(
                "'FirstCondition' indicates the first condition but already exist some conditions.".to_string()
            ))
        }
        else if bind_method != BindMethod::FirstCondition && self.conditions.len() == 0 {

            self.bind_methods.push(BindMethod::FirstCondition);
        }
        else {
            self.bind_methods.push(bind_method)
        }

        self.conditions.push(condition);
        Ok(())
    }

    pub(crate) fn get_condition_statement(&self, start_placeholder_num: u16) -> String {
        let mut statement_vec = vec!["WHERE".to_string()];

        for (idx, (condition, bind_method)) in self.conditions.iter().zip(&self.bind_methods).enumerate() {
            statement_vec.push(format!("{}", bind_method));
            statement_vec.push(condition.get_condition_statement(idx as u16 + start_placeholder_num));
        }

        statement_vec.join(" ")
    }
}

pub struct Condition<'a> {
    column: &'a Column<'a>,
    ref_value: ReferenceValue<'a>,
    operator: ConditionOperator,
}

impl <'a> Condition<'a> {
    pub fn new(
        column: &'a Column<'a>,
        condition_ref_value: ReferenceValue<'a>,
        condition_operator: ConditionOperator) -> Condition<'a> {

        Condition {
            column,
            ref_value: condition_ref_value,
            operator: condition_operator,
        }
    }

    pub(crate) fn get_table_name(&self) -> String {
        self.column.get_table_name()
    }

    pub(crate) fn get_condition_statement(&self, placeholder_num: u16) -> String {
        todo!()
    }

    pub(crate) fn get_condition_parameter(&self) -> Variable {
        todo!()
    }

    pub fn get_parameter_num(&self) -> u16 {
        self.column.get_parameter_num() + self.ref_value.get_parameter_num()
    }
}
