use crate::access::conditions::IsInJoinedTable::Yes;
use crate::access::errors::{ConditionError, ConditionErrorGenerator};
use crate::access::validators::validate_string;

/// Provides the available comparison operators for standardizing input for the `Conditions.add_condition()` method.
///
/// The available comparison operators are:
///  - `Equal`: Represents the equality condition, where the column and the value are chained by "="
///  - `Lower`: Represents the less than condition, where the column and the value are chained by "<"
///  - `Greater`: Represents the greater than condition, where the column and the value are chained by ">"
///  - `LowerEq`: Represents the less than or equal to condition, where the column and the value are chained by "<="
///  - `GreaterEq`: Represents the greater than or equal to condition, where the column and the value are chained by ">="
#[derive(Clone)]
pub enum ComparisonOperator {
    Equal,
    Lower,
    Grater,
    LowerEq,
    GraterEq,
}

/// Represents whether the column is from a joined table or not.
///
/// The available variants are:
///  - `Yes`: Represents that the column is from a joined tables.
///    It contains the following fields:
///      - `schema_name`: The name of the schema of the joined table which has the condition column (if applicable).
///      - `table_name`: The name of the joined table which has the condition column.
///  - `No`: Represents that the column is not from a joined table.
#[derive(Clone)]
pub enum IsInJoinedTable {
    Yes {
        schema_name: String,
        table_name: String,
    },
    No,
}

/// Provides the available logical operators for combining conditions between a previous condition.
///
/// The available logical operators are:
///  - `FirstCondition`: Represents the first condition, used to start the condition chain.
///  - `And`: Represents the logical "AND" operator, which combines multiple conditions with a logical AND.
///  - `Or`: Represents the logical "OR" operator, which combines multiple conditions with a logical OR.
#[derive(Clone, PartialEq, Debug)]
pub enum LogicalOperator {
    FirstCondition,
    And,
    Or,
}

/// Represents a condition to be used in an execution.
///
/// # Fields
/// - `is_joined_table_condition`: A flag indicating whether the condition belongs to a joined table or the main table.
/// - `key`: The column name to apply the condition on.
/// - `operator`: The comparison operator to use for the condition.
/// - `value`: The value to compare against.
#[derive(Clone)]
struct Condition {
    is_joined_table_condition: IsInJoinedTable,
    key: String,
    operator: ComparisonOperator,
    value: String,
}

/// Represents a set of conditions to be used in an execution.
///
/// # Example
/// ```rust
/// use safety_postgres::access::conditions::ComparisonOperator::{Equal, Lower};
/// use safety_postgres::access::conditions::Conditions;
/// use safety_postgres::access::conditions::IsInJoinedTable::{No, Yes};
/// use safety_postgres::access::conditions::LogicalOperator::{And, FirstCondition};
///
/// let mut conditions = Conditions::new();
///
/// conditions.add_condition("column1", "condition1", Equal, FirstCondition, No).expect("add condition failed");
/// conditions.add_condition("column2", "condition2", Lower, And, Yes{
///     schema_name: "schema_name".to_string(), table_name: "table_name".to_string()
/// }).expect("add condition failed");
///
/// assert_eq!(
///     conditions.get_condition_text(),
///     "column1 = condition1 AND schema_name.table_name.column2 < condition2")
///
/// ```
/// And you can specify the condition more intuitive using
/// `Conditions.add_condition_from_str(column, value, condition_operator, condition_chain_operator, is_joined_table_condition)` method.
///
/// ```rust
/// use safety_postgres::access::conditions::{Conditions, IsInJoinedTable};
///
/// let mut conditions = Conditions::new();
///
/// conditions.add_condition_from_str(
///     "column1",
///     "condition1",
///     "eq",
///     "", IsInJoinedTable::No).expect("add failed");
/// conditions.add_condition_from_str(
///     "column2",
///     "condition2",
///     ">=",
///     "or",
///     IsInJoinedTable::Yes{
///         schema_name: "schema_name".to_string(), table_name: "table_name".to_string()
///     }).expect("add failed");
///
/// assert_eq!(
///     conditions.get_condition_text(),
///     "column1 = condition1 OR schema_name.table_name.column2 >= condition2")
/// ```
#[derive(Clone)]
pub struct Conditions {
    logics: Vec<LogicalOperator>,
    conditions: Vec<Condition>,
}

impl Conditions {
    /// Creates a new empty `Conditions` instance.
    pub fn new() -> Self {
        Self {
            logics: Vec::new(),
            conditions: Vec::new(),
        }
    }

    /// Adds a condition based on the input string parameters.
    ///
    /// # Arguments
    ///
    /// * `column` - The name of the column to compare.
    /// * `value` - The value to compare against.
    /// * `comparison_operator` - The operator to use for the comparison.
    ///   * Available operator:
    ///     * Equal: "=", "equal", "eq"
    ///     * Greater: ">", "greater", "gt"
    ///     * GreaterEqual: ">=", "greater_equal", "ge", "greater_eq"
    ///     * Lower: "<", "lower", "lt"
    ///     * LowerEqual: "<=", "lower_equal", "le", "lower_eq"
    /// * `condition_chain_operator` - The operator to use for chaining multiple conditions.
    ///   * Available operator:
    ///     * FirstCondition(there is no previous condition): "", "first", "none"
    ///     * And: "and", "&"
    ///     * Or: "or", "|"
    /// * `is_joined_table_condition` - Whether the condition is for a joined table.
    ///
    /// # Errors
    ///
    /// Returns a `ConditionError` if there's an error in the input parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use safety_postgres::access::conditions::Conditions;
    /// use safety_postgres::access::conditions::IsInJoinedTable::No;
    ///
    /// let mut conditions = Conditions::new();
    /// conditions
    ///     .add_condition_from_str("name", "John", "=", "first", No)
    ///     .expect("adding condition failed");
    /// conditions
    ///     .add_condition_from_str("age", "40", "le", "or", No)
    ///     .expect("adding condition failed");
    ///
    /// assert_eq!(conditions.get_condition_text(), "name = John OR age <= 40")
    /// ```
    pub fn add_condition_from_str(&mut self, column: &str, value: &str, comparison_operator: &str, condition_chain_operator: &str, is_joined_table_condition: IsInJoinedTable) -> Result<&mut Self, ConditionError> {
        let comparison_op = match comparison_operator {
            "=" | "equal" | "eq" => ComparisonOperator::Equal,
            ">" | "greater" | "gt" => ComparisonOperator::Grater,
            ">=" | "greater_equal" | "ge" | "greater_eq" => ComparisonOperator::GraterEq,
            "<" | "lower" | "lt" => ComparisonOperator::Lower,
            "<=" | "lower_equal" | "le" | "lower_eq" => ComparisonOperator::LowerEq,
            _ => return Err(ConditionError::InputInvalidError(format!("'comparison operator' can select symbol('=', '>', '<', '>=', '<=') or some specify string, but got {}", comparison_operator))),
        };
        let condition_chain_op = match condition_chain_operator {
            "" | "first" | "none" => LogicalOperator::FirstCondition,
            "and" | "&" => LogicalOperator::And,
            "or" | "|" => LogicalOperator::Or,
            _ => return Err(ConditionError::InputInvalidError(format!("'condition_chain_operator' indicates the chain operator between the previous condition and the current condition by symbols('&', '|') or specified strings, but got {}", condition_chain_operator))),
        };

        self.add_condition(column, value, comparison_op, condition_chain_op, is_joined_table_condition)
    }

    /// Adds a condition to the query builder.
    ///
    /// # Arguments
    ///
    /// * `column` - The column name to which the condition is applied.
    /// * `value` - The value for comparison.
    /// * `comparison` - The operator used for comparison.
    /// * `condition_chain` - The logical operator used to chain the conditions.
    /// * `is_joined_table_condition` - Indicates whether the condition is for a joined table or not.
    ///
    /// # Returns
    ///
    /// A mutable reference to `Self (Conditions)` if the condition is successfully added, otherwise a `ConditionError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use safety_postgres::access::conditions::Conditions;
    /// use safety_postgres::access::conditions::{ComparisonOperator, LogicalOperator, IsInJoinedTable};
    ///
    /// let mut conditions = Conditions::new();
    ///
    /// let _ = conditions.add_condition(
    ///     "name",
    ///     "John",
    ///     ComparisonOperator::Equal,
    ///     LogicalOperator::FirstCondition,
    ///     IsInJoinedTable::No).expect("add condition failed")
    ///     .add_condition(
    ///     "age",
    ///     "40",
    ///     ComparisonOperator::LowerEq,
    ///     LogicalOperator::Or,
    ///     IsInJoinedTable::No).expect("add condition failed");
    ///
    /// assert_eq!(conditions.get_condition_text(), "name = John OR age <= 40");
    /// ```
    pub fn add_condition(&mut self, column: &str, value: &str, comparison: ComparisonOperator, condition_chain: LogicalOperator, is_joined_table_condition: IsInJoinedTable) -> Result<&mut Self, ConditionError> {
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

        match &is_joined_table_condition {
            Yes {schema_name, table_name} => {
                if !schema_name.is_empty() && table_name.is_empty() {
                    return Err(
                        ConditionError::InputInvalidError(
                            "`table_name` must be specified when `schema_name` name is specified".to_string()
                        ))
                }
            },
            IsInJoinedTable::No => {}
        }

        let condition = Condition {
            is_joined_table_condition,
            key: column.to_string(),
            operator: comparison,
            value: value.to_string(),
        };

        self.logics.push(validated_condition_chain);
        self.conditions.push(condition);

        Ok(self)
    }

    /// Checks if the conditions is empty.
    ///
    /// # Returns
    ///
    /// Returns `true` if the conditions is empty, `false` otherwise.
    ///
    pub(super) fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }

    /// Generates the SQL statement text for the conditions.
    ///
    /// # Arguments
    ///
    /// * `start_index` - The starting index of the statement parameters' placeholder.
    ///
    /// # Returns
    ///
    /// The generated SQL statement text.
    ///
    /// # Example
    ///
    /// ```
    /// use safety_postgres::access::conditions::{Conditions, IsInJoinedTable};
    ///
    /// let mut conditions = Conditions::new();
    /// conditions.add_condition_from_str("name", "John", "=", "first", IsInJoinedTable::No).expect("add condition failed");
    /// conditions.add_condition_from_str("age", "40", "le", "or", IsInJoinedTable::No).expect("add condition failed");
    ///
    /// let statement_text = conditions.get_condition_text();
    ///
    /// assert_eq!(conditions.get_condition_text(), "name = John OR age <= 40");
    /// ```
    pub(super) fn generate_statement_text(&self, start_index: usize) -> String {
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
            let condition_text = condition.generate_statement_text();
            let statement_text = format!("{} ${}", condition_text, index + start_index + 1);
            statement_texts.push(statement_text);
        }

        statement_texts.join(" ")
    }

    /// Returns the condition text generated by the conditions you set.
    ///
    ///
    /// # Returns
    ///
    /// The set condition as a `String`.
    pub fn get_condition_text(&self) -> String {
        let mut conditions_txt: Vec<String> = Vec::new();

        for (condition, logic) in self.conditions.iter().zip(&self.logics) {
            match logic {
                LogicalOperator::FirstCondition => {},
                LogicalOperator::And => conditions_txt.push("AND".to_string()),
                LogicalOperator::Or => conditions_txt.push("OR".to_string()),
            }
            let condition_txt = format!("{} {}", condition.generate_statement_text(), condition.value);
            conditions_txt.push(condition_txt);
        }

        conditions_txt.join(" ")
    }

    /// Retrieves the values of the conditions as flatten vec.
    pub(super) fn get_flat_values(&self) -> Vec<String> {
        self.conditions.iter().map(|condition| condition.value.clone()).collect::<Vec<String>>()
    }
}

impl Condition {
    /// Generates one part of the condition by the set condition.
    fn generate_statement_text(&self) -> String {
        let table_name = match &self.is_joined_table_condition {
            Yes{ schema_name, table_name } => {
                if schema_name.is_empty() & table_name.is_empty() {
                    self.key.to_string()
                }
                else if schema_name.is_empty() {
                    format!("{}.{}", table_name, self.key)
                }
                else {
                    format!("{}.{}.{}", schema_name, table_name, self.key)
                }
            },
            IsInJoinedTable::No => self.key.to_string(),
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


#[cfg(test)]
mod tests {
    use crate::access::errors::ConditionError;
    use super::{Conditions, ComparisonOperator, LogicalOperator, IsInJoinedTable};

    /// Tests that conditions can be added properly and results in the correct condition text and statement.
    #[test]
    fn test_add_and_get_condition() {
        let mut conditions = Conditions::new();
        conditions.add_condition(
            "column1",
            "value1",
            ComparisonOperator::Equal,
            LogicalOperator::FirstCondition,
            IsInJoinedTable::No).unwrap();
        conditions.add_condition(
            "column2",
            "value2",
            ComparisonOperator::Grater,
            LogicalOperator::And,
            IsInJoinedTable::No).unwrap();
        conditions.add_condition(
            "column3",
            "value3",
            ComparisonOperator::LowerEq,
            LogicalOperator::Or,
            IsInJoinedTable::Yes {
                schema_name: "schema1".to_string(),
                table_name: "table1".to_string()
            }).unwrap();

        let expected_statement = "WHERE column1 = $1 AND column2 > $2 OR schema1.table1.column3 <= $3";
        let expected_text = "column1 = value1 AND column2 > value2 OR schema1.table1.column3 <= value3";

        assert_eq!(conditions.get_condition_text(), expected_text);
        assert_eq!(conditions.generate_statement_text(0), expected_statement);

        let expected_values = vec!["value1".to_string(), "value2".to_string(), "value3".to_string()];

        assert_eq!(conditions.get_flat_values(), expected_values);
    }

    /// Tests adding and getting conditions using string representation of operators and confirms
    /// it results in the correct condition text and statement.
    #[test]
    fn test_add_get_condition_by_str() {
        let mut conditions = Conditions::new();
        let _ =
            conditions.
                add_condition_from_str(
                    "column1",
                    "value1",
                    "equal",
                    "",
                    IsInJoinedTable::No).unwrap()
                .add_condition_from_str(
                    "column2",
                    "value2",
                    ">=",
                    "or",
                    IsInJoinedTable::No).unwrap();

        let expected_statement = "WHERE column1 = $1 OR column2 >= $2";
        let expected_text = "column1 = value1 OR column2 >= value2";

        assert_eq!(conditions.get_condition_text().as_str(), expected_text);
        assert_eq!(conditions.generate_statement_text(0).as_str(), expected_statement);

        let expected_values = vec!["value1", "value2"];

        assert_eq!(conditions.get_flat_values(), expected_values);
    }

    /// Tests providing invalid string as comparison operator results in an appropriate error.
    #[test]
    fn test_invalid_comparison_str_input() {
        let mut conditions = Conditions::new();
        let Err(e) = conditions.add_condition_from_str(
            "column1",
            "value1",
            "+",
            "",
            IsInJoinedTable::No,
        ) else { panic!() };

        assert_eq!(e, ConditionError::InputInvalidError(format!(
            "'comparison operator' can select symbol('=', '>', '<', '>=', '<=') or some specify string, but got {}",
            "+")));
    }

    /// Tests providing invalid string as condition chain operator results in an appropriate error.
    #[test]
    fn test_invalid_chain_str_input() {
        let mut conditions = Conditions::new();
        let Err(e) = conditions.add_condition_from_str(
            "column1",
            "value1",
            "=",
            "test",
            IsInJoinedTable::No,
        ) else { panic!() };

        assert_eq!(e, ConditionError::InputInvalidError(format!(
            "'condition_chain_operator' indicates the chain operator between the previous condition and the current condition by symbols('&', '|') or specified strings, but got {}",
            "test")));

    }

    /// Tests providing invalid column name results in an appropriate error.
    #[test]
    fn test_invalid_column() {
        let mut conditions = Conditions::new();
        let Err(e) = conditions.add_condition(
            "column1;",
            "value1",
            ComparisonOperator::Equal,
            LogicalOperator::FirstCondition,
            IsInJoinedTable::No) else { panic!() };
        assert_eq!(e,
                   ConditionError::InputInvalidError(
                       format!(
                           "'{}' has invalid characters. '{}' allows alphabets, numbers and under bar only.",
                           "column1;", "column")
                   ));
    }

    /// Tests that the default logical operator for the first condition is "FirstCondition".
    #[test]
    fn test_default_value() {
        let mut conditions = Conditions::new();
        conditions.add_condition(
            "column1",
            "value1",
            ComparisonOperator::Equal,
            LogicalOperator::And,
            IsInJoinedTable::No).unwrap();

        assert_eq!(conditions.logics[0], LogicalOperator::FirstCondition);
    }

    /// Tests that applying "FirstCondition" more than once results in an appropriate error.
    #[test]
    fn test_multiple_declaration_first_condition() {
        let mut conditions = Conditions::new();
        conditions.add_condition(
            "column1",
            "value1",
            ComparisonOperator::Equal,
            LogicalOperator::FirstCondition,
            IsInJoinedTable::No
        ).unwrap();
        let Err(e) = conditions.add_condition(
            "column2",
            "value2",
            ComparisonOperator::Equal,
            LogicalOperator::FirstCondition,
            IsInJoinedTable::No
        ) else { panic!() };

        assert_eq!(e,
                   ConditionError::InputInvalidError(
                       "Already condition exists. 'FirstCondition' can be used only specifying the first condition."
                           .to_string()
                   ));
    }
}
