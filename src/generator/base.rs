pub trait Generator {
    fn get_statement() -> String;
    fn get_params() -> Vec<String>;
}

pub enum ConditionOperator {
    Equal,
    NotEqual,
    Greater,
    GreaterEq,
    Lower,
    LowerEq
}
