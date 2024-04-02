use std::error::Error;
use rust_decimal::Decimal;
use safety_postgres::connector::connection_config::ConnectionConfig;
use safety_postgres::connector::Connector;
use safety_postgres::generator::base::{Aggregation, BindMethod, ConditionOperator, Generator, SortMethod, SortRule};
use safety_postgres::generator::base::condition::Condition;
use safety_postgres::generator::base::join_table::{JoinColumn, JoinTable, JoinType};
use safety_postgres::generator::query::{QueryGenerator};
use safety_postgres::generator::query::query_column::{QueryColumn, QueryColumns};
use safety_postgres::utils::helpers::{Column, Pair, Schema, Table, Variable};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = ConnectionConfig::set_config(
        "app", "password", "localhost", 5432, "postgres"
    );

    let connection = Connector::connect(config).await?;

    let schema = Schema::new("validate_schema");
    let main_table = schema.get_table("records");

    let query_columns = QueryColumns::AllColumns(&main_table);

    let mut query_generator = QueryGenerator::new(&main_table, &query_columns);

    let joined_table = schema.get_table("subcategories");
    let mut query_columns_joined = QueryColumns::new(false, None)?;

    let column1 = main_table.get_column("subcategory_id");
    let column2 = joined_table.get_column("subcategory");

    let query_column = QueryColumn::AsIs(&column2);
    query_columns_joined.add_query_column(query_column)?;


    let join_column = joined_table.get_column("id");
    let join_columns = vec![JoinColumn::new(Pair::new(&column1, &join_column), ConditionOperator::Equal, BindMethod::FirstCondition)];
    let join_table1 = JoinTable::new(&joined_table, &query_columns_joined, &join_columns, JoinType::Left);

    query_generator.add_join_table(&join_table1)?;

    let condition_column = main_table.get_column("work_time");
    let value = Variable::Decimal(Decimal::from_f32_retain(1.1).unwrap());
    let condition = Condition::new(&condition_column, &value, ConditionOperator::GreaterEq);
    query_generator.add_condition(&condition, BindMethod::FirstCondition)?;

    let sort_rule = SortRule::new(&column2, SortMethod::Asc);
    query_generator.add_sort_rule(&sort_rule)?;

    println!("{}", query_generator.get_statement());

    Ok(())
}

