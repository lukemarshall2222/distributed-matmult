use serde::{Deserialize, Serialize};
use std::error::Error;

pub type Matrix = Vec<Vec<i32>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct DotProductPayload {
    pub row: Vec<i32>,
    pub col: Vec<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct DotProductResponse {
    pub result: i32,
}

#[derive(Deserialize, Debug)]
pub struct MatMultRequest {
    pub left: Matrix,
    pub right: Matrix,
}

// Custom error type for our application to simplify error handling in Warp
#[derive(Debug)]
pub struct AppError(pub String);
// Custom error type for the worker (optional, but good practice)
#[derive(Debug)]
pub struct WorkerError(pub String);

impl warp::reject::Reject for AppError {}
impl warp::reject::Reject for WorkerError {}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError(s)
    }
}

impl From<Box<dyn Error + Send + Sync>> for AppError {
    fn from(err: Box<dyn Error + Send + Sync>) -> Self {
        AppError(err.to_string())
    }
}

