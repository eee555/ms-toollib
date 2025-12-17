use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
// use pyo3::PyTraverseError;
// use pyo3::class::basic::PyObjectProtocol;
// use std::cmp::{max, min};

// 发布pypi：打tag，工作流直接发布

// 开发文档：https://pyo3.rs/latest/

use ms_toollib_original::*;
mod board;
pub use board::{PyMinesweeperBoard, PySafeMinesweeperBoard};

mod gameboard;
pub use gameboard::{PyBoard, PyGameBoard};

mod base_video;
pub use base_video::{
    PyBaseVideo, PyKeyDynamicParams, PySafeBoard, PySafeBoardRow, PyVideoActionStateRecorder,
};

// mod avf_video;
// pub use avf_video::PyAvfVideo;

// mod evf_video;
// pub use evf_video::PyEvfVideo;

// mod mvf_video;
// pub use mvf_video::PyMvfVideo;

// mod rmv_video;
// pub use rmv_video::PyRmvVideo;

mod videos;
pub use videos::{AvfVideo, EvfVideo, MvfVideo, RmvVideo};

mod evfs;
pub use evfs::{PyEvfs, PyEvfsCell};

// pip install maturin
// maturin publish --manylinux 2014

#[pyfunction]
#[pyo3(
    name = "refresh_matrix",
    signature = (game_board)
)]
fn py_refresh_matrix(
    game_board: Vec<Vec<i32>>,
) -> PyResult<(Vec<Vec<i32>>, Vec<(usize, usize)>, Vec<i32>)> {
    Ok(refresh_matrix(&game_board))
}

#[pyfunction]
#[pyo3(
    name = "refresh_matrixs",
    signature = (game_board)
)]
fn py_refresh_matrixs(
    game_board: Vec<Vec<i32>>,
) -> PyResult<(
    Vec<Vec<Vec<i32>>>,
    Vec<Vec<(usize, usize)>>,
    Vec<Vec<i32>>,
    usize,
    usize,
)> {
    Ok(refresh_matrixs(&game_board))
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
#[pyo3(
    name = "cal_op",
    signature = (board)
)]
fn py_cal_op(board: Vec<Vec<i32>>) -> PyResult<usize> {
    Ok(cal_op(&board))
}

///  通用标准埋雷引擎。起手位置非雷，其余位置的雷服从均匀分布。
/// 
/// # 参数
/// - `row`: 局面行数。
/// - `column`：局面列数。。
/// - `mine_num`：雷数。
/// - `x0`：起手位置在第几行。
/// - `y0`：起手位置在第几列。
///
/// # 返回值
/// 二维的局面，其中0代表空，1~8代表1~8，-1代表雷。
#[pyfunction]
#[pyo3(name = "laymine", signature = (row, column, mine_num, x0, y0))]
fn py_laymine(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
) -> PyResult<Vec<Vec<i32>>> {
    Ok(laymine(row, column, mine_num, x0, y0))
}

#[pyfunction]
#[pyo3(
    name = "cal_bbbv",
    signature = (board)
)]
fn py_cal_bbbv(board: Vec<Vec<i32>>) -> PyResult<usize> {
    Ok(cal_bbbv(&board))
}

#[pyfunction]
#[pyo3(name = "solve_minus")]
fn py_solve_minus(
    mut a_mats: Vec<Vec<Vec<i32>>>,
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
    let t = solve_minus(&mut a_mats, &mut xs, &mut bs, &mut board_of_game);
    match t {
        Ok(aa) => {
            not = aa.0;
            is = aa.1;
        }
        Err(code) => return Err(PyErr::new::<PyRuntimeError, _>(format!("code: {}.", code))),
    };
    Ok((a_mats, xs, bs, board_of_game, not, is))
}

#[pyfunction]
#[pyo3(name = "refresh_board")]
fn py_refresh_board(
    board: Vec<Vec<i32>>,
    mut board_of_game: Vec<Vec<i32>>,
    clicked_poses: Vec<(usize, usize)>,
) -> PyResult<Vec<Vec<i32>>> {
    refresh_board(&board, &mut board_of_game, clicked_poses);
    Ok(board_of_game)
}

#[pyfunction]
#[pyo3(
    name = "get_all_not_and_is_mine_on_board",
    signature = (game_board)
)]
fn py_get_all_not_and_is_mine_on_board(
    mut game_board: Vec<Vec<i32>>,
) -> PyResult<(Vec<Vec<i32>>, Vec<(usize, usize)>, Vec<(usize, usize)>)> {
    let (mut a_mats, mut xs, mut bs, _, _) = refresh_matrixs(&game_board);
    let (not, is) =
        get_all_not_and_is_mine_on_board(&mut a_mats, &mut xs, &mut bs, &mut game_board);
    Ok((game_board, not, is))
}

#[pyfunction]
#[pyo3(name = "solve_direct")]
fn py_solve_direct(
    mut a_mats: Vec<Vec<Vec<i32>>>,
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
    let t = solve_direct(&mut a_mats, &mut xs, &mut bs, &mut board_of_game);
    match t {
        Ok(aa) => {
            not = aa.0;
            is = aa.1;
        }
        Err(code) => return Err(PyErr::new::<PyRuntimeError, _>(format!("code: {}.", code))),
    };
    // let (not, is) = solve_direct(&mut a_mats, &mut xs, &mut bs, &mut board_of_game);
    Ok((a_mats, xs, bs, board_of_game, not, is))
}


///  通用win7规则埋雷引擎。起手位置开空，其余位置的雷服从均匀分布。
/// 
/// # 参数
/// - `row`: 局面行数。
/// - `column`：局面列数。。
/// - `mine_num`：雷数。
/// - `x0`：起手位置在第几行。
/// - `y0`：起手位置在第几列。
///
/// # 返回值
/// 二维的局面，其中0代表空，1~8代表1~8，-1代表雷。
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
    let (a_mats, xs, bs, _, _) = refresh_matrixs(&board_of_game);
    let (not, is) = solve_enumerate(&a_mats, &xs, &bs);
    Ok((not, is))
}

#[pyfunction]
#[pyo3(name = "unsolvable_structure")]
fn py_unsolvable_structure(board_check: Vec<Vec<i32>>) -> PyResult<bool> {
    Ok(unsolvable_structure(&board_check))
}

#[pyfunction]
#[pyo3(
    name = "is_solvable",
    signature = (board, x0, y0)
)]
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
#[pyo3(
    name = "cal_probability",
    signature = (game_board, mine_num)
)]
fn py_cal_probability(
    mut game_board: Vec<Vec<i32>>,
    mine_num: f64,
) -> PyResult<(Vec<((usize, usize), f64)>, f64, [usize; 3], usize)> {
    // mine_num为局面中雷的总数，不管有没有标
    // 还返回局面中雷数的范围
    mark_board(&mut game_board, true)
        .map_err(|_| PyErr::new::<PyRuntimeError, _>("标记阶段无解的局面"))?;
    match cal_probability(&game_board, mine_num) {
        Ok(t) => return Ok(t),
        Err(1) => return Err(PyErr::new::<PyRuntimeError, _>("枚举阶段无解的局面")),
        _ => return Err(PyErr::new::<PyRuntimeError, _>("未知的错误")),
    };
}

/// 计算局面中各位置是雷的概率，按照所在的位置返回。
///
/// # 参数
/// - `game_board`: 游戏局面。自动纠正错误的标雷。
/// - `mine_num`：雷数。>=1时，理解为总的雷数；<1时，理解为雷的比例。
///
/// # 返回值
/// - 元组的第一个元素是一个局面中，所有位置是雷的概率
/// - 元组的第二个元素是一个长度为3的列表，表示最小雷数、当前雷数、最大雷数
/// 
/// # 异常
/// - `PyRuntimeError`: `标记阶段无解的局面`和`枚举阶段无解的局面`两种。
#[pyfunction]
#[pyo3(
    name = "cal_probability_onboard",
    signature = (game_board, mine_num)
)]
fn py_cal_probability_onboard(
    // 可以接受无解的局面
    mut game_board: Vec<Vec<i32>>,
    mine_num: f64,
) -> PyResult<(Vec<Vec<f64>>, [usize; 3])> {
    // mine_num为局面中雷的总数，不管有没有标
    let legal_flag = mark_board(&mut game_board, true);
    match legal_flag {
        Ok(_) => {}
        Err(_) => return Err(PyErr::new::<PyRuntimeError, _>("标记阶段无解的局面")),
    }
    match cal_probability_onboard(&game_board, mine_num) {
        Ok(t) => return Ok(t),
        Err(_) => return Err(PyErr::new::<PyRuntimeError, _>("枚举阶段无解的局面")),
    };
}

#[pyfunction]
#[pyo3(
    name = "sample_bbbvs_exp",
    signature = (x0, y0, n)
)]
fn py_sample_bbbvs_exp(x0: usize, y0: usize, n: usize) -> PyResult<[usize; 382]> {
    Ok(sample_bbbvs_exp(x0, y0, n))
}

// #[pyfunction]
// #[pyo3(name = "OBR_board", signature = (data_vec, height, width))]
// fn py_obr_board_old(data_vec: Vec<usize>, height: usize, width: usize) -> PyResult<Vec<Vec<i32>>> {
//     let _ = Python::with_gil(|py| {
//         let deprecation_warning = py.get_type_bound::<pyo3::exceptions::PyDeprecationWarning>();
//         PyErr::warn_bound(py, &deprecation_warning, "Renamed to obr_board", 0)?;
//         Ok::<(), PyErr>(())
//     });
//     // Ok(obr_board(data_vec, height, width).unwrap())
//     match obr_board(data_vec, height, width) {
//         //判断方法结果
//         Ok(ans) => Ok(ans),
//         Err(_e) => Ok(vec![vec![200]]),
//     }
// }


#[pyfunction]
#[pyo3(name = "obr_board", signature = (data_vec, height, width))]
fn py_obr_board(data_vec: Vec<usize>, height: usize, width: usize) -> PyResult<Vec<Vec<i32>>> {
    // Ok(obr_board(data_vec, height, width).unwrap())
    match obr_board(data_vec, height, width) {
        //判断方法结果
        Ok(ans) => Ok(ans),
        Err(_e) => Ok(vec![vec![200]]),
    }
}

#[pyfunction]
#[pyo3(
    name = "mark_board",
    signature = (game_board, remark = false)
)]
fn py_mark_board(mut game_board: Vec<Vec<i32>>, remark: bool) -> PyResult<Vec<Vec<i32>>> {
    mark_board(&mut game_board, remark).unwrap();
    Ok(game_board)
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

/// 软件的合法时间范围。单位为秒。
///
/// # 参数
/// - `software`: 软件名称，"Arbiter"、"0.97 beta"、"Viennasweeper"、"元3.1.9"、"元3.1.11"、"元3.2.0"等等
///
/// # 返回值
/// 秒为单位的开始时间戳字符串、秒为单位的结束时间戳字符串
#[pyfunction]
#[pyo3(name = "valid_time_period")]
fn py_valid_time_period(software: &str) -> PyResult<(String, String)> {
    match valid_time_period(software) {
        Ok(a) => Ok(a),
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
    m.add_function(wrap_pyfunction!(py_cal_probability, m)?)?;
    m.add_function(wrap_pyfunction!(py_sample_bbbvs_exp, m)?)?;
    m.add_function(wrap_pyfunction!(py_obr_board, m)?)?;
    // m.add_function(wrap_pyfunction!(py_obr_board_old, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal_probability_onboard, m)?)?;
    m.add_function(wrap_pyfunction!(py_mark_board, m)?)?;
    m.add_function(wrap_pyfunction!(py_is_guess_while_needless, m)?)?;
    m.add_function(wrap_pyfunction!(py_is_able_to_solve, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal_all_solution, m)?)?;
    m.add_function(wrap_pyfunction!(py_cal_board_numbers, m)?)?;
    m.add_function(wrap_pyfunction!(py_valid_time_period, m)?)?;
    m.add_class::<PyMinesweeperBoard>()?;
    m.add_class::<PySafeMinesweeperBoard>()?;
    m.add_class::<AvfVideo>()?;
    m.add_class::<RmvVideo>()?;
    m.add_class::<MvfVideo>()?;
    m.add_class::<EvfVideo>()?;
    m.add_class::<PyBaseVideo>()?;
    m.add_class::<PyGameBoard>()?;
    m.add_class::<PyBoard>()?;
    m.add_class::<PySafeBoard>()?;
    m.add_class::<PySafeBoardRow>()?;
    m.add_class::<PyVideoActionStateRecorder>()?;
    m.add_class::<PyKeyDynamicParams>()?;
    m.add_class::<PyEvfs>()?;
    m.add_class::<PyEvfsCell>()?;
    Ok(())
}
