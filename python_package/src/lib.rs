use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
// use pyo3::PyTraverseError;
use pyo3::class::basic::PyObjectProtocol;
use std::cmp::{max, min};

use ms_toollib::*;
mod board;
pub use board::{PyAvfVideo, PyMinesweeperBoard};

// pip install maturin
// maturin publish --manylinux 2014

#[pyfunction]
#[pyo3(name = "refresh_matrix")]
fn py_refresh_matrix(
    board_of_game: Vec<Vec<i32>>,
) -> PyResult<(Vec<Vec<i32>>, Vec<(usize, usize)>, Vec<i32>)> {
    Ok(refresh_matrix(&board_of_game))
}

#[pyfunction]
#[pyo3(name = "refresh_matrixs")]
fn py_refresh_matrixs(
    board_of_game: Vec<Vec<i32>>,
) -> PyResult<(
    Vec<Vec<Vec<i32>>>,
    Vec<Vec<(usize, usize)>>,
    Vec<Vec<i32>>,
    usize,
    usize,
)> {
    Ok(refresh_matrixs(&board_of_game))
}

#[pyfunction]
#[pyo3(name = "cal_op")]
fn py_cal_op(board: Vec<Vec<i32>>) -> PyResult<usize> {
    Ok(cal_op(board))
}

#[pyfunction]
#[pyo3(name = "laymine_number")]
fn py_laymine_number(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
) -> PyResult<Vec<Vec<i32>>> {
    // 通用标准埋雷引擎
    // 输出为二维的局面
    Ok(laymine_number(row, column, mine_num, x0, y0))
}

#[pyfunction]
#[pyo3(name = "cal3BV")]
fn py_cal3BV(board: Vec<Vec<i32>>) -> PyResult<usize> {
    Ok(cal3BV(&board))
}

// #[pyfunction]
// fn solve_minus(
//     mut matrix_as: Vec<Vec<Vec<i32>>>,
//     mut matrix_xs: Vec<Vec<(usize, usize)>>,
//     mut matrix_bs: Vec<Vec<i32>>,
//     mut board_of_game: Vec<Vec<i32>>,
// ) -> PyResult<(Vec<Vec<Vec<i32>>>, Vec<Vec<(usize, usize)>>, Vec<Vec<i32>>, Vec<Vec<i32>>, Vec<(usize, usize)>, Vec<(usize, usize)>)> {
//     let (not_mine, is_mine) = ms_toollib_rs::solve_minus(&mut matrix_as, &mut matrix_xs, &mut matrix_bs, &mut board_of_game);
//     Ok((matrix_as, matrix_xs, matrix_bs, board_of_game, not_mine, is_mine))
// }

#[pyfunction]
#[pyo3(name = "refresh_board")]
fn py_refresh_board(
    board: Vec<Vec<i32>>,
    mut board_of_game: Vec<Vec<i32>>,
    ClickedPoses: Vec<(usize, usize)>,
) -> PyResult<Vec<Vec<i32>>> {
    refresh_board(&board, &mut board_of_game, ClickedPoses);
    Ok(board_of_game)
}

#[pyfunction]
#[pyo3(name = "get_all_not_mine_on_board")]
fn py_get_all_not_mine_on_board(game_board: Vec<Vec<i32>>) -> PyResult<Vec<(usize, usize)>> {
    Ok(get_all_not_mine_on_board(&game_board, 40))
}

// #[pyfunction]
// fn solve_direct(
//     mut MatrixA: Vec<Vec<i32>>,
//     mut Matrixx: Vec<(usize, usize)>,
//     mut Matrixb: Vec<i32>,
//     mut board_of_game: Vec<Vec<i32>>,
// ) -> PyResult<(Vec<Vec<i32>>, Vec<(usize, usize)>, Vec<(usize, usize)>)> {
//     let (notMine, is_mine) = ms_toollib_rs::solve_direct(&mut MatrixA, &mut Matrixx, &mut Matrixb, &mut board_of_game);
//     Ok((board_of_game, notMine, is_mine))
// }

#[pyfunction]
#[pyo3(name = "laymine_op_number")]
fn py_laymine_op_number(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
) -> PyResult<Vec<Vec<i32>>> {
    Ok(laymine_op_number(row, column, mine_num, x0, y0))
}

// #[pyfunction(enuLimit = 30)]
// fn solve_enumerate(
//     Matrix_as: Vec<Vec<Vec<i32>>>,
//     Matrix_xs: Vec<Vec<(usize, usize)>>,
//     Matrix_bs: Vec<Vec<i32>>,
//     mut board_of_game: Vec<Vec<i32>>,
//     enuLimit: usize,
// ) -> PyResult<(Vec<Vec<i32>>, Vec<(usize, usize)>, Vec<(usize, usize)>)> {
//     let (notMine, is_mine) = ms_toollib_rs::solve_enumerate(
//         &Matrix_as,
//         &Matrix_xs,
//         &Matrix_bs,
//         enuLimit,
//     );
//     Ok((board_of_game, notMine, is_mine))
// }

#[pyfunction]
#[pyo3(name = "unsolvable_structure")]
fn py_unsolvable_structure(boardCheck: Vec<Vec<i32>>) -> PyResult<bool> {
    Ok(unsolvable_structure(&boardCheck))
}

#[pyfunction(enuLimit = 30)]
#[pyo3(name = "is_solvable")]
fn py_is_solvable(board: Vec<Vec<i32>>, x0: usize, y0: usize, enuLimit: usize) -> PyResult<bool> {
    Ok(is_solvable(&board, x0, y0, enuLimit))
}

#[pyfunction(min3BV = 0, max3BV = 1000_000, max_times = 1000_000, method = 0)]
#[pyo3(name = "laymine_op")]
pub fn py_laymine_op(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
    min3BV: usize,
    max3BV: usize,
    max_times: usize,
    method: usize,
) -> PyResult<(Vec<Vec<i32>>, Vec<usize>)> {
    Ok(laymine_op(
        row, column, mine_num, x0, y0, min3BV, max3BV, max_times, method,
    ))
}

#[pyfunction(min3BV = 0, max3BV = 1000000, max_times = 1000000, enuLimit = 30)]
#[pyo3(name = "laymine_solvable")]
pub fn py_laymine_solvable(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
    min3BV: usize,
    max3BV: usize,
    max_times: usize,
    method: usize,
) -> PyResult<(Vec<Vec<i32>>, Vec<usize>)> {
    Ok(laymine_solvable(
        row, column, mine_num, x0, y0, min3BV, max3BV, max_times, method,
    ))
}

#[pyfunction(min3BV = 0, max3BV = 1000_000, max_times = 1000_000, method = 0)]
#[pyo3(name = "laymine")]
pub fn py_laymine(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
    min3BV: usize,
    max3BV: usize,
    max_times: usize,
    method: usize,
) -> PyResult<(Vec<Vec<i32>>, Vec<usize>)> {
    Ok(laymine(
        row, column, mine_num, x0, y0, min3BV, max3BV, max_times, method,
    ))
}

#[pyfunction(min3BV = 0, max3BV = 1000000, max_times = 1000000, enuLimit = 30)]
#[pyo3(name = "laymine_solvable_thread")]
pub fn py_laymine_solvable_thread(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
    min3BV: usize,
    max3BV: usize,
    mut max_times: usize,
    enuLimit: usize,
) -> PyResult<(Vec<Vec<i32>>, [usize; 3])> {
    Ok(laymine_solvable_thread(
        row, column, mine_num, x0, y0, min3BV, max3BV, max_times, enuLimit,
    ))
}

#[pyfunction]
#[pyo3(name = "cal_possibility")]
fn py_cal_possibility(
    board_of_game: Vec<Vec<i32>>,
    mine_num: f64,
) -> PyResult<(Vec<((usize, usize), f64)>, f64, [usize; 3])> {
    // mine_num为局面中雷的总数，不管有没有标
    // 还返回局面中雷数的范围
    let mut board_of_game = board_of_game.clone();
    mark_board(&mut board_of_game);
    match cal_possibility(&board_of_game, mine_num) {
        Ok(t) => return Ok(t),
        Err(e) => return Ok((vec![], f64::NAN, [0, 0, 0])),
    };
}

#[pyfunction]
#[pyo3(name = "cal_possibility_onboard")]
fn py_cal_possibility_onboard(
    board_of_game: Vec<Vec<i32>>,
    mine_num: f64,
) -> PyResult<(Vec<Vec<f64>>, [usize; 3])> {
    // mine_num为局面中雷的总数，不管有没有标
    let mut board_of_game = board_of_game.clone();
    mark_board(&mut board_of_game);
    match cal_possibility_onboard(&board_of_game, mine_num) {
        Ok(t) => return Ok(t),
        Err(e) => return Ok((vec![], [0, 0, 0])),
    };
}

#[pyfunction]
#[pyo3(name = "sample_3BVs_exp")]
fn py_sample_3BVs_exp(x0: usize, y0: usize, n: usize) -> PyResult<Vec<usize>> {
    Ok((&sample_3BVs_exp(x0, y0, n)).to_vec())
}

#[pyfunction]
#[pyo3(name = "OBR_board", text_signature = "(data_vec, height, width)")]
fn py_OBR_board(data_vec: Vec<usize>, height: usize, width: usize) -> PyResult<Vec<Vec<i32>>> {
    // Ok(OBR_board(data_vec, height, width).unwrap())
    match OBR_board(data_vec, height, width) {
        //判断方法结果
        Ok(ans) => Ok(ans),
        Err(e) => Ok(vec![vec![200]]),
    }
}

// #[pyproto]
// impl PyObjectProtocol for Minesweeperboard {
//     fn __getattr__(&self, name: &str) -> PyResult<usize> {
//         match name {
//             "left" => return Ok(self.left),
//             "right" => return Ok(self.right),
//             "chording" => return Ok(self.chording),
//             "solved3BV" => return Ok(self.solved3BV),
//             "ces" => return Ok(self.ces),
//             "flag" => return Ok(self.flag),
//             _ => return Ok(999),
//         }
//     }
// }

#[pymodule]
fn ms_toollib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_refresh_matrix, m)?)?;
    m.add_function(wrap_pyfunction!(py_refresh_matrixs, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal_op, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal3BV, m)?)?;
    m.add_function(wrap_pyfunction!(py_laymine_number, m)?)?;
    m.add_function(wrap_pyfunction!(py_refresh_board, m)?)?;
    m.add_function(wrap_pyfunction!(py_laymine, m)?)?;
    m.add_function(wrap_pyfunction!(py_get_all_not_mine_on_board, m)?)?;
    m.add_function(wrap_pyfunction!(py_laymine_op_number, m)?)?;
    m.add_function(wrap_pyfunction!(py_laymine_op, m)?)?;
    // m.add_function(wrap_pyfunction!(solve_direct, m)?)?;
    // m.add_function(wrap_pyfunction!(solve_enumerate, m)?)?;
    m.add_function(wrap_pyfunction!(py_unsolvable_structure, m)?)?;
    m.add_function(wrap_pyfunction!(py_is_solvable, m)?)?;
    // m.add_function(wrap_pyfunction!(enuOneStep, m)?)?;
    m.add_function(wrap_pyfunction!(py_laymine_solvable, m)?)?;
    m.add_function(wrap_pyfunction!(py_laymine_solvable_thread, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal_possibility, m)?)?;
    m.add_function(wrap_pyfunction!(py_sample_3BVs_exp, m)?)?;
    m.add_function(wrap_pyfunction!(py_OBR_board, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal_possibility_onboard, m)?)?;
    m.add_class::<MinesweeperBoard>()?;
    m.add_class::<AvfVideo>()?;
    Ok(())
}
