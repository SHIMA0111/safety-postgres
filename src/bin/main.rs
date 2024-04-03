use std::error::Error;
use rust_decimal::Decimal;
use tokio::io::Join;
use safety_postgres::connector::connection_config::ConnectionConfig;
use safety_postgres::connector::Connector;
use safety_postgres::generator::base::{Aggregation, BindMethod, ConditionOperator, Generator, SortMethod, SortRule};
use safety_postgres::generator::base::BindMethod::FirstCondition;
use safety_postgres::generator::base::condition::Condition;
use safety_postgres::generator::base::ConditionOperator::Equal;
use safety_postgres::generator::base::join_table::{JoinColumn, JoinTable, JoinType};
use safety_postgres::generator::base::join_table::JoinType::Inner;
use safety_postgres::generator::query::{QueryGenerator};
use safety_postgres::generator::query::query_column::{QueryColumn, QueryColumns};
use safety_postgres::utils::helpers::{Column, Pair, Schema, Table, Variable};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = ConnectionConfig::set_config(
        "app", "password", "localhost", 5432, "postgres"
    );

    let connection = Connector::connect(config).await?;

    let schema = Schema::new("work_time");
    let main_table = schema.get_table("records");

    let query_columns = QueryColumns::AllColumns(&main_table);

    let mut query_generator = QueryGenerator::new(&main_table, &query_columns);

    let joined_table = schema.get_table("subcategories");
    let mut query_columns_joined = QueryColumns::create_specify_columns();

    let column1 = main_table.get_column("subcategory_id");
    let column2 = joined_table.get_column("subcategory");

    query_columns_joined.add_as_is_column(&column2)?;

    let join_column = joined_table.get_column("id");
    let mut join_table1 = JoinTable::new(&joined_table, &query_columns_joined, Inner);
    join_table1.add_join_columns(Pair::new(&column1, &join_column), Equal, FirstCondition);

    query_generator.add_join_table(&join_table1)?;

    let condition_column = main_table.get_column("work_hour");
    let test_value = Variable::from(2);
    let value = Variable::Decimal(Decimal::from_f32_retain(1.1).unwrap());
    let condition = Condition::new(&condition_column, &value, ConditionOperator::GreaterEq);
    query_generator.add_condition(&condition, BindMethod::FirstCondition)?;

    let sort_rule = SortRule::new(&column2, SortMethod::Asc);
    query_generator.add_sort_rule(&sort_rule)?;

    println!("{}", query_generator.get_statement());

    Ok(())
}

