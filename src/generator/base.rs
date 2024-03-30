pub trait Generator {
    fn get_statement(&self) -> String;
    fn get_params(&self) -> Vec<String>;
}

pub enum ConditionOperator {
    Equal,
    NotEqual,
    Greater,
    GreaterEq,
    Lower,
    LowerEq
}

impl ConditionOperator {
    pub(crate) fn get_symbol(&self) -> String {
        match self {
            ConditionOperator::Equal => "=".to_string(),
            ConditionOperator::NotEqual => "!=".to_string(),
            ConditionOperator::Greater => ">".to_string(),
            ConditionOperator::GreaterEq => ">=".to_string(),
            ConditionOperator::Lower => "<".to_string(),
            ConditionOperator::LowerEq => "<=".to_string(),
        }
    }
}

pub enum ChainMethod {
    And,
    Or
}

impl ChainMethod {
    pub(crate) fn get_string(&self) -> String {
        match self {
            ChainMethod::And => "AND".to_string(),
            ChainMethod::Or => "OR".to_string(),
        }
    }
}
