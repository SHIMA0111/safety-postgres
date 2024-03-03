use crate::postgres::conditions::{ComparisonOperator, Conditions, IsJoin, LogicalOperator};
use crate::postgres::join_tables::JoinTables;
use crate::postgres::postgres_base::PostgresBase;
use crate::postgres::sqls::{InsertRecords, QueryColumns, UpdateSets};

mod postgres;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sql_base = QueryColumns::new(true);
    let mut join_base = JoinTables::new();
    let join_columns = vec!["who", "test"];
    let destinations = vec!["who_id", "test_main"];
    join_base.add_join_table("other_table", "another", &join_columns, &destinations)?;
    let mut condition_base = Conditions::new(IsJoin::True("another".to_string(), "other_table".to_string()));
    condition_base.add_condition("column1", "4", ComparisonOperator::Grater, LogicalOperator::FirstCondition)?;
    condition_base.add_condition("column2", "Who", ComparisonOperator::Equal, LogicalOperator::And)?;
    condition_base.add_condition("column3", "9.3", ComparisonOperator::Lower, LogicalOperator::Or)?;
    let postgres = PostgresBase::new("records")?.set_schema("work_time");
    postgres.query_inner_join_conditions(sql_base, join_base, condition_base).await?;
    let keys = vec!["column1", "column2", "column3", "column4"];
    let values = vec![
        vec!["a", "b", "c", "d", "e"],
        vec!["a1", "b1", "c1", "d1", "e1"],
        vec!["a2", "b2", "c2", "d2", "e2"],
        vec!["a3", "b3", "c3", "d3", "e3"],
    ];
    let mut insert_values = InsertRecords::new(&keys);
    for index in 0..values[0].len() {
        let values = values.iter().map(|value| value[index]).collect::<Vec<&str>>();
        insert_values.add_value(&values)?;
    }
    postgres.insert(insert_values).await?;
    let keys = vec!["column1", "column2"];
    let values = vec!["value1", "value2"];
    let condition_keys = vec!["column_n"];
    let condition_values = vec!["value_n"];
    let logicals = vec![LogicalOperator::FirstCondition, LogicalOperator::And];

    let mut update_set = UpdateSets::new();
    for (key, value) in keys.iter().zip(values) {
        update_set.add_set(key, value)?;
    }
    let mut conditions = Conditions::new(IsJoin::False);
    for (key, (value, logical_op)) in condition_keys.iter().zip(condition_values.iter().zip(logicals)) {
        conditions.add_condition(key, value, ComparisonOperator::Equal, logical_op)?;
    }

    postgres.update_condition(update_set, conditions).await?;

    let condition_keys = vec!["column_id"];
    let condition_values = vec!["1"];
    let condition_operator = vec![LogicalOperator::FirstCondition];

    let mut conditions = Conditions::new(IsJoin::False);
    for (key, (value, logical_op)) in condition_keys.iter().zip(condition_values.iter().zip(condition_operator)) {
        conditions.add_condition(key, value, ComparisonOperator::Equal, logical_op)?;
    }
    postgres.delete(conditions).await?;
    Ok(())
}