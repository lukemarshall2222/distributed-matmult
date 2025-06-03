use hydro_lang::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MatrixCellTask {
    target_row: i32,
    target_col: i32,
    row_vec: Vec<i32>,
    col_vec: Vec<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MatrixCellValue {
    row_ind: i32,
    col_ind: i32,
    value: i32,
}

pub struct Leader {}
pub struct Worker {}

pub type Matrix = Vec<Vec<i32>>;

pub fn cluster_matmult<'a>(
    leader: &Process<'a, Leader>,
    workers: &Cluster<'a, Worker>,
) {
    let result_rows: i32 = 2;
    let result_cols: i32 = 2;

    leader
        .source_iter(q!({
            let left_matrix_base: Matrix = vec![vec![1, 2, 3], vec![4, 5, 6]];
            let right_matrix_base: Matrix = vec![vec![7, 8], vec![9, 10], vec![11, 12]];

            (0..result_rows).flat_map(move |row_ind| {
                let left_matrix_for_r = left_matrix_base.clone();
                let right_matrix_for_r = right_matrix_base.clone();

                (0..result_cols).map(move |col_ind| {
                    let left_matrix = left_matrix_for_r.clone();
                    let right_matrix = right_matrix_for_r.clone();

                    let row_ind = row_ind as usize;
                    let row_vec = if row_ind < left_matrix.len() {
                        left_matrix[row_ind].clone()
                    } else {
                        vec![]
                    };

                    let col_ind = col_ind as usize;
                    let mut col_vec = Vec::new();
                    for r in 0..right_matrix.len() {
                        if col_ind < right_matrix[r].len() {
                            col_vec.push(right_matrix[r][col_ind]);
                        } else {
                            panic!("Column index out of bounds for right matrix.");
                        }
                    }

                    MatrixCellTask {
                        target_row: row_ind as i32,
                        target_col: col_ind as i32,
                        row_vec,
                        col_vec,
                    }
                })
            })
        }))
        .round_robin_bincode(workers)
        .map(q!(|task: MatrixCellTask| {
            let row_vec = task.row_vec;
            let col_vec = task.col_vec;

            let mut sum = 0;
            if row_vec.len() == col_vec.len() {
                for k in 0..row_vec.len() {
                    sum += row_vec[k] * col_vec[k];
                }
            } else {
                eprintln!(
                    "Error: Vector lengths mismatch for cell C[{}, {}] (row: {}, col: {})",
                    task.target_row, task.target_col, row_vec.len(), col_vec.len()
                );
            }

            MatrixCellValue {
                row_ind: task.target_row,
                col_ind: task.target_col,
                value: sum,
            }
        }))
        .send_bincode_anonymous(leader)
        .for_each(q!(|cell_value: MatrixCellValue| {
            println!(
                "Matrix Cell Result: C[{}, {}] = {}",
                cell_value.row_ind, cell_value.col_ind, cell_value.value
            );
        }));
}