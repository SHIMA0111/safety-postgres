use std::fmt::Display;

#[derive(Clone)]
pub struct Pair<F> {
    value1: F,
    value2: F,
}

impl<F: Clone + ?Sized> Pair<F> {
    pub fn new(value1: F, value2: F) -> Self {
        Pair {
            value1,
            value2
        }
    }

    pub fn get_values(&self) -> (&F, &F) {
        (&self.value1, &self.value2)
    }

    pub fn get_first(&self) -> &F {
        &self.value1
    }

    pub fn get_second(&self) -> &F {
        &self.value2
    }
}

pub(crate) fn check_aggregation(column_name: String) -> bool {
    let aggregations = ["AVG", "COUNT", "SUM", "MIN", "MAX"];
    if column_name.contains("(") && column_name.contains(")") {
        let aggregation_name = column_name.split("(").collect::<Vec<&str>>()[0];
        if !aggregations.contains(&aggregation_name) {
            return false
        }
    }
    else {
        return false
    }

    true
}
