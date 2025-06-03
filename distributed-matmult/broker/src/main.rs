use std::error::Error;
use std::sync::Arc; // For sharing worker list and client across threads
use std::sync::atomic::{AtomicUsize, Ordering}; // For round-robin load balancing

use futures::future::join_all;
use reqwest::Client;
use tokio::task;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

mod types;
use types::{
    AppError, DotProductPayload, DotProductResponse, MatMultRequest, Matrix,
};

// local endpoints to send matrix multiplication dot products to:
const WORKER_NODES: &[&str] = &[
    "http://worker1:9001/calculate_dot_product",
    "http://worker2:9002/calculate_dot_product",
];


async fn distribute_mat_mult(
    left: &Matrix,
    right: &Matrix,
    http_client: Arc<Client>,
    worker_urls: Arc<Vec<String>>,
    next_worker_index: Arc<AtomicUsize>,
) -> Result<Matrix, Box<dyn Error + Send + Sync>> { // Send and Sync traits are what allow this to be multithreaded
    // ensure matrices are populated:
    if left.is_empty() || right.is_empty() {
        return Err("At least one of the given matrices was empty".into());
    }

    // extract the dimensions of the matrices to ensure they are mathematically compatible:
    let num_rows_left = left.len();
    let num_cols_left = left[0].len();
    let num_rows_right = right.len();
    let num_cols_right = right[0].len();

    // check matrices are mathematically compatitible:
    if num_cols_left != num_rows_right {
        return Err(format!(
            "Unable multiply {}x{} matrix by a {}x{} matrix, number of columns on left should
            equal the number of rows on the right",
            num_rows_left, num_cols_left, num_rows_right, num_cols_right
        )
        .into());
    }

    // init result matrix and populate w all 0s:
    let mut result = vec![vec![0; num_cols_right]; num_rows_left];

    // init an empty vector of http request tasks:
    let mut http_call_tasks = Vec::new();

    // check there is at one worker
    if worker_urls.is_empty() {
        return Err("Need worker node URLs".into());
    }

    (0..num_rows_left).for_each(|i: usize| {
        (0..num_cols_right).for_each(|j: usize| {
            let row_of_left: Vec<i32> = left[i].clone();
            let col_of_right: Vec<i32> = (0..num_rows_right).map(|k| right[k][j]).collect();

            // round robin selection of worker node:
            let chosen_worker_id = next_worker_index.fetch_add(1, Ordering::Relaxed) % worker_urls.len();
            let chosen_worker_url = worker_urls[chosen_worker_id].clone();

            // clone the reference to the client which is used to send the dot product task
            let client_clone = Arc::clone(&http_client);

            // spawning a task creates a new thread
            let task_handle = task::spawn(async move {
                // create the request payload out of the given row and colum 
                // of the left and righ matrices
                let payload = DotProductPayload {
                    row: row_of_left,
                    col: col_of_right,
                };

                // use the client to send the request to the chosen worker:
                let response_result = client_clone
                    .post(&chosen_worker_url) // POST request
                    .json(&payload) // data serialized to JSON before being sent
                    .send() // request is sent
                    .await; // task waits for the response

                // Example of error propogation in rust, the server responds with a result type
                // if the result is ok, and the response stantus is success, we extract the reposnse
                // values by deserializing the response from JSON, again checking this is successful, 
                // and then finally packing the unpacked values into an Ok to be used in the matrix
                // If any of those checks fail, we report the error, adding on a little context and
                // propogating the original error back to the caller so they can make an informed
                // decision on how to handle it
                match response_result {
                    Ok(response) => { // first result is ok, so a value is unpacked
                        if response.status().is_success() { // check the response stat
                            match response.json::<DotProductResponse>().await {
                                // check if parsing of respsponse JSON payload worked
                                Ok(data) => Ok((i, j, data.result)), // it worked, upack the value
                                Err(e) => Err(format!( // didnt work, propogate back to caller
                                    "Failed to parse worker response for ({}, {}) from {}: {}",
                                    i, j, chosen_worker_url, e
                                )),
                            }
                        } else { // status is not success
                            let status = response.status();
                            let error_body =
                                response.text().await.unwrap_or_else(|_| "N/A".to_string());
                            Err(format!( // propogate the error back to the caller, informed by the 
                                // response of the server
                                "Worker error for ({}, {}) from {} - Status: {}, Body: {}",
                                i, j, chosen_worker_url, status, error_body
                            ))
                        }
                    }
                    Err(e) => Err(format!( // first result is error, propogate back to caller
                        "HTTP request to worker {} failed for ({}, {}): {}",
                        chosen_worker_url, i, j, e
                    )),
                }
            });
            // all the http requests can be completed concurrently, and are held in 
            // a vector of tasks 
            http_call_tasks.push(task_handle);
        });
    });

    // wait for all the http request results, and create a new vector out of those results
    let task_outcomes = join_all(http_call_tasks).await;

    // for each of the tasks, make sure the result found is the value and not an error,
    // otherwise proposgate it back up to the caller with more information
    for task_result in task_outcomes {
        match task_result { // match the result of the task itself
            Ok(http_call_outcome) => match http_call_outcome { // it worked, unpack the values from the work inside the task
                Ok((row_ind, col_ind, value)) => { // unpacks to another result value
                    result[row_ind][col_ind] = value; // set the value for the result matrix in the given coordinates
                }
                Err(err) => return Err(format!("worker sub-task failed: {}", err).into()), // unpacks to an error
                // propogate it to the caller with more info
            }, // result of the task itself is an error
            Err(err) => return Err(format!("spawned task failed: {}", err).into()),
        }
    }

    
    Ok(result) // warp the result in an Ok to tell the caller the thing was successful
}

/// Warp handler for the /multiply_matrices_distributed endpoint.
async fn matmult_handler(
    body: MatMultRequest,
    http_client: Arc<Client>,
    worker_urls: Arc<Vec<String>>,
    next_worker_index: Arc<AtomicUsize>,
) -> Result<impl Reply, Rejection> {

    // sends the given matrices 
    let answer = distribute_mat_mult(
        &body.left,
        &body.right,
        http_client, // reqwest client
        worker_urls,
        next_worker_index,
    )
    .await;

    match answer {
        Ok(result_matrix) => Ok(warp::reply::json(&result_matrix)),
        Err(e) => {
            eprintln!("Error during distributed multiplication: {}", e);
            // Convert Box<dyn Error> to our AppError for Warp's rejection system
            Err(warp::reject::custom(AppError(e.to_string())))
        }
    }
}

// Custom rejection handler to convert AppError into a proper HTTP response.
async fn rejection_handler(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(app_err) = err.find::<AppError>() {
        let json = warp::reply::json(&serde_json::json!({
            "error": app_err.0,
        }));
        Ok(warp::reply::with_status(json, StatusCode::BAD_REQUEST)) // Or INTERNAL_SERVER_ERROR
    } else {
        // For other rejections, like missing headers or method not allowed
        eprintln!("Unhandled rejection: {:?}", err);
        let json = warp::reply::json(&serde_json::json!({
            "error": format!("Unhandled error: {:?}", err),
        }));
        Ok(warp::reply::with_status(
            json,
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

#[tokio::main]
async fn main() {
    // init single reqwest client to be shared:
    let http_client = Arc::new(Client::new());

    // prepare worker URLs:
    let worker_urls_vec: Vec<String> = WORKER_NODES.iter().map(|s| s.to_string()).collect();
    let worker_urls_arc = Arc::new(worker_urls_vec);

    // Atomic counter for simple round-robin load balancing
    let next_worker_index = Arc::new(AtomicUsize::new(0));

    // CORS support needed to allow different origins to access the server
    // CORS configuration allows POST or OPTIONS from any origin with specified headers
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-Type"])
        .allow_methods(vec!["POST", "OPTIONS"]); // OPTIONS allows for preflight requests

    // have to clone Arcs for the filter closure
    let http_client_filter = warp::any().map(move || Arc::clone(&http_client));
    let worker_urls_filter = warp::any().map(move || Arc::clone(&worker_urls_arc));
    let next_worker_filter = warp::any().map(move || Arc::clone(&next_worker_index));

    // Define the route for matrix multiplication
    // POST /multiply_matrices_distributed
    let multiply_route = warp::post() // limit requests to POST
       .and(warp::path("multiply_matrices_distributed")) // matches URL path "/multiply_matrices_distributed"
       .and(warp::body::json()) // deserialize request body from JSON into expected type
       .and(http_client_filter) // inject reqwest client
       .and(worker_urls_filter) // inject worker node URLs
       .and(next_worker_filter) // inject next worker index
       .and_then(matmult_handler); // call handler function with all injected values

    // combine routes with CORS support and rejection handler
    let routes = multiply_route.with(cors).recover(rejection_handler);

    let port = 8000; // port for this server

    // init the server and wait for requests
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
