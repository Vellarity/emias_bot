use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BasicRequest<T> {
    pub id: Option<String>,
    pub jsonrpc: String,
    pub method: String,
    pub params: T
}
