use hydro_lang::*;

pub type Matrix = Vec<Vec<i32>>;

pub fn sync_matmult(process: &Process) {
    process
        .source_iter(q!(0..1))
        .for_each(q!(|_i: i32| {
            let left: Matrix = vec![vec![1, 2, 3], vec![4, 5, 6]];
            let right: Matrix = vec![vec![7, 8], vec![9, 10], vec![11, 12]];
            
            let multiply_result = || -> Result<Matrix, String> {
                let num_rows_left = left.len();
                if num_rows_left == 0 {
                    let num_rows_right = right.len();
                    let num_cols_right = if num_rows_right > 0 { right[0].len() } else { 0 };
                    return Ok(vec![vec![0; num_cols_right]; 0]);
                }
                let num_cols_left = left[0].len();

                let num_rows_right = right.len();
                if num_rows_right == 0 {
                    if num_cols_left != 0 {
                        return Err(format!(
                            "Matrices are incompatible: A is {}x{} but B has 0 rows (requires {} rows).",
                            num_rows_left, num_cols_left, num_cols_left
                        ));
                    }
                    return Ok(vec![vec![0; 0]; num_rows_left]);
                }
                let num_cols_right = right[0].len();

                if num_cols_left != num_rows_right {
                    return Err(format!(
                        "Matrices are incompatible for multiplication: cols_A ({}) != rows_B ({}).",
                        num_cols_left, num_rows_right
                    ));
                }

                if !left.iter().all(|row| row.len() == num_cols_left) {
                    return Err("Matrix A is not rectangular.".to_string());
                }
                if !right.iter().all(|row| row.len() == num_cols_right) {
                    return Err("Matrix B is not rectangular.".to_string());
                }

                if num_cols_left == 0 {
                    return Ok(vec![vec![0; num_cols_right]; num_rows_left]);
                }

                let mut result: Matrix = vec![vec![0; num_cols_right]; num_rows_left];

                for i in 0..num_rows_left {
                    for j in 0..num_cols_right {
                        let mut sum = 0;
                        for k in 0..num_cols_left {
                            sum += left[i][k] * right[k][j];
                        }
                        result[i][j] = sum;
                    }
                }
                Ok(result)
            };
            
            match multiply_result() {
                Ok(result_matrix) => {
                    for row in &result_matrix {
                        println!("{:?}", row);
                    }
                }
                Err(e) => {
                    eprintln!("Error during matrix multiplication: {}", e);
                }
            }
        }));
}