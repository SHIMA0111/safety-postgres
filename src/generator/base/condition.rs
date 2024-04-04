use crate::generator::base::{BindMethod, ConditionOperator, GeneratorPlaceholder, GeneratorPlaceholderWrapper, MainGenerator, Parameters, ReferenceValue};
use crate::utils::errors::GeneratorError;
use crate::Column;

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
}

impl GeneratorPlaceholderWrapper for Conditions<'_> {
    fn get_total_statement(&self, start_placeholder: u16) -> String {
        let mut statement_vec = vec!["WHERE".to_string()];

        let mut index = start_placeholder;

        for (condition, bind_method) in self.conditions.iter().zip(&self.bind_methods) {
            statement_vec.push(format!("{}", bind_method));
            statement_vec.push(condition.get_statement(index));
            index += condition.get_parameters_number();
        }

        statement_vec.join(" ")
    }

    fn get_all_params(&self) -> Parameters {
        let mut params = Parameters::new();

        for condition in &self.conditions {
            params += condition.get_params();
        }
        params
    }

    fn len(&self) -> usize {
        self.conditions.len()
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
}

impl GeneratorPlaceholder for Condition<'_> {
    fn get_statement(&self, start_placeholder_number: u16) -> String {
        match &self.ref_value {
            ReferenceValue::Variable(_) => format!("{} {} ${}", self.column, self.operator, start_placeholder_number),
            ReferenceValue::SubQueryAggregation(query) => {
                query.get_statement()
            }
        }
    }

    fn get_params(&self) -> Parameters {
        self.ref_value.get_parameters()
    }

    fn get_parameters_number(&self) -> u16 {
        self.column.get_parameter_num() + self.ref_value.get_parameter_num()
    }

    fn get_table_name(&self) -> String {
        self.column.get_table_name()
    }
}
