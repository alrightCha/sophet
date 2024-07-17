use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StoreRequest {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct RetrieveRequest {
    pub key: String,
}
