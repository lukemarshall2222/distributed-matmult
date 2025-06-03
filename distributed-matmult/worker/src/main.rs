use std::convert::Infallible; // For infallible handlers
use warp::{Filter, Rejection, Reply};
use warp::http::StatusCode;

mod types;
use types::{DotProductPayload, DotProductResponse, WorkerError};
use std::env;

async fn calculate_dot_product_handler(
    payload: DotProductPayload,
) -> Result<impl Reply, Rejection> {
    if payload.row.len() != payload.col.len() {
        eprintln!(
            "Worker error: Row length ({}) does not match column length ({}).",
            payload.row.len(),
            payload.col.len()
        );
        return Err(warp::reject::custom(WorkerError(
            "Row and column vectors must have the same length.".to_string(),
        )));
    }

    let mut total: i32 = 0;
    for i in 0..payload.row.len() {
        total = total.saturating_add(payload.row[i].saturating_mul(payload.col[i]));
    }

    let response = DotProductResponse { result: total };

    Ok(warp::reply::json(&response))
}

async fn handle_worker_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_string();
    } else if let Some(worker_err) = err.find::<WorkerError>() {
        code = StatusCode::BAD_REQUEST; // Or INTERNAL_SERVER_ERROR depending on error type
        message = worker_err.0.clone();
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED".to_string();
    } else if err.find::<warp::reject::UnsupportedMediaType>().is_some() {
        code = StatusCode::UNSUPPORTED_MEDIA_TYPE;
        message = "UNSUPPORTED_MEDIA_TYPE".to_string();
    }
     else if err.find::<warp::body::BodyDeserializeError>().is_some() {
        code = StatusCode::BAD_REQUEST;
        message = "INVALID_REQUEST_BODY".to_string();
    }
    else {
        eprintln!("Unhandled worker rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION".to_string();
    }

    let json = warp::reply::json(&serde_json::json!({
        "error": message,
    }));

    Ok(warp::reply::with_status(json, code))
}

#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-Type"])
        .allow_methods(vec!["POST", "OPTIONS"]);

    let dot_product_route = warp::post()
        .and(warp::path("calculate_dot_product"))
        .and(warp::body::json())
        .and_then(calculate_dot_product_handler);

    let routes = dot_product_route
        .with(cors)
        .recover(handle_worker_rejection);

    let port_str = env::var("WORKER_PORT").unwrap_or_else(|_| "9001".to_string());
    let port: u16 = port_str.parse().expect("WORKER_PORT must be a valid port number");

    println!(
        "Worker node server running on http://0.0.0.0:{}/calculate_dot_product", // Listen on all interfaces
        port
    );
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
