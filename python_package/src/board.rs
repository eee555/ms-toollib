use ms_toollib::*;
use pyo3::prelude::*;

#[pyclass]
pub struct PyMinesweeperBoard {
    /// 局面
    pub game_board: Vec<Vec<i32>>,
    /// 左键数
    pub left: usize,
    /// 右键数
    pub right: usize,
    /// 双击数
    pub chording: usize,
    /// ce数
    pub ces: usize,
    /// 标雷数
    pub flag: usize,
    /// 已解决的3BV数
    pub solved3BV: usize,
    pub core: Box<MinesweeperBoard>,
}

#[pymethods]
impl PyMinesweeperBoard {
    #[new]
    pub fn new(board: Vec<Vec<i32>>) -> PyMinesweeperBoard {
        let row = board.len();
        let column = board[0].len();
        MinesweeperBoard {
            board: board,
            row: row,
            column: column,
            gameBoard: vec![vec![10; column]; row],
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
}







