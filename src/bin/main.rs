use std::error::Error;
use tokio_postgres::Row;
use safety_postgres::access::json_parser::row_to_json;
use safety_postgres::access::postgres::PostgresBase;
use safety_postgres::access::sql_base::{InsertRecords, QueryColumns};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let query_columns = QueryColumns::new(true);
    let mut postgres = PostgresBase::new("records")?;
    postgres.set_schema("validate_schema");
    postgres.connect().await?;
    println!("{:?}", postgres);

    let columns = vec!["user_id", "record_date", "subcategory_id", "work_time", "message_comment"];
    let mut insert_record = InsertRecords::new(&columns);
    insert_record.add_record(&vec!["1", "2024-03-29", "3", "23.8d", "domain_data"])?;
    postgres.insert(&insert_record).await?;

    let query_result = postgres.query_raw(&query_columns).await?;

    let json = row_to_json(&query_result)?;
    println!("{}", json);

    Ok(())
}