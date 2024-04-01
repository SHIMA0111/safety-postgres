use crate::generator::base::{BindMethod, ConditionOperator};
use crate::generator::query::QueryGenerator;
use crate::utils::errors::GeneratorError;
use crate::utils::helpers::Variable;

pub(crate) struct Conditions<'a> {
    conditions: Vec<Condition<'a>>,
    bind_methods: Vec<BindMethod>
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

    pub(crate) fn add_condition(&mut self, condition: Condition<'a>) -> Result<(), GeneratorError> {
        if let BindMethod::FirstCondition = condition.get_bind_method() {
            if self.conditions.len() != 0 {
                return Err(GeneratorError::InconsistentConfigError(
                    "'FirstCondition' indicates the first condition but already exist some conditions.".to_string()
                ))
            }
        }
        else {
            if self.conditions.len() == 0 {
                self.bind_methods.push(BindMethod::FirstCondition);
            }
            else {
                self.bind_methods.push(condition.get_bind_method());
            }
        }

        self.conditions.push(condition);
        Ok(())
    }

    pub(crate) fn get_condition_statement(&self) -> String {
        let mut statement_vec = vec!["WHERE".to_string()];

        for (condition, bind_method) in self.conditions.iter().zip(&self.bind_methods) {
            if let BindMethod::FirstCondition = bind_method {}
            else {
                statement_vec.push(format!("{}", bind_method));
            }
            statement_vec.push(condition.get_condition_statement());
        }

        statement_vec.join(" ")
    }
}

pub enum Condition<'a> {
    OnMainTable {
        column: &'a str,
        ref_value: Variable,
        operator: ConditionOperator,
        bind_method: BindMethod,
    },
    SameSchemaTable {
        table_name: &'a str,
        column: &'a str,
        ref_value: Variable,
        operator: ConditionOperator,
        bind_method: BindMethod,
    },
    AnotherSchemaTable {
        schema_name: &'a str,
        table_name: &'a str,
        column: &'a str,
        ref_value: Variable,
        operator: ConditionOperator,
        bind_method: BindMethod
    },
    SubQueryCondition {
        sub_query: QueryGenerator<'a>,
        ref_value: Variable,
        operator: ConditionOperator,
        bind_method: BindMethod,
    }
}

impl <'a> Condition<'a> {
    pub(crate) fn get_table_name(&self) -> String {
        match self {
            Condition::OnMainTable {..} => "main".to_string(),
            Condition::SameSchemaTable { table_name, .. } => format!("{}", table_name),
            Condition::AnotherSchemaTable {
                schema_name,
                table_name, ..} => format!("{}.{}", schema_name, table_name),
            Condition::SubQueryCondition {..} => "sub_query".to_string(),
        }
    }

    pub(crate) fn get_condition_statement(&self) -> String {
        match self {
            Self::OnMainTable {
                column,
                ref_value,
                operator, ..} => format!("{} {} {}", column, operator, ref_value),
            Self::SameSchemaTable {
                table_name,
                column,
                ref_value,
                operator, ..
            } => format!("{}.{} {} {}", table_name, column, operator, ref_value),
            Self::AnotherSchemaTable {
                schema_name,
                table_name,
                column,
                ref_value,
                operator, ..
            } => format!("{}.{}.{} {} {}", schema_name, table_name,column, operator, ref_value),
            Self::SubQueryCondition {..} => todo!(),
        }
    }

    fn get_bind_method(&self) -> BindMethod {
        match self {
            Condition::OnMainTable { bind_method, .. } => bind_method.clone(),
            Condition::SameSchemaTable { bind_method, .. } => bind_method.clone(),
            Condition::AnotherSchemaTable { bind_method, .. } => bind_method.clone(),
            Condition::SubQueryCondition { bind_method, .. } => bind_method.clone(),
        }
    }
}
