use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryableData {
    pub average: f64,
    pub nb_values: u128,
}

#[allow(dead_code)]
impl QueryableData {
    pub fn new(average: f64, nb_values: u128) -> Self {
        QueryableData { average, nb_values }
    }
}

#[allow(dead_code)]
fn main() {
    ()
}