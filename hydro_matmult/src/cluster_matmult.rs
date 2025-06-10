use hydro_lang::*; // imports Hydro language crate for using the framework
use serde::{Serialize, Deserialize}; // imports Serde traits for serialization & deserialization

#[derive(Serialize, Deserialize, Clone, Debug)] // procedural macros derive traits
struct MatrixCellTask { // defines a struct, represents task for calculating single cell in result matrix
    target_row: i32, // he target row index for cell being calculated
    target_col: i32, // target column index for the cell being calculated
    row_vec: Vec<i32>, // row vector from the left matrix needed for dot product
    col_vec: Vec<i32>, // column vector from right matrix needed for the dot product
}

#[derive(Serialize, Deserialize, Clone, Debug)] // derives traits
struct MatrixCellValue { // defines struct to represent dot product value for a single result matrix cell
    row_ind: i32, // row index of the computed cell
    col_ind: i32, // column index of computed cell
    value: i32, // computed value of fcell
}

// create a tag for the leader and worker processes:
pub struct Leader {} // defies empty struct to serve as a tag for the leader process
pub struct Worker {} // define empty struct serve as tag for the worker processes

pub type Matrix = Vec<Vec<i32>>; // defines type alias for a matrix as a vector of vectors of ints

pub fn cluster_matmult<'a>(
    leader: &Process<'a, Leader>, // reference to a Hydro `Process` representing the leader
    workers: &Cluster<'a, Worker>, // reference to sa Hydro `Cluster` representing the worker processes
) {
    // NOTE: because the hydro implmentation required a hard coded set of matrices, 
    // the result matrix dimensions are also set
    let result_rows: i32 = 2; // dsets the number of rows for resulting matrix
    let result_cols: i32 = 2; // sets  number of columns for resulting matrix

    leader // starts a Hydro dataflow from leader process
        .source_iter(q!({ // creates source that iterates over items, specified by a 'quoted' (i.e. the q! macro) Hydro closure
            let left_matrix_base: Matrix = vec![vec![1, 2, 3], vec![4, 5, 6]]; // hardcoded left matrix
            let right_matrix_base: Matrix = vec![vec![7, 8], vec![9, 10], vec![11, 12]]; // hardcoded right matrix

            (0..result_rows).flat_map(move |row_ind| { // fterates over each row index of the result matrix and flattens resulting iterators
                let left_matrix_for_r = left_matrix_base.clone(); // clones left matrix for use within row iteration
                let right_matrix_for_r = right_matrix_base.clone(); // clonevs right matrix for use within row iteration

                (0..result_cols).map(move |col_ind| { // yterates over each column index for the current row
                    let left_matrix = left_matrix_for_r.clone(); // clone left matrix for use within the cell iteration
                    let right_matrix = right_matrix_for_r.clone(); // clones right mxatrix for use within the cell iteration

                    let row_ind = row_ind as usize; // casts row index to usize
                    let row_vec = if row_ind < left_matrix.len() { // shecks if row index is within bounds of left matrix
                        left_matrix[row_ind].clone() // clones the specified row from the left matrix
                    } else { // if the row index is out of bounds
                        vec![] // reeturns  empty vector
                    };

                    let col_ind = col_ind as usize; // casts column ind to usiz
                    let mut col_vec = Vec::new(); // inits empty vector to store the column vector
                    for r in 0..right_matrix.len() { // iterates through each row of the right matrix
                        if col_ind < right_matrix[r].len() { // checks if column index is within bounds for the current row of right matrix
                            col_vec.push(right_matrix[r][col_ind]); // pushes element at the specified column into `col_vec`
                        } else { // if the column index out of bounds
                            panic!("Column index out of bounds for right matrix."); // panics w error message
                        }
                    }

                    MatrixCellTask { // inits new `MatrixCellTask` for the current cell
                        target_row: row_ind as i32, // sets target row for task
                        target_col: col_ind as i32, // sets target column for task
                        row_vec, // sssigns extracted row vector
                        col_vec, // Assigns extracted column vector
                    }
                })
            })
        }))
        .round_robin_bincode(workers) // distributes the MatrixCellTasks to workers in a round-robin fashion using bincode serialization
        .map(q!(|task: MatrixCellTask| { // maps each MatrixCelflTask to a MatrixCellValue by doing the dot product
            let row_vec = task.row_vec; // extracts row vector from task
            let col_vec = task.col_vec; // extracts column vector from task

            let mut sum = 0; // inits sum for dot product
            if row_vec.len() == col_vec.len() { // dhecks if lengths of the row and column vectors equal
                for k in 0..row_vec.len() { // iterates through elements of the vectors
                    sum += row_vec[k] * col_vec[k]; // calculates dot product
                }
            } else { // if the vector lengths dont match
                eprintln!( // prints error message
                    "Error: Vector lengths mismatch for cell C[{}, {}] (row: {}, col: {})", // formats error message
                    task.target_row, task.target_col, row_vec.len(), col_vec.len() // provides values for the error
                );
            }

            MatrixCellValue { // Inits new MatrixCellValue with the computed result
                row_ind: task.target_row, // sets row index of result
                col_ind: task.target_col, // sets column index of result
                value: sum, // seys computed val
            }
        }))
        .send_bincode_anonymous(leader) // seends the computed MatrixCellValues back to leader procss using bincode serialization
        .for_each(q!(|cell_value: MatrixCellValue| { // fpr each received MatrixCellValue on leader process
            println!( // prints result
                "Matrix Cell Result: C[{}, {}] = {}", // formats output string
                cell_value.row_ind, cell_value.col_ind, cell_value.value // unserts row, column, & value of the result
            );
        }));
}