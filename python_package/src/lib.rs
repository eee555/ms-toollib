use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
// use pyo3::PyTraverseError;
use pyo3::class::basic::PyObjectProtocol;
use std::cmp::{max, min};

use ms_toollib as ms_toollib_rs;
use ms_toollib::refreshBoard as refresh_board_;


// pip install maturin
// maturin publish --manylinux 2014

#[pyfunction]
fn refresh_matrix(
    board_of_game: Vec<Vec<i32>>,
) -> PyResult<(Vec<Vec<i32>>, Vec<(usize, usize)>, Vec<i32>)> {
    Ok(ms_toollib_rs::refresh_matrix(&board_of_game))
}

#[pyfunction]
fn refresh_matrixs(
    board_of_game: Vec<Vec<i32>>,
) -> PyResult<(
    Vec<Vec<Vec<i32>>>,
    Vec<Vec<(usize, usize)>>,
    Vec<Vec<i32>>,
    usize,
    usize,
)> {
    Ok(ms_toollib_rs::refresh_matrixs(&board_of_game))
}

#[pyfunction]
fn cal_op(board: Vec<Vec<i32>>) -> PyResult<usize> {
    Ok(ms_toollib_rs::cal_op(board))
}

#[pyfunction]
fn laymine_number(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
) -> PyResult<Vec<Vec<i32>>> {
    // 通用标准埋雷引擎
    // 输出为二维的局面
    Ok(ms_toollib_rs::laymine_number(row, column, mine_num, x0, y0))
}

#[pyfunction]
fn cal3BV(board: Vec<Vec<i32>>) -> PyResult<usize> {
    Ok(ms_toollib_rs::cal3BV(&board))
}

#[pyfunction]
fn solve_minus(
    mut MatrixA: Vec<Vec<i32>>,
    mut Matrixx: Vec<(usize, usize)>,
    mut Matrixb: Vec<i32>,
    mut board_of_game: Vec<Vec<i32>>,
) -> PyResult<(Vec<Vec<i32>>, Vec<(usize, usize)>, bool)> {
    let (notMine, flag) = ms_toollib_rs::solve_minus(&mut MatrixA, &mut Matrixx, &mut Matrixb, &mut board_of_game);
    Ok((board_of_game, notMine, flag))
}

#[pyfunction]
fn refresh_board(
    board: Vec<Vec<i32>>,
    mut board_of_game: Vec<Vec<i32>>,
    ClickedPoses: Vec<(usize, usize)>,
) -> PyResult<Vec<Vec<i32>>> {
    refresh_board_(&board, &mut board_of_game, ClickedPoses);
    Ok(board_of_game)
}

#[pyfunction]
fn solve_direct(
    mut MatrixA: Vec<Vec<i32>>,
    mut Matrixx: Vec<(usize, usize)>,
    mut Matrixb: Vec<i32>,
    mut board_of_game: Vec<Vec<i32>>,
) -> PyResult<(Vec<Vec<i32>>, Vec<(usize, usize)>, bool)> {
    let (notMine, flag) = ms_toollib_rs::solve_direct(&mut MatrixA, &mut Matrixx, &mut Matrixb, &mut board_of_game);
    Ok((board_of_game, notMine, flag))
}

#[pyfunction]
fn laymine_op_number(
    row: usize,
    column: usize,
    mine_num: usize,
    x0: usize,
    y0: usize,
) -> PyResult<Vec<Vec<i32>>> {
    Ok(ms_toollib_rs::laymine_op_number(row, column, mine_num, x0, y0))
}

#[pyfunction(enuLimit = 30)]
fn solve_enumerate(
    Matrix_as: Vec<Vec<Vec<i32>>>,
    Matrix_xs: Vec<Vec<(usize, usize)>>,
    Matrix_bs: Vec<Vec<i32>>,
    mut board_of_game: Vec<Vec<i32>>,
    enuLimit: usize,
) -> PyResult<(Vec<Vec<i32>>, Vec<(usize, usize)>, bool)> {
    let (notMine, flag) = ms_toollib_rs::solve_enumerate(
        &Matrix_as,
        &Matrix_xs,
        &Matrix_bs,
        &mut board_of_game,
        enuLimit,
    );
    Ok((board_of_game, notMine, flag))
}

// #[pyfunction]
// fn py_enuOneStep(
//     AllTable: Vec<Vec<usize>>,
//     TableId: Vec<usize>,
//     b: i32,
// ) -> PyResult<Vec<Vec<usize>>> {
//     Ok(ms_toollib_rs::enuone_step(AllTable, TableId, b))
// }

#[pyfunction]
fn unsolvable_structure(boardCheck: Vec<Vec<i32>>) -> PyResult<bool> {
    Ok(ms_toollib_rs::unsolvable_structure(&boardCheck))
}

#[pyfunction(enuLimit = 30)]
fn is_solvable(board: Vec<Vec<i32>>, x0: usize, y0: usize, enuLimit: usize) -> PyResult<bool> {
    Ok(ms_toollib_rs::is_solvable(&board, x0, y0, enuLimit))
}

#[pyfunction(min3BV = 0, max3BV = 1000_000, max_times = 1000_000, method = 0)]
pub fn laymine_op(
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
    Ok(ms_toollib_rs::laymine_op(
        row, column, mine_num, x0, y0, min3BV, max3BV, max_times, method,
    ))
}

#[pyfunction(min3BV = 0, max3BV = 1000000, max_times = 1000000, enuLimit = 30)]
pub fn laymine_solvable(
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
    Ok(ms_toollib_rs::laymine_solvable(
        row, column, mine_num, x0, y0, min3BV, max3BV, max_times, method,
    ))
}

#[pyfunction(min3BV = 0, max3BV = 1000_000, max_times = 1000_000, method = 0)]
pub fn laymine(
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
    Ok(ms_toollib_rs::laymine(
        row, column, mine_num, x0, y0, min3BV, max3BV, max_times, method,
    ))
}

#[pyfunction(min3BV = 0, max3BV = 1000000, max_times = 1000000, enuLimit = 30)]
pub fn laymine_solvable_thread(
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
    Ok(ms_toollib_rs::laymine_solvable_thread(
        row, column, mine_num, x0, y0, min3BV, max3BV, max_times, enuLimit,
    ))
}

mark_board

#[pyfunction]
fn cal_possibility(
    board_of_game: Vec<Vec<i32>>,
    mine_num: f64,
) -> PyResult<(Vec<((usize, usize), f64)>, f64, [usize; 3])> {
    // mine_num为局面中雷的总数，不管有没有标
    // 还返回局面中雷数的范围
    let mut board_of_game = board_of_game.clone();
    ms_toollib_rs::mark_board(&mut board_of_game);
    match ms_toollib_rs::cal_possibility(&board_of_game, mine_num) {
        Ok(t) => return Ok(t),
        Err(e) => return Ok((vec![], f64::NAN, [0, 0, 0])),
    };
}

#[pyfunction]
fn cal_possibility_onboard(
    board_of_game: Vec<Vec<i32>>,
    mine_num: f64,
) -> PyResult<(Vec<Vec<f64>>, [usize; 3])> {
    // mine_num为局面中雷的总数，不管有没有标
    let mut board_of_game = board_of_game.clone();
    ms_toollib_rs::mark_board(&mut board_of_game);
    match ms_toollib_rs::cal_possibility_onboard(&board_of_game, mine_num) {
        Ok(t) => return Ok(t),
        Err(e) => return Ok((vec![], [0, 0, 0])),
    };
}

#[pyfunction]
fn sample_3BVs_exp(x0: usize, y0: usize, n: usize) -> PyResult<Vec<usize>> {
    Ok((&ms_toollib_rs::sample_3BVs_exp(x0, y0, n)).to_vec())
}

#[pyfunction]
fn OBR_board(data_vec: Vec<usize>, height: usize, width: usize) -> PyResult<Vec<Vec<i32>>> {
    // Ok(OBR_board(data_vec, height, width).unwrap())
    match ms_toollib_rs::OBR_board(data_vec, height, width) {
        //判断方法结果
        Ok(ans) => Ok(ans),
        Err(e) => Ok(vec![vec![200]]),
    }
}

#[pyclass]
struct Minesweeperboard {
    // 局面类，分析操作与局面的交互
    #[pyo3(get)]
    board: Vec<Vec<i32>>,
    gameboard: Vec<Vec<i32>>,
    flagedList: Vec<(usize, usize)>, //记录哪些雷曾经被标过，则再标这些雷不记为ce
    left: usize,
    right: usize,
    chording: usize,
    ces: usize,
    flag: usize,
    solved3BV: usize,
    row: usize,
    column: usize,
    rightFlag: bool,    // 若rightFlag=True，则如果紧接着再chording就要把right减去1
    chordingFlag: bool, // chordingFlag=True，代表上一个时刻是双击弹起，此时再弹起左键或右键不做任何处理
}

#[pymethods]
impl Minesweeperboard {
    #[new]
    pub fn new(board: Vec<Vec<i32>>) -> Minesweeperboard {
        let row = board.len();
        let column = board[0].len();
        Minesweeperboard {
            board: board,
            row: row,
            column: column,
            gameboard: vec![vec![10; column]; row],
            left: 0,
            right: 0,
            chording: 0,
            ces: 0,
            flag: 0,
            solved3BV: 0,
            rightFlag: false,
            chordingFlag: false,
            flagedList: vec![],
        }
    }
    fn leftClick(&mut self, x: usize, y: usize) {
        self.left += 1;
        if self.gameboard[x][y] != 10 {
            return;
        }
        match self.board[x][y] {
            0 => {
                self.solved3BV += 1;
                self.ces += 1;
                refresh_board_(&self.board, &mut self.gameboard, vec![(x, y)]);
                return;
            }
            -1 => {
                return;
            }
            _ => {
                refresh_board_(&self.board, &mut self.gameboard, vec![(x, y)]);
                if self.numIs3BV(x, y) {
                    self.solved3BV += 1;
                    self.ces += 1;
                    return;
                } else {
                    self.ces += 1;
                    return;
                }
            }
        }
    }
    fn rightClick(&mut self, x: usize, y: usize) {
        self.right += 1;
        if self.gameboard[x][y] < 10 {
            return;
        } else {
            if self.board[x][y] != -1 {
                match self.gameboard[x][y] {
                    10 => {
                        self.gameboard[x][y] = 11;
                        self.flag += 1;
                    }
                    11 => {
                        self.gameboard[x][y] = 10;
                        self.flag -= 1;
                    }
                    _ => return,
                }
                return;
            } else {
                match self.gameboard[x][y] {
                    10 => {
                        self.gameboard[x][y] = 11;
                        self.flag += 1;
                        self.flagedList.push((x, y));
                        let mut not_flag_flaged = true;
                        for flags in self.flagedList.clone() {
                            if x == flags.0 && y == flags.1 {
                                not_flag_flaged = false;
                                break;
                            }
                        }
                        if not_flag_flaged {
                            self.ces += 1;
                        }
                    }
                    11 => {
                        self.gameboard[x][y] = 10;
                        self.flag -= 1;
                    }
                    _ => return,
                }
                return;
            }
        }
    }
    fn chordingClick(&mut self, x: usize, y: usize) {
        self.chording += 1;
        if self.gameboard[x][y] == 0 || self.gameboard[x][y] > 8 {
            return;
        }
        let mut flagChordingUseful = false; // 双击有效的基础上，周围是否有未打开的格子
        let mut chordingCells = vec![]; // 未打开的格子的集合
        let mut flagedNum = 0; // 双击点周围的标雷数
        let mut surround3BV = 0; // 周围的3BV
        let mut flag_ch_op = false; // 是否通过双击开空了：一次双击最多打开一个空
        for i in max(1, x) - 1..min(self.row, x + 2) {
            for j in max(1, y) - 1..min(self.column, y + 2) {
                if i != x || j != y {
                    if self.gameboard[i][j] == 11 {
                        flagedNum += 1
                    }
                    if self.gameboard[i][j] == 10 && self.board[i][j] != -1 {
                        if self.board[i][j] == 0 {
                            flag_ch_op = true;
                        }
                        flagChordingUseful = true;
                        chordingCells.push((i, j));
                        if self.numIs3BV(i, j) {
                            surround3BV += 1;
                        }
                    }
                }
            }
        }
        if flagedNum == self.gameboard[x][y] && flagChordingUseful {
            self.ces += 1;
            self.solved3BV += surround3BV;
            if flag_ch_op {
                self.solved3BV += 1;
            }
            refresh_board_(&self.board, &mut self.gameboard, chordingCells);
        }
    }
    pub fn numIs3BV(&self, x: usize, y: usize) -> bool {
        // 判断该数字是不是3BV，0也可以
        if self.board[x][y] == -1 {
            return false;
        }
        for i in max(1, x) - 1..min(self.row, x + 2) {
            for j in max(1, y) - 1..min(self.column, y + 2) {
                if self.board[i][j] == 0 {
                    return false;
                }
            }
        }
        true
    }
    pub fn step(&mut self, operation: Vec<(&str, (usize, usize))>) {
        for op in operation {
            match op.0 {
                "c1" => {
                    if self.rightFlag {
                        self.rightFlag = false;
                        self.right -= 1;
                    }
                }
                "l2" => {
                    if self.chordingFlag {
                        self.chordingFlag = false;
                    } else {
                        self.leftClick(op.1 .0, op.1 .1)
                    }
                }
                "r1" => self.rightClick(op.1 .0, op.1 .1),
                "c2" => {
                    self.chordingClick(op.1 .0, op.1 .1);
                    self.chordingFlag = true;
                }
                "r2" => {
                    if self.chordingFlag {
                        self.chordingFlag = false;
                    }
                    self.rightFlag = false; // 若rightFlag=True，则如果紧接着再chording就要把right减去1
                }
                _ => continue,
            }
        }
    }
    // pub fn reset(&self) {
    //     // 重载，暂时没用不写
    // }
}

#[pyproto]
impl PyObjectProtocol for Minesweeperboard {
    fn __getattr__(&self, name: &str) -> PyResult<usize> {
        match name {
            "left" => return Ok(self.left),
            "right" => return Ok(self.right),
            "chording" => return Ok(self.chording),
            "solved3BV" => return Ok(self.solved3BV),
            "ces" => return Ok(self.ces),
            "flag" => return Ok(self.flag),
            _ => return Ok(999),
        }
    }
}

#[pymodule]
fn ms_toollib(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(refresh_matrix, m)?)?;
    m.add_function(wrap_pyfunction!(refresh_matrixs, m)?)?;
    m.add_function(wrap_pyfunction!(cal_op, m)?)?;
    m.add_function(wrap_pyfunction!(cal3BV, m)?)?;
    m.add_function(wrap_pyfunction!(laymine_number, m)?)?;
    m.add_function(wrap_pyfunction!(refresh_board, m)?)?;
    m.add_function(wrap_pyfunction!(laymine, m)?)?;
    m.add_function(wrap_pyfunction!(solve_minus, m)?)?;
    m.add_function(wrap_pyfunction!(laymine_op_number, m)?)?;
    m.add_function(wrap_pyfunction!(laymine_op, m)?)?;
    m.add_function(wrap_pyfunction!(solve_direct, m)?)?;
    m.add_function(wrap_pyfunction!(solve_enumerate, m)?)?;
    m.add_function(wrap_pyfunction!(unsolvable_structure, m)?)?;
    m.add_function(wrap_pyfunction!(is_solvable, m)?)?;
    // m.add_function(wrap_pyfunction!(enuOneStep, m)?)?;
    m.add_function(wrap_pyfunction!(laymine_solvable, m)?)?;
    m.add_function(wrap_pyfunction!(laymine_solvable_thread, m)?)?;
    m.add_function(wrap_pyfunction!(cal_possibility, m)?)?;
    m.add_function(wrap_pyfunction!(sample_3BVs_exp, m)?)?;
    m.add_function(wrap_pyfunction!(OBR_board, m)?)?;
    m.add_function(wrap_pyfunction!(cal_possibility_onboard, m)?)?;
    m.add_class::<Minesweeperboard>()?;
    Ok(())
}


