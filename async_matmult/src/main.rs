use tokio::task;
use std::{error::Error, sync::Arc};

type Matrix = Vec<Vec<i32>>;

pub async fn async_matmult(left: Matrix, right: Matrix) -> Result<Matrix, Box<dyn Error>> {
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

    // wraping the matrices in Arcs makes them atomically refeerence counted and able to be
    // shared between threads
    let left: Arc<Matrix> = Arc::new(left);
    let right: Arc<Matrix> = Arc::new(right);
    
    // Create tasks for each dot product (each element of the result matrix)
    // Each task will return a tuple: (row_index, col_index, calculated_value)
    let mut tasks: Vec<task::JoinHandle<(usize, usize, i32)>> = Vec::new();
    
    // Iterate over each cell of the target matrix
    (0..num_rows_left).for_each(|i: usize| { // i is the row index for the result matrix
        (0..num_cols_right).for_each(|j: usize| { // j is the column index for the result matrix
            // create new references to the matrices for this specific dot product task:
            let left_clone: Arc<Matrix> = Arc::clone(&left);
            let right_clone: Arc<Matrix> = Arc::clone(&right
    );
            
            // num_cols_left captured from outer scope, 
            // variable determines the length of dot product sum

            // spawn new task to calculate dot product for result[i][j]:
            let task: task::JoinHandle<(usize, usize, i32)> = task::spawn(async move {
                // accumulator for the sot product:
                let mut sum: i32 = 0;
                // dot product: sum(left[i][k] * right[k][j]) for k:
                (0..num_cols_left).for_each(|k: usize| {
                    sum += left_clone[i][k] * right_clone[k][j];
                });
                // return calculated value with its coordinates in result:
                (i, j, sum)
            });
            
            // add the task to the vector of tasks, then move on to creating the next task
            tasks.push(task);
        });
    });
    
    // wait for all dot product tasks to complete and populate the result matrix with the individual results, 
    // given the coordinates
    for task_handle in tasks {
        // task_handle.await returns Result<(usize, usize, i32), JoinError>
        let (i, j, value) = task_handle.await.map_err(|e: task::JoinError| format!("Task failed: {}", e))?;
        result[i][j] = value;
    }
    
    Ok(result)
}

#[tokio::main]
async fn main() {
    let left_matrix: Matrix = vec![vec![1, 2], vec![3, 4]]; // first matrix
    let right_matrix: Matrix = vec![vec![5, 6], vec![7, 8]]; // second matrix

    // unwrap returns the value of a Result if it is a value, 
    // otherwise panics with the error
    let result_matrix: Matrix = async_matmult(left_matrix, right_matrix).await.unwrap();
    let expected_matrix: Matrix = vec![
        vec![19, 22], // [1*5 + 2*7, 1*6 + 2*8]
        vec![43, 50], // [3*5 + 4*7, 3*6 + 4*8]
    ]; // expected result from the matrix multiplication

    println!("result:   {:?}\nexpected: {:?}", result_matrix, expected_matrix);
}