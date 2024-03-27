use serde::{Serialize, Deserialize};
use serde_json::{Map, Value, Error as JSONError};
use tokio_postgres::Row;
use crate::legacies::converter::row_to_value;

#[derive(Serialize, Deserialize)]
struct GetData {
    data: Vec<Map<String, Value>>
}

pub(super) fn row_to_json(query_result: &Vec<Row>) -> Result<String, JSONError> {
    let mut data: Vec<Map<String, Value>> = Vec::new();
    let columns: Vec<String> =
        query_result[0].columns().iter().map(
            |column| column.name().to_string()
        ).collect();

    for row in query_result {
        let mut row_data: Map<String, Value> = Map::new();
        for column in &columns {
            row_data.insert(column.to_string(), row_to_value(row, column));
        }
        data.push(row_data);
    }

    let get_data = GetData {
        data
    };
    serde_json::to_string(&get_data)
}

fn execution_from_json(json: &str) {

}
