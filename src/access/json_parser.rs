use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use serde_json::{json, Map, Value, Error as JSONError};
use tokio_postgres::Row;

#[derive(Serialize, Deserialize)]
struct GetData {
    data: Vec<Map<String, Value>>
}

fn raw_convert_val(row: &Row, column: &String) -> Value {
    if let Ok(i) = row.try_get::<&str, i32>(column.as_str()) {
        json!(i)
    }
    else if let Ok(dec) = row.try_get::<&str, Decimal>(column.as_str()) {
        json!(dec)
    }
    else if let Ok(f) = row.try_get::<&str, f32>(column.as_str()) {
        json!(f)
    }
    else if let Ok(dt) = row.try_get::<&str, NaiveDateTime>(column.as_str()) {
        json!(dt.to_string())
    }
    else if let Ok(d) = row.try_get::<&str, NaiveDate>(column.as_str()) {
        json!(d.to_string())
    }
    else if let Ok(t) = row.try_get::<&str, NaiveTime>(column.as_str()) {
        json!(t.to_string())
    }
    else if let Ok(b) = row.try_get::<&str, bool>(column.as_str()) {
        json!(b)
    }
    else {
        json!(row.get::<&str, String>(column.as_str()))
    }
}

pub fn row_to_json(query_result: &Vec<Row>) -> Result<String, JSONError> {
    let mut data: Vec<Map<String, Value>> = Vec::new();
    let columns: Vec<String> =
        query_result[0].columns().iter().map(
            |column| column.name().to_string()
        ).collect();

    for row in query_result {
        let mut row_data: Map<String, Value> = Map::new();
        for column in &columns {
            row_data.insert(column.to_string(), raw_convert_val(row, column));
        }
        data.push(row_data);
    }

    let get_data = GetData {
        data
    };
    serde_json::to_string(&get_data)
}
