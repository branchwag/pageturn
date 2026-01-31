use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: u32,
    pub title: String,
    pub author: String,
    pub date: String,
    pub content: String,
    pub summary: String,
}
