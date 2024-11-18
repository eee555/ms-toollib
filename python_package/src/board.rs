use itertools::Itertools;
use ms_toollib_original::*;
use pyo3::prelude::*;

#[pyclass(name = "MinesweeperBoard")]
pub struct PyMinesweeperBoard {
    pub core: MinesweeperBoard<Vec<Vec<i32>>>,
}

#[pyclass(name = "SafeMinesweeperBoard")]
pub struct PySafeMinesweeperBoard {
    pub core: MinesweeperBoard<SafeBoard>,
}

#[pymethods]
impl PyMinesweeperBoard {
    #[new]
    pub fn new(board: Vec<Vec<i32>>) -> PyMinesweeperBoard {
        let c = MinesweeperBoard::<Vec<Vec<i32>>>::new(board.clone());
        PyMinesweeperBoard { core: c }
    }
    pub fn step(&mut self, e: &str, pos: (usize, usize)) {
        self.core.step(e, pos).unwrap();
    }
    pub fn reset(&mut self) {
        self.core.reset();
    }
    pub fn step_flow(&mut self, operation: Vec<(String, (usize, usize))>) {
        self.core
            .step_flow(operation.iter().map(|s| (s.0.as_str(), s.1)).collect())
            .unwrap();
    }
    // 这个方法与强可猜、弱可猜、埋雷有关
    #[setter]
    fn set_board(&mut self, board: Vec<Vec<i32>>) {
        self.core.board = board;
    }
    // #[setter]
    // fn set_game_board(&mut self, game_board: Vec<Vec<i32>>) {
    //     self.core.game_board = game_board;
    // }
    #[getter]
    fn get_board(&self) -> PyResult<Vec<Vec<i32>>> {
        Ok(self.core.board.clone())
    }
    #[getter]
    fn get_game_board(&self) -> PyResult<Vec<Vec<i32>>> {
        Ok(self.core.game_board.clone())
    }
    fn get_game_board_2(&self, mine_num: f64) -> PyResult<Vec<Vec<Vec<f64>>>> {
        // 返回用于强化学习的局面，即状态
        let mut game_board_clone = self.core.game_board.clone();
        let t_1: Vec<Vec<f64>> = game_board_clone
            .iter()
            .map(|x| {
                x.iter()
                    .map(|x| {
                        if *x == 10 {
                            return -1;
                        } else if *x == 11 {
                            return -2;
                        } else {
                            return *x;
                        }
                    })
                    .map(|y| y as f64)
                    .collect::<Vec<f64>>()
            })
            .collect_vec();
        // 把玩家或ai标的错的雷都删了
        game_board_clone.iter_mut().for_each(|x| {
            x.iter_mut().for_each(|x| {
                if *x > 10 {
                    *x = 10
                }
            })
        });
        mark_board(&mut game_board_clone);
        let (t_2, _) = cal_possibility_onboard(&game_board_clone, mine_num).unwrap();
        let t = vec![t_1, t_2];
        Ok(t)
    }
    #[getter]
    fn get_left(&self) -> PyResult<usize> {
        Ok(self.core.left)
    }
    #[getter]
    fn get_right(&self) -> PyResult<usize> {
        Ok(self.core.right)
    }
    #[getter]
    fn get_double(&self) -> PyResult<usize> {
        Ok(self.core.double)
    }
    #[getter]
    fn get_lce(&self) -> PyResult<usize> {
        Ok(self.core.lce)
    }
    #[getter]
    fn get_rce(&self) -> PyResult<usize> {
        Ok(self.core.rce)
    }
    #[getter]
    fn get_dce(&self) -> PyResult<usize> {
        Ok(self.core.dce)
    }
    #[getter]
    fn get_ce(&self) -> PyResult<usize> {
        Ok(self.core.lce + self.core.rce + self.core.dce)
    }
    #[getter]
    fn get_flag(&self) -> PyResult<usize> {
        Ok(self.core.flag)
    }
    #[getter]
    fn get_bbbv_solved(&self) -> PyResult<usize> {
        Ok(self.core.bbbv_solved)
    }
    #[getter]
    fn get_row(&self) -> PyResult<usize> {
        Ok(self.core.row)
    }
    #[getter]
    fn get_column(&self) -> PyResult<usize> {
        Ok(self.core.column)
    }
    #[getter]
    fn get_game_board_state(&self) -> PyResult<usize> {
        match self.core.game_board_state {
            GameBoardState::Ready => Ok(1),
            GameBoardState::Playing => Ok(2),
            GameBoardState::Win => Ok(3),
            GameBoardState::Loss => Ok(4),
            GameBoardState::PreFlaging => Ok(5),
            GameBoardState::Display => Ok(6),
        }
    }
    #[getter]
    fn get_mouse_state(&self) -> PyResult<usize> {
        match self.core.mouse_state {
            MouseState::UpUp => Ok(1),
            MouseState::UpDown => Ok(2),
            MouseState::UpDownNotFlag => Ok(3),
            MouseState::DownUp => Ok(4),
            MouseState::Chording => Ok(5),
            MouseState::ChordingNotFlag => Ok(6),
            MouseState::DownUpAfterChording => Ok(7),
            MouseState::Undefined => Ok(8),
        }
    }
}

#[pymethods]
impl PySafeMinesweeperBoard {
    #[new]
    pub fn new(board: Vec<Vec<i32>>) -> PySafeMinesweeperBoard {
        let c = MinesweeperBoard::<SafeBoard>::new(SafeBoard::new(board));
        PySafeMinesweeperBoard { core: c }
    }
    pub fn step(&mut self, e: &str, pos: (usize, usize)) {
        self.core.step(e, pos).unwrap();
    }
    // pub fn reset(&mut self) {
    //     self.core.reset();
    // }
    pub fn step_flow(&mut self, operation: Vec<(String, (usize, usize))>) {
        self.core
            .step_flow(operation.iter().map(|s| (s.0.as_str(), s.1)).collect())
            .unwrap();
    }
    // 这个方法与强可猜、弱可猜、埋雷有关
    #[setter]
    fn set_board(&mut self, board: Vec<Vec<i32>>) {
        self.core.board.set(board);
    }
    // #[setter]
    // fn set_game_board(&mut self, game_board: Vec<Vec<i32>>) {
    //     self.core.game_board = game_board;
    // }
    #[getter]
    fn get_board(&self) -> PyResult<Vec<Vec<i32>>> {
        Ok(self.core.board.into_vec_vec())
    }
    #[getter]
    fn get_game_board(&self) -> PyResult<Vec<Vec<i32>>> {
        Ok(self.core.game_board.clone())
    }
    fn get_game_board_2(&self, mine_num: f64) -> PyResult<Vec<Vec<Vec<f64>>>> {
        // 返回用于强化学习的局面，即状态
        let mut game_board_clone = self.core.game_board.clone();
        let t_1: Vec<Vec<f64>> = game_board_clone
            .iter()
            .map(|x| {
                x.iter()
                    .map(|x| {
                        if *x == 10 {
                            return -1;
                        } else if *x == 11 {
                            return -2;
                        } else {
                            return *x;
                        }
                    })
                    .map(|y| y as f64)
                    .collect::<Vec<f64>>()
            })
            .collect_vec();
        // 把玩家或ai标的错的雷都删了
        game_board_clone.iter_mut().for_each(|x| {
            x.iter_mut().for_each(|x| {
                if *x > 10 {
                    *x = 10
                }
            })
        });
        mark_board(&mut game_board_clone);
        let (t_2, _) = cal_possibility_onboard(&game_board_clone, mine_num).unwrap();
        let t = vec![t_1, t_2];
        Ok(t)
    }
    #[getter]
    fn get_left(&self) -> PyResult<usize> {
        Ok(self.core.left)
    }
    #[getter]
    fn get_right(&self) -> PyResult<usize> {
        Ok(self.core.right)
    }
    #[getter]
    fn get_double(&self) -> PyResult<usize> {
        Ok(self.core.double)
    }
    #[getter]
    fn get_lce(&self) -> PyResult<usize> {
        Ok(self.core.lce)
    }
    #[getter]
    fn get_rce(&self) -> PyResult<usize> {
        Ok(self.core.rce)
    }
    #[getter]
    fn get_dce(&self) -> PyResult<usize> {
        Ok(self.core.dce)
    }
    #[getter]
    fn get_ce(&self) -> PyResult<usize> {
        Ok(self.core.lce + self.core.rce + self.core.dce)
    }
    #[getter]
    fn get_flag(&self) -> PyResult<usize> {
        Ok(self.core.flag)
    }
    #[getter]
    fn get_bbbv_solved(&self) -> PyResult<usize> {
        Ok(self.core.bbbv_solved)
    }
    #[getter]
    fn get_row(&self) -> PyResult<usize> {
        Ok(self.core.row)
    }
    #[getter]
    fn get_column(&self) -> PyResult<usize> {
        Ok(self.core.column)
    }
    #[getter]
    fn get_game_board_state(&self) -> PyResult<usize> {
        match self.core.game_board_state {
            GameBoardState::Ready => Ok(1),
            GameBoardState::Playing => Ok(2),
            GameBoardState::Win => Ok(3),
            GameBoardState::Loss => Ok(4),
            GameBoardState::PreFlaging => Ok(5),
            GameBoardState::Display => Ok(6),
        }
    }
    #[getter]
    fn get_mouse_state(&self) -> PyResult<usize> {
        match self.core.mouse_state {
            MouseState::UpUp => Ok(1),
            MouseState::UpDown => Ok(2),
            MouseState::UpDownNotFlag => Ok(3),
            MouseState::DownUp => Ok(4),
            MouseState::Chording => Ok(5),
            MouseState::ChordingNotFlag => Ok(6),
            MouseState::DownUpAfterChording => Ok(7),
            MouseState::Undefined => Ok(8),
        }
    }
}
