use crate::postgres::conditions::{ComparisonOperator, Conditions, IsJoin, LogicalOperator};
use crate::postgres::join_tables::JoinTables;
use crate::postgres::postgres_base::PostgresBase;
use crate::postgres::sql_base::{InsertRecords, QueryColumns, UpdateSets};

mod postgres;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut postgres = PostgresBase::new("records")?
        .set_schema("validate_schema");
    let column = vec!["user_id", "record_date", "subcategory_id", "work_time", "message_comment"];
    let mut sql_base = InsertRecords::new(&column);
    // sql_base.add_set("record_date", "2023-03-06")?;
    let record = vec!["1", "2024-03-06", "2", "3.6", ""];
    sql_base.add_record(&record)?;
    let record = vec!["3", "2024-03-08", "1", "10.2", "Test strings"];
    sql_base.add_record(&record)?;

    // let mut condition = Conditions::new(IsJoin::False);
    // condition.add_condition("id", "17", ComparisonOperator::Equal, LogicalOperator::FirstCondition)?;

    postgres.connect().await?;
    // postgres.delete(condition).await?;
    postgres.insert(sql_base).await?;
    Ok(())
}