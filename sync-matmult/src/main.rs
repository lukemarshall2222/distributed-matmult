use std::error::Error;

type Matrix = Vec<Vec<i32>>; // type alias

pub fn matmult(left: &Matrix, right: &Matrix) -> Result<Matrix, Box<dyn Error>> {
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

    // multiply the matrices synchronously, one dot product at a time:
    for i in 0..num_rows_left {
        for j in 0..num_cols_right {
            for k in 0..num_cols_left {
                result[i][j] += left[i][k] * right[k][j];
            }
        }
    }

    Ok(result) // Returns the result wrapped in Ok to denote it was successful to the caller
}

fn main() {
    let left: Matrix = vec![vec![1, 2], vec![3, 4]]; // first matrix
    let right: Matrix = vec![vec![5, 6], vec![7, 8]]; // second matrix

    // unwrap returns the value of a Result if it is a value, 
    // otherwise panics with the error
    let result: Matrix = matmult(&left, &right).unwrap();
    let expected: Matrix = vec![
        vec![19, 22], // [1*5 + 2*7, 1*6 + 2*8]
        vec![43, 50], // [3*5 + 4*7, 3*6 + 4*8]
    ]; // expected result from the matrix multiplication

    println!("result:   {:?}\nexpected: {:?}", result, expected);
}
