#[cfg(test)]
mod tests_curd {
    use std::str::FromStr;
    use chrono::NaiveDate;
    use testcontainers::clients::Cli;
    use testcontainers::core::WaitFor;
    use testcontainers::{Container, GenericImage};
    use tokio::fs;
    use tokio_postgres::NoTls;
    use safety_postgres::access::conditions::{Conditions, IsInJoinedTable};
    use safety_postgres::access::conditions::IsInJoinedTable::No;
    use safety_postgres::access::join_tables::JoinTables;
    use safety_postgres::access::postgres_base::PostgresBase;
    use safety_postgres::access::sql_base::{InsertRecords, QueryColumns, UpdateSets};

    const DB_USER: &str = "testuser";
    const DB_PASSWORD: &str = "testpassword";
    const DB_HOST: &str = "localhost";
    const DB_NAME: &str = "test";

    async fn test_data_creation(docker: &Cli) -> Result<Container<GenericImage>, Box<dyn std::error::Error>> {
        let image = GenericImage::new("postgres", "16.2-alpine3.19")
            .with_wait_for(WaitFor::message_on_stderr("ready to accept connections"))
            .with_env_var("POSTGRES_USER", DB_USER)
            .with_env_var("POSTGRES_PASSWORD", DB_PASSWORD)
            .with_env_var("POSTGRES_DB", DB_NAME);

        let node: Container<GenericImage> = docker.run(image);

        let (client, connection) = tokio_postgres::Config::new()
            .user(DB_USER)
            .password(DB_PASSWORD)
            .host(DB_HOST)
            .dbname(DB_NAME)
            .port(node.get_host_port_ipv4(5432))
            .connect(NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        let sql = fs::read_to_string(
            "./tests/sql/init.sql"
        ).await?;

        client.batch_execute(&sql).await?;
        Ok(node)
    }

    fn set_env(port: u16) {
        std::env::set_var("DB_USER", DB_USER);
        std::env::set_var("DB_PASSWORD", DB_PASSWORD);
        std::env::set_var("DB_HOST", DB_HOST);
        std::env::set_var("DB_PORT", port.to_string());
        std::env::set_var("DB_NAME",DB_NAME);
    }

    #[tokio::test]
    async fn test_connection() {
        let docker = Cli::default();
        let node = test_data_creation(&docker).await.unwrap();

        let port = node.get_host_port_ipv4(5432);

        set_env(port);
        let mut postgres = PostgresBase::new("table").unwrap();
        postgres.connect().await.unwrap();
        match postgres.connect().await {
            Ok(_) => {},
            Err(e) => panic!("connection failed due to: {}", e),
        }
    }

    #[tokio::test]
    async fn test_select() {
        let docker = Cli::default();
        let node = test_data_creation(&docker).await.unwrap();

        let port  = node.get_host_port_ipv4(5432);

        set_env(port);

        let mut postgres = PostgresBase::new("users").unwrap();
        postgres.set_schema("test_schema");
        postgres.connect().await.unwrap();

        let query_columns = QueryColumns::new(true);
        let query_result = postgres.query_raw(query_columns).await.unwrap();

        assert_eq!(query_result.len(), 4);
        assert_eq!(query_result[0].len(), 10);

        let mut query_columns = QueryColumns::new(false);
        query_columns.add_column("", "", "id").unwrap();
        query_columns.add_column("", "", "username").unwrap();
        let mut conditions = Conditions::new();
        conditions.add_condition_from_str("id", "2", "lt", "", IsInJoinedTable::No).unwrap();
        let query_result = postgres.query_condition_raw(query_columns, conditions).await.unwrap();

        assert_eq!(query_result.len(), 1);
        assert_eq!(query_result[0].len(), 2);

        let query_columns = QueryColumns::new(true);
        let mut joined_tables = JoinTables::new();
        joined_tables.add_join_table("test_schema", "records", &["user_id"], &["id"]).unwrap();

        let mut conditions = Conditions::new();
        conditions.add_condition_from_str(
            "id", "2", "le", "", IsInJoinedTable::Yes {
                schema_name: "".to_string(), table_name: "users".to_string()
            }
        ).unwrap();

        let query_result = postgres.query_inner_join_conditions(
            query_columns, joined_tables, conditions
        ).await.unwrap();

        assert_eq!(query_result.len(), 10);
        assert_eq!(query_result[0].len(), 16);
    }

    #[tokio::test]
    async fn test_insert() {
        let docker = Cli::default();
        let node = test_data_creation(&docker).await.unwrap();

        let port  = node.get_host_port_ipv4(5432);

        set_env(port);

        let mut postgres = PostgresBase::new("records").unwrap();
        postgres.set_schema("test_schema");
        postgres.connect().await.unwrap();

        let query_columns = QueryColumns::new(true);
        let base_records = postgres.query_raw(query_columns).await.unwrap();

        assert_eq!(base_records.len(), 16);

        let record_columns = vec!["user_id", "record_date", "subcategory_id", "work_time", "message_comment"];
        let mut insert_records = InsertRecords::new(&record_columns);
        let record = vec!["2", "2023-09-28", "2", "3.8", "inserted_record"];
        insert_records.add_record(&record).unwrap();

        let _ = postgres.insert(insert_records).await.unwrap();

        let query_columns = QueryColumns::new(true);
        let after_query = postgres.query_raw(query_columns);

        let query_columns = QueryColumns::new(true);
        let mut conditions = Conditions::new();
        conditions.add_condition_from_str("id", "17", "eq", "", No).unwrap();
        let inserted_query = postgres.query_condition_raw(query_columns, conditions);

        let (after_records, inserted_records) = futures::join!(after_query, inserted_query);

        assert_eq!(after_records.unwrap().len(), 17);

        let conditioned_records = inserted_records.unwrap();
        assert_eq!(conditioned_records.len(), 1);
        let record_one = conditioned_records.get(0).unwrap();

        let (rid, uid, rec_date, subcat_id, work_time, message) = {
            let rid: i32 = record_one.get("id");
            let uid: i32 = record_one.get("user_id");
            let rec_date: NaiveDate = record_one.get("record_date");
            let subcat_id: i32 = record_one.get("subcategory_id");
            let work_time: f32 = record_one.get("work_time");
            let message: &str = record_one.get("message_comment");

            (rid, uid, rec_date, subcat_id, work_time, message)
        };

        assert_eq!(rid, 17);
        assert_eq!(uid, 2);
        assert_eq!(rec_date, NaiveDate::from_str("2023-09-28").unwrap());
        assert_eq!(subcat_id, 2);
        assert_eq!(work_time, 3.8);
        assert_eq!(message, "inserted_record");
    }

    #[tokio::test]
    async fn test_update() {
        let docker = Cli::default();
        let node = test_data_creation(&docker).await.unwrap();

        let port  = node.get_host_port_ipv4(5432);

        set_env(port);

        let mut postgres = PostgresBase::new("records").unwrap();
        postgres.set_schema("test_schema");
        postgres.connect().await.unwrap();

        let mut query_column = QueryColumns::new(false);
        query_column.add_column("", "", "work_time").unwrap();
        let mut conditions = Conditions::new();
        conditions.add_condition_from_str("id", "1", "eq", "", No).unwrap();

        let original_records = postgres.query_condition_raw(query_column, conditions).await.unwrap();
        let origin_record = &original_records[0];

        let origin_work_time: f32 = origin_record.get("work_time");
        assert_eq!(origin_work_time, 4.5);

        let mut update_set = UpdateSets::new();
        update_set.add_set("work_time", "2.9").unwrap();
        let mut conditions = Conditions::new();
        conditions.add_condition_from_str("id", "1", "=", "", No).unwrap();

        postgres.update_condition(update_set, conditions).await.unwrap();

        let mut query_column = QueryColumns::new(false);
        query_column.add_column("", "", "work_time").unwrap();

        let mut conditions = Conditions::new();
        conditions.add_condition_from_str("id", "1", "eq", "", No).unwrap();

        let updated_records = postgres.query_condition_raw(query_column, conditions).await.unwrap();
        assert_eq!(updated_records.len(), 1);

        let updated_record = &updated_records[0];

        let updated_work_time: f32 = updated_record.get("work_time");

        assert_eq!(updated_work_time, 2.9);
    }

    #[tokio::test]
    async fn test_delete() {
        let docker = Cli::default();
        let node = test_data_creation(&docker).await.unwrap();

        let port  = node.get_host_port_ipv4(5432);

        set_env(port);

        let mut postgres = PostgresBase::new("records").unwrap();
        postgres.set_schema("test_schema");
        postgres.connect().await.unwrap();

        let original_records = postgres.query_raw(
            QueryColumns::new(true)
        ).await.unwrap();

        assert_eq!(original_records.len(), 16);

        let mut condition = Conditions::new();
        condition.add_condition_from_str("user_id", "1", "eq", "", No).unwrap();
        let _ = postgres.delete(condition).await.unwrap();

        let updated_records = postgres.query_raw(
            QueryColumns::new(true)
        ).await.unwrap();

        assert_eq!(updated_records.len(), 11);
    }
}