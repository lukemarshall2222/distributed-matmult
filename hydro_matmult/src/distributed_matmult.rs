use hydro_lang::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MatrixCellTask {
    target_row_i32: i32,
    target_col_i32: i32,
    row_vec: Vec<i32>,
    col_vec: Vec<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MatrixCellValue {
    row_i32: i32,
    col_i32: i32,
    value: i32,
}


pub type Matrix = Vec<Vec<i32>>;

pub struct P1 {}
pub struct P2 {}

pub fn distributed_matmult<'a>(p1: &Process<'a, P1>, p2: &Process<'a, P2>) {
    let result_rows_i32: i32 = 2;
    let result_cols_i32: i32 = 2;

    p1.source_iter(q!({
        let left_matrix_base: Matrix = vec![vec![1, 2, 3], vec![4, 5, 6]]; // 2x3
        let right_matrix_base: Matrix = vec![vec![7, 8], vec![9, 10], vec![11, 12]]; // 3x2

        (0..result_rows_i32).flat_map(move |r_ind_i32| {
            let left_matrix_for_r = left_matrix_base.clone();
            let right_matrix_for_r = right_matrix_base.clone();

            (0..result_cols_i32).map(move |c_ind_i32| {
                let left_matrix = left_matrix_for_r.clone();
                let right_matrix = right_matrix_for_r.clone();

                let row_ind = r_ind_i32 as usize;
                let row_vec = if row_ind < left_matrix.len() {
                    left_matrix[row_ind].clone()
                } else {
                    vec![]
                };

                let col_ind = c_ind_i32 as usize;
                let mut col_vec = Vec::new();
                for r in 0..right_matrix.len() {
                    if col_ind < right_matrix[r].len() {
                        col_vec.push(right_matrix[r][col_ind]);
                    } else {
                        panic!("Column index out of bounds for right matrix.");
                    }
                }

                MatrixCellTask {
                    target_row_i32: r_ind_i32,
                    target_col_i32: c_ind_i32,
                    row_vec,
                    col_vec,
                }
            })
        })
    }))
    .send_bincode(p2)
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
                task.target_row_i32, task.target_col_i32, row_vec.len(), col_vec.len()
            );
        }

        MatrixCellValue {
            row_i32: task.target_row_i32,
            col_i32: task.target_col_i32,
            value: sum,
        }
    }))
    .send_bincode(p1)
    .for_each(q!(|cell_value: MatrixCellValue| {
        println!(
            "Computed Matrix Cell Result: C[{}, {}] = {}",
            cell_value.row_i32, cell_value.col_i32, cell_value.value
        );
    }));
}