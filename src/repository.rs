#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

use chrono::{DateTime, Local};

use serde::Deserialize;

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct Repository {
    id: usize,
    pub name: String,
    pub description: Option<String>,
    pub archived: bool,
    pub updated_at: DateTime<Local>,
    pub pushed_at: DateTime<Local>,

    // #[serde(flatten)]
    // extras: HashMap<String, Value>,
}
