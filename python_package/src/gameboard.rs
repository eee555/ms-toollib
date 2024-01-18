use ms_toollib_original::*;
use pyo3::prelude::*;

#[pyclass(name = "GameBoard")]
pub struct PyGameBoard {
    pub core: GameBoard,
}

impl PyGameBoard {
    pub fn set_core(&mut self, value: GameBoard) {
        self.core = value;
    }
}

#[pymethods]
impl PyGameBoard {
    #[new]
    pub fn new(mine_num: usize) -> PyGameBoard {
        let c = GameBoard::new(mine_num);
        PyGameBoard { core: c }
    }
    #[setter]
    fn set_game_board(&mut self, board: Vec<Vec<i32>>) {
        self.core.set_game_board(&board);
    }
    #[getter]
    fn get_poss(&mut self) -> PyResult<Vec<Vec<f64>>> {
        Ok(self.core.get_poss().to_vec())
    }
    #[getter]
    fn get_basic_not_mine(&mut self) -> PyResult<Vec<(usize, usize)>> {
        Ok(self.core.get_basic_not_mine().to_vec())
    }
    #[getter]
    fn get_basic_is_mine(&mut self) -> PyResult<Vec<(usize, usize)>> {
        Ok(self.core.get_basic_is_mine().to_vec())
    }
    #[getter]
    fn get_enum_not_mine(&mut self) -> PyResult<Vec<(usize, usize)>> {
        Ok(self.core.get_enum_not_mine().to_vec())
    }
    #[getter]
    fn get_enum_is_mine(&mut self) -> PyResult<Vec<(usize, usize)>> {
        Ok(self.core.get_enum_is_mine().to_vec())
    }
}

#[pyclass(name = "Board")]
pub struct PyBoard {
    pub core: Board,
}

#[pymethods]
impl PyBoard {
    #[new]
    pub fn new(board: Vec<Vec<i32>>) -> PyBoard {
        let c = Board::new(board);
        PyBoard { core: c }
    }
    #[getter]
    fn get_bbbv(&mut self) -> PyResult<usize> {
        Ok(self.core.get_bbbv())
    }
    #[getter]
    fn get_op(&mut self) -> PyResult<usize> {
        Ok(self.core.get_op())
    }
    #[getter]
    fn get_isl(&mut self) -> PyResult<usize> {
        Ok(self.core.get_isl())
    }
    #[getter]
    fn get_cell0(&mut self) -> PyResult<usize> {
        Ok(self.core.get_cell0())
    }
    #[getter]
    fn get_cell1(&mut self) -> PyResult<usize> {
        Ok(self.core.get_cell1())
    }
    #[getter]
    fn get_cell2(&mut self) -> PyResult<usize> {
        Ok(self.core.get_cell2())
    }
    #[getter]
    fn get_cell3(&mut self) -> PyResult<usize> {
        Ok(self.core.get_cell3())
    }
    #[getter]
    fn get_cell4(&mut self) -> PyResult<usize> {
        Ok(self.core.get_cell4())
    }
    #[getter]
    fn get_cell5(&mut self) -> PyResult<usize> {
        Ok(self.core.get_cell5())
    }
    #[getter]
    fn get_cell6(&mut self) -> PyResult<usize> {
        Ok(self.core.get_cell6())
    }
    #[getter]
    fn get_cell7(&mut self) -> PyResult<usize> {
        Ok(self.core.get_cell7())
    }
    #[getter]
    fn get_cell8(&mut self) -> PyResult<usize> {
        Ok(self.core.get_cell8())
    }
}
