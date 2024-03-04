use crate::postgres::errors::{ConditionError, ConditionErrorGenerator, StatementError};
use crate::postgres::validators::validate_string;

/// Represents a comparison operator.
///
/// # Variants
/// - `Equal`: The equal operator.
/// - `Lower`: The lower than operator.
/// - `Grater`: The greater than operator.
/// - `LowerEq`: The lower than or equal to operator.
/// - `GraterEq`: The greater than or equal to operator.
#[derive(Clone)]
pub(crate) enum ComparisonOperator {
    Equal,
    Lower,
    Grater,
    LowerEq,
    GraterEq,
}


/// Represents whether a table is joined or not.
///
/// # Variants
/// - `True(String, String)`: Specifies that the table is joined. The first `String` represents the schema name and the
/// second `String` represents the table name.
/// - `False`: Specifies that the table has no join.
#[derive(Clone)]
pub(crate) enum IsJoin {
    True(String, String),
    False,
}

/// Represents a logical operator.
///
/// # Variants
/// - `FirstCondition`: The first condition operator.
/// - `And`: The logical AND operator.
/// - `Or`: The logical OR operator.
#[derive(Clone)]
pub(crate) enum LogicalOperator {
    FirstCondition,
    And,
    Or,
}

/// Represents a specific condition used for filtering or joining data.
#[derive(Clone)]
struct Condition {
    key: String,
    operator: ComparisonOperator,
    value: String,
}

/// Represents a set of conditions used for filtering or joining data.
#[derive(Clone)]
pub(crate) struct Conditions {
    logics: Vec<LogicalOperator>,
    conditions: Vec<Condition>,
    is_join: IsJoin,
}

impl Conditions {
    /// Creates a new instance of the structure with the specified join type.
    ///
    /// # Arguments
    ///
    /// * `join` - An `IsJoin` enumeration value representing the type of join.
    ///
    /// # Returns
    ///
    /// A new instance of the structure with empty vectors for logics and conditions, and the specified join type.
    pub(crate) fn new(join: IsJoin) -> Self {
        Self {
            logics: Vec::new(),
            conditions: Vec::new(),
            is_join: join,
        }
    }

    pub(crate) fn add_condition(&mut self, column: &str, value: &str, comparison: ComparisonOperator, condition_chain: LogicalOperator) -> Result<&mut Self, ConditionError> {
        validate_string(column, "column", &ConditionErrorGenerator)?;

        let mut validated_condition_chain: LogicalOperator = condition_chain.clone();
        if let LogicalOperator::FirstCondition = condition_chain  {
            if !self.conditions.is_empty() {
                return Err(ConditionError::InputInvalidError(
                    "Already condition exists. 'FirstCondition' can be used only specifying the first condition.".to_string()));
            }
        }
        else {
            if self.conditions.is_empty() {
                eprintln!("The first condition should have 'FirstCondition' as 'condition_chain'. Automatically converted.");
                validated_condition_chain = LogicalOperator::FirstCondition;
            }
        }

        let condition = Condition {
            key: column.to_string(),
            operator: comparison,
            value: value.to_string(),
        };

        self.logics.push(validated_condition_chain);
        self.conditions.push(condition);

        Ok(self)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }

    pub(super) fn generate_statement_text(&self, start_index: usize) -> Result<String, StatementError> {
        let mut statement_texts: Vec<String> = Vec::new();

        for (index, (condition, logic)) in self.conditions.iter().zip(&self.logics).enumerate() {
            if statement_texts.is_empty() {
                statement_texts.push("WHERE".to_string());
            }
            match logic {
                LogicalOperator::FirstCondition => {},
                LogicalOperator::And => statement_texts.push("AND".to_string()),
                LogicalOperator::Or => statement_texts.push("OR".to_string()),
            }
            let condition_text = condition.generate_statement_text(self.is_join.clone());
            let statement_text = format!("{} ${}", condition_text, index + start_index + 1);
            statement_texts.push(statement_text);
        }

        Ok(statement_texts.join(" "))
    }

    pub(super) fn get_flat_values(&self) -> Vec<String> {
        self.conditions.iter().map(|condition| condition.value.clone()).collect::<Vec<String>>()
    }
}

impl Condition {
    fn generate_statement_text(&self, is_join: IsJoin) -> String {
        let table_name = match is_join {
            IsJoin::True(schema, table_name) => format!("{}.{}.{}", schema, table_name, self.key),
            IsJoin::False => self.key.to_string(),
        };
        let operator = match self.operator {
            ComparisonOperator::Equal => "=",
            ComparisonOperator::Lower => "<",
            ComparisonOperator::LowerEq => "<=",
            ComparisonOperator::Grater => ">",
            ComparisonOperator::GraterEq => ">="
        };

        format!("{} {}", table_name, operator)
    }
}
