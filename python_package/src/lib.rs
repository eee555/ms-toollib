use pyo3::exceptions::PyRuntimeError;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
// use pyo3::PyTraverseError;
// use pyo3::class::basic::PyObjectProtocol;
// use std::cmp::{max, min};

use ms_toollib_original::*;
mod board;
pub use board::{PyMinesweeperBoard, PySafeMinesweeperBoard};

mod gameboard;
pub use gameboard::{PyBoard, PyGameBoard};

mod base_video;
pub use base_video::{PyBaseVideo, PySafeBoard};

mod avf_video;
pub use avf_video::PyAvfVideo;

mod evf_video;
pub use evf_video::PyEvfVideo;

mod mvf_video;
pub use mvf_video::PyMvfVideo;

mod rmv_video;
pub use rmv_video::PyRmvVideo;

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
#[pyo3(name = "refresh_matrixses")]
fn py_refresh_matrixses(
    board_of_game: Vec<Vec<i32>>,
) -> PyResult<(
    Vec<Vec<Vec<Vec<i32>>>>,
    Vec<Vec<Vec<(usize, usize)>>>,
    Vec<Vec<Vec<i32>>>,
)> {
    Ok(refresh_matrixses(&board_of_game))
}

#[pyfunction]
#[pyo3(name = "cal_op")]
fn py_cal_op(board: Vec<Vec<i32>>) -> PyResult<usize> {
    Ok(cal_op(&board))
}

#[pyfunction]
#[pyo3(name = "laymine", signature = (row, column, mine_num, x0, y0))]
fn py_laymine(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
) -> PyResult<Vec<Vec<i32>>> {
    // 通用标准埋雷引擎
    // 输出为二维的局面
    Ok(laymine(row, column, mine_num, x0, y0))
}

#[pyfunction]
#[pyo3(name = "cal_bbbv")]
fn py_cal_bbbv(board: Vec<Vec<i32>>) -> PyResult<usize> {
    Ok(cal_bbbv(&board))
}

#[pyfunction]
#[pyo3(name = "solve_minus")]
fn py_solve_minus(
    mut As: Vec<Vec<Vec<i32>>>,
    mut xs: Vec<Vec<(usize, usize)>>,
    mut bs: Vec<Vec<i32>>,
    mut board_of_game: Vec<Vec<i32>>,
) -> PyResult<(
    Vec<Vec<Vec<i32>>>,
    Vec<Vec<(usize, usize)>>,
    Vec<Vec<i32>>,
    Vec<Vec<i32>>,
    Vec<(usize, usize)>,
    Vec<(usize, usize)>,
)> {
    let not;
    let is;
    let t = solve_minus(&mut As, &mut xs, &mut bs, &mut board_of_game);
    match t {
        Ok(aa) => {
            not = aa.0;
            is = aa.1;
        }
        Err(code) => return Err(PyErr::new::<PyRuntimeError, _>(format!("code: {}.", code))),
    };
    Ok((As, xs, bs, board_of_game, not, is))
}

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
#[pyo3(name = "get_all_not_and_is_mine_on_board")]
fn py_get_all_not_and_is_mine_on_board(
    mut board_of_game: Vec<Vec<i32>>,
) -> PyResult<(Vec<Vec<i32>>, Vec<(usize, usize)>, Vec<(usize, usize)>)> {
    let (mut As, mut xs, mut bs, _, _) = refresh_matrixs(&board_of_game);
    let (not, is) = get_all_not_and_is_mine_on_board(&mut As, &mut xs, &mut bs, &mut board_of_game);
    Ok((board_of_game, not, is))
}

#[pyfunction]
#[pyo3(name = "solve_direct")]
fn py_solve_direct(
    mut As: Vec<Vec<Vec<i32>>>,
    mut xs: Vec<Vec<(usize, usize)>>,
    mut bs: Vec<Vec<i32>>,
    mut board_of_game: Vec<Vec<i32>>,
) -> PyResult<(
    Vec<Vec<Vec<i32>>>,
    Vec<Vec<(usize, usize)>>,
    Vec<Vec<i32>>,
    Vec<Vec<i32>>,
    Vec<(usize, usize)>,
    Vec<(usize, usize)>,
)> {
    let not;
    let is;
    let t = solve_direct(&mut As, &mut xs, &mut bs, &mut board_of_game);
    match t {
        Ok(aa) => {
            not = aa.0;
            is = aa.1;
        }
        Err(code) => return Err(PyErr::new::<PyRuntimeError, _>(format!("code: {}.", code))),
    };
    // let (not, is) = solve_direct(&mut As, &mut xs, &mut bs, &mut board_of_game);
    Ok((As, xs, bs, board_of_game, not, is))
}

#[pyfunction]
#[pyo3(
    name = "laymine_op",
    signature = (row, column, mine_num, x0, y0)
)]
fn py_laymine_op(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
) -> PyResult<Vec<Vec<i32>>> {
    Ok(laymine_op(row, column, mine_num, x0, y0))
}

#[pyfunction]
#[pyo3(name = "solve_enumerate")]
fn py_solve_enumerate(
    board_of_game: Vec<Vec<i32>>,
) -> PyResult<(Vec<(usize, usize)>, Vec<(usize, usize)>)> {
    let (As, xs, bs, _, _) = refresh_matrixs(&board_of_game);
    let (not, is) = solve_enumerate(&As, &xs, &bs);
    Ok((not, is))
}

#[pyfunction]
#[pyo3(name = "unsolvable_structure")]
fn py_unsolvable_structure(boardCheck: Vec<Vec<i32>>) -> PyResult<bool> {
    Ok(unsolvable_structure(&boardCheck))
}

#[pyfunction]
#[pyo3(name = "is_solvable")]
fn py_is_solvable(board: Vec<Vec<i32>>, x0: usize, y0: usize) -> PyResult<bool> {
    Ok(is_solvable(&board, x0, y0))
}

#[pyfunction]
#[pyo3(
    name = "laymine_solvable",
    signature = (row, column, mine_num, x0, y0, max_times = 1000000)
)]
pub fn py_laymine_solvable(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
    max_times: usize,
) -> PyResult<(Vec<Vec<i32>>, bool)> {
    Ok(laymine_solvable(row, column, mine_num, x0, y0, max_times))
}

#[pyfunction]
#[pyo3(
    name = "laymine_solvable_thread",
    signature = (row, column, mine_num, x0, y0, max_times = 1000000)
)]
pub fn py_laymine_solvable_thread(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
    max_times: usize,
) -> PyResult<(Vec<Vec<i32>>, bool)> {
    Ok(laymine_solvable_thread(
        row, column, mine_num, x0, y0, max_times,
    ))
}

#[pyfunction]
#[pyo3(
    name = "laymine_solvable_adjust",
    signature = (row, column, mine_num, x0, y0)
)]
pub fn py_laymine_solvable_adjust(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
) -> PyResult<(Vec<Vec<i32>>, bool)> {
    Ok(laymine_solvable_adjust(row, column, mine_num, x0, y0))
}

#[pyfunction]
#[pyo3(name = "cal_possibility")]
fn py_cal_possibility(
    mut board_of_game: Vec<Vec<i32>>,
    mine_num: f64,
) -> PyResult<(Vec<((usize, usize), f64)>, f64, [usize; 3], usize)> {
    // mine_num为局面中雷的总数，不管有没有标
    // 还返回局面中雷数的范围
    let legal_flag = mark_board(&mut board_of_game);
    match legal_flag {
        Ok(_) => {}
        Err(_) => return Err(PyErr::new::<PyRuntimeError, _>("标记阶段无解的局面")),
    }
    match cal_possibility(&board_of_game, mine_num) {
        Ok(t) => return Ok(t),
        Err(1) => return Err(PyErr::new::<PyRuntimeError, _>("枚举阶段无解的局面")),
        _ => return Err(PyErr::new::<PyRuntimeError, _>("未知的错误")),
    };
}

#[pyfunction]
#[pyo3(name = "cal_possibility_onboard")]
fn py_cal_possibility_onboard(
    // 可以接受无解的局面
    mut board_of_game: Vec<Vec<i32>>,
    mine_num: f64,
) -> PyResult<(Vec<Vec<f64>>, [usize; 3])> {
    // mine_num为局面中雷的总数，不管有没有标
    let legal_flag = mark_board(&mut board_of_game);
    match legal_flag {
        Ok(_) => {}
        Err(_) => return Err(PyErr::new::<PyRuntimeError, _>("标记阶段无解的局面")),
    }
    match cal_possibility_onboard(&board_of_game, mine_num) {
        Ok(t) => return Ok(t),
        Err(_) => return Err(PyErr::new::<PyRuntimeError, _>("枚举阶段无解的局面")),
    };
}

#[pyfunction]
#[pyo3(name = "sample_3BVs_exp")]
fn py_sample_3BVs_exp(x0: usize, y0: usize, n: usize) -> PyResult<Vec<usize>> {
    Ok((&sample_3BVs_exp(x0, y0, n)).to_vec())
}

#[pyfunction]
#[pyo3(name = "OBR_board", signature = (data_vec, height, width))]
fn py_OBR_board(data_vec: Vec<usize>, height: usize, width: usize) -> PyResult<Vec<Vec<i32>>> {
    // Ok(OBR_board(data_vec, height, width).unwrap())
    match OBR_board(data_vec, height, width) {
        //判断方法结果
        Ok(ans) => Ok(ans),
        Err(e) => Ok(vec![vec![200]]),
    }
}

#[pyfunction]
#[pyo3(name = "mark_board")]
fn py_mark_board(mut board_of_game: Vec<Vec<i32>>) -> PyResult<Vec<Vec<i32>>> {
    mark_board(&mut board_of_game).unwrap();
    Ok(board_of_game)
}

#[pyfunction]
#[pyo3(name = "is_guess_while_needless")]
fn py_is_guess_while_needless(
    mut board_of_game: Vec<Vec<i32>>,
    xy: (usize, usize),
) -> PyResult<i32> {
    Ok(is_guess_while_needless(&mut board_of_game, &xy))
}

#[pyfunction]
#[pyo3(name = "is_able_to_solve")]
fn py_is_able_to_solve(mut board_of_game: Vec<Vec<i32>>, xy: (usize, usize)) -> PyResult<bool> {
    Ok(is_able_to_solve(&mut board_of_game, &xy))
}

#[pyfunction]
#[pyo3(name = "cal_all_solution")]
fn py_cal_all_solution(a: Vec<Vec<i32>>, b: Vec<i32>) -> PyResult<Vec<Vec<u8>>> {
    Ok(cal_all_solution(&a, &b))
}

#[pyfunction]
#[pyo3(name = "cal_board_numbers")]
fn py_cal_board_numbers(mut board: Vec<Vec<i32>>) -> PyResult<Vec<Vec<i32>>> {
    cal_board_numbers(&mut board);
    Ok(board)
}

#[pyfunction]
#[pyo3(name = "cal_board_numbers")]
fn py_valid_time_period(software: &str) -> PyResult<(String, String)> {
    match valid_time_period(software) {
        Ok(a)=> Ok(a),
        Err(e) => Err(PyErr::new::<PyRuntimeError, _>(e)),
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
fn ms_toollib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_refresh_matrix, m)?)?;
    m.add_function(wrap_pyfunction!(py_refresh_matrixs, m)?)?;
    m.add_function(wrap_pyfunction!(py_refresh_matrixses, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal_op, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal_bbbv, m)?)?;
    m.add_function(wrap_pyfunction!(py_refresh_board, m)?)?;
    m.add_function(wrap_pyfunction!(py_laymine, m)?)?;
    m.add_function(wrap_pyfunction!(py_get_all_not_and_is_mine_on_board, m)?)?;
    m.add_function(wrap_pyfunction!(py_laymine_op, m)?)?;
    m.add_function(wrap_pyfunction!(py_solve_direct, m)?)?;
    m.add_function(wrap_pyfunction!(py_solve_minus, m)?)?;
    m.add_function(wrap_pyfunction!(py_solve_enumerate, m)?)?;
    m.add_function(wrap_pyfunction!(py_unsolvable_structure, m)?)?;
    m.add_function(wrap_pyfunction!(py_is_solvable, m)?)?;
    m.add_function(wrap_pyfunction!(py_laymine_solvable, m)?)?;
    m.add_function(wrap_pyfunction!(py_laymine_solvable_thread, m)?)?;
    m.add_function(wrap_pyfunction!(py_laymine_solvable_adjust, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal_possibility, m)?)?;
    m.add_function(wrap_pyfunction!(py_sample_3BVs_exp, m)?)?;
    m.add_function(wrap_pyfunction!(py_OBR_board, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal_possibility_onboard, m)?)?;
    m.add_function(wrap_pyfunction!(py_mark_board, m)?)?;
    m.add_function(wrap_pyfunction!(py_is_guess_while_needless, m)?)?;
    m.add_function(wrap_pyfunction!(py_is_able_to_solve, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal_all_solution, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal_board_numbers, m)?)?;
    m.add_function(wrap_pyfunction!(py_valid_time_period, m)?)?;
    m.add_class::<PyMinesweeperBoard>()?;
    m.add_class::<PySafeMinesweeperBoard>()?;
    m.add_class::<PyAvfVideo>()?;
    m.add_class::<PyRmvVideo>()?;
    m.add_class::<PyMvfVideo>()?;
    m.add_class::<PyEvfVideo>()?;
    m.add_class::<PyBaseVideo>()?;
    m.add_class::<PyGameBoard>()?;
    m.add_class::<PyBoard>()?;
    m.add_class::<PySafeBoard>()?;
    Ok(())
}
