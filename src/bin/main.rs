use std::error::Error;
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
    insert_record.add_record(&vec!["1", "24-11-01T21:02:05+09:00", "3", "23.8dec", "domain_data"])?;
    postgres.insert(&insert_record).await?;
    let json = postgres.query_json(&query_columns).await?;
    println!("{}", json);

    Ok(())
}