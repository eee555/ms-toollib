use crate::PyGameBoard;
use itertools::Itertools;
use ms_toollib_original::*;
use pyo3::prelude::*;
use pyo3::*;
use ms_toollib_original::videos::base_video::NewBaseVideo2;

#[pyclass(name = "SafeBoardRow")]
pub struct PySafeBoardRow {
    pub core: SafeBoardRow,
}

#[pymethods]
impl PySafeBoardRow {
    #[new]
    pub fn new(row: Vec<i32>) -> PySafeBoardRow {
        PySafeBoardRow { core: SafeBoardRow::new(row) }
    }
    fn __getitem__(&self, key: isize) -> PyResult<i32> {
        Ok(self.core[key as usize])
    }
}

#[pyclass]
#[pyo3(name = "SafeBoard", subclass)]
pub struct PySafeBoard {
    pub core: SafeBoard,
}

#[pymethods]
impl PySafeBoard {
    #[new]
    pub fn new(board: Vec<Vec<i32>>) -> PySafeBoard {
        let c = SafeBoard::new(board);
        PySafeBoard { core: c }
    }
    pub fn into_vec_vec(&self) -> PyResult<Vec<Vec<i32>>> {
        Ok(self.core.into_vec_vec())
    }
    pub fn set(&mut self, board: Vec<Vec<i32>>) {
        self.core.set(board);
    }
    fn __getitem__(&self, key: isize) -> PyResult<PySafeBoardRow> {
        Ok(PySafeBoardRow::new(self.core[key as usize].into_vec()))
    }
}

// #[pyproto]
// impl PySequenceProtocol for PySafeBoardRow {
//     fn __getitem__(&self, key: isize) -> PyResult<i32> {
//         Ok(self.core[key as usize])
//     }
// }

// #[pyproto]
// impl PySequenceProtocol for PySafeBoard {
//     fn __getitem__(&self, key: isize) -> PyResult<PySafeBoardRow> {
//         Ok(PySafeBoardRow::new(self.core[key as usize].into_vec()))
//     }
// }

#[pyclass]
#[pyo3(name = "BaseVideo", subclass)]
pub struct PyBaseVideo {
    pub core: BaseVideo<SafeBoard>,
}

#[pymethods]
impl PyBaseVideo {
    #[new]
    pub fn new(board: Vec<Vec<i32>>, cell_pixel_size: u8) -> PyBaseVideo {
        let c = BaseVideo::<SafeBoard>::new(board, cell_pixel_size);
        PyBaseVideo { core: c }
    }
    // pub fn analyse(&mut self) {
    //     self.core.analyse();
    // }
    // pub fn analyse_for_features(&mut self, controller: Vec<&str>) {
    //     self.core.analyse_for_features(controller);
    // }
    pub fn generate_evf_v0_raw_data(&mut self) {
        self.core.generate_evf_v0_raw_data();
    }
    pub fn generate_evf_v2_raw_data(&mut self) {
        self.core.generate_evf_v2_raw_data();
    }
    pub fn generate_evf_v3_raw_data(&mut self) {
        self.core.generate_evf_v3_raw_data();
    }
    pub fn save_to_evf_file(&self, file_name: &str) {
        self.core.save_to_evf_file(file_name);
    }
    pub fn step(&mut self, e: &str, pos: (usize, usize)) {
        // println!("{:?}: '{:?}', ({:?}, {:?})", self.core.get_time(), e, pos.0, pos.1);
        self.core.step(e, pos).unwrap();
    }
    pub fn reset(&mut self, row: usize, column: usize, pix_size: u8) {
        self.core.reset(row, column, pix_size);
    }
    pub fn win_then_flag_all_mine(&mut self) {
        self.core.win_then_flag_all_mine();
    }
    pub fn loss_then_open_all_mine(&mut self) {
        self.core.loss_then_open_all_mine();
    }
    #[getter]
    fn get_raw_data(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.get_raw_data().unwrap())
    }
    #[getter]
    fn get_time(&self) -> PyResult<f64> {
        Ok(self.core.get_time())
    }
    #[getter]
    fn get_software(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.software.clone())
    }
    #[getter]
    fn get_row(&self) -> PyResult<usize> {
        Ok(self.core.height)
    }
    #[getter]
    fn get_column(&self) -> PyResult<usize> {
        Ok(self.core.width)
    }
    #[getter]
    fn get_level(&self) -> PyResult<u8> {
        Ok(self.core.level)
    }
    #[getter]
    fn get_mode(&self) -> PyResult<u16> {
        Ok(self.core.mode)
    }
    #[getter]
    fn get_is_completed(&self) -> PyResult<bool> {
        Ok(self.core.is_completed)
    }
    #[getter]
    fn get_is_official(&self) -> PyResult<bool> {
        Ok(self.core.is_official)
    }
    #[getter]
    fn get_is_fair(&self) -> PyResult<bool> {
        Ok(self.core.is_fair)
    }
    #[getter]
    fn get_mine_num(&self) -> PyResult<usize> {
        Ok(self.core.mine_num)
    }
    #[getter]
    fn get_player_identifier(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.player_identifier.clone())
    }
    #[getter]
    fn get_race_identifier(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.race_identifier.clone())
    }
    #[getter]
    fn get_uniqueness_identifier(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.uniqueness_identifier.clone())
    }
    #[getter]
    fn get_country(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.country.clone())
    }
    #[getter]
    fn get_bbbv(&self) -> PyResult<usize> {
        Ok(self.core.static_params.bbbv)
    }
    #[getter]
    fn get_start_time(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.start_time.clone())
    }
    #[getter]
    fn get_end_time(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.end_time.clone())
    }
    #[getter]
    fn get_op(&self) -> PyResult<usize> {
        Ok(self.core.static_params.op)
    }
    #[getter]
    fn get_isl(&self) -> PyResult<usize> {
        Ok(self.core.static_params.isl)
    }
    #[getter]
    fn get_hizi(&self) -> PyResult<usize> {
        Ok(self.core.static_params.hizi)
    }
    #[getter]
    fn get_cell0(&self) -> PyResult<usize> {
        Ok(self.core.static_params.cell0)
    }
    #[getter]
    fn get_cell1(&self) -> PyResult<usize> {
        Ok(self.core.static_params.cell1)
    }
    #[getter]
    fn get_cell2(&self) -> PyResult<usize> {
        Ok(self.core.static_params.cell2)
    }
    #[getter]
    fn get_cell3(&self) -> PyResult<usize> {
        Ok(self.core.static_params.cell3)
    }
    #[getter]
    fn get_cell4(&self) -> PyResult<usize> {
        Ok(self.core.static_params.cell4)
    }
    #[getter]
    fn get_cell5(&self) -> PyResult<usize> {
        Ok(self.core.static_params.cell5)
    }
    #[getter]
    fn get_cell6(&self) -> PyResult<usize> {
        Ok(self.core.static_params.cell6)
    }
    #[getter]
    fn get_cell7(&self) -> PyResult<usize> {
        Ok(self.core.static_params.cell7)
    }
    #[getter]
    fn get_cell8(&self) -> PyResult<usize> {
        Ok(self.core.static_params.cell8)
    }
    #[getter]
    fn get_rtime(&self) -> PyResult<f64> {
        Ok(self.core.get_rtime().unwrap())
    }
    #[getter]
    fn get_rtime_ms(&self) -> PyResult<u32> {
        Ok(self.core.get_rtime_ms().unwrap())
    }
    #[getter]
    fn get_etime(&self) -> PyResult<f64> {
        Ok(self.core.get_etime().unwrap())
    }
    #[getter]
    pub fn get_video_start_time(&self) -> PyResult<f64> {
        Ok(self.core.get_video_start_time().unwrap())
    }
    #[getter]
    pub fn get_video_end_time(&self) -> PyResult<f64> {
        Ok(self.core.get_video_end_time().unwrap())
    }
    #[getter]
    fn get_bbbv_s(&self) -> PyResult<f64> {
        Ok(self.core.get_bbbv_s().unwrap())
    }
    #[getter]
    fn get_stnb(&self) -> PyResult<f64> {
        Ok(self.core.get_stnb().unwrap())
    }
    #[getter]
    fn get_rqp(&self) -> PyResult<f64> {
        Ok(self.core.get_rqp().unwrap())
    }
    #[getter]
    fn get_left(&self) -> PyResult<usize> {
        Ok(self.core.get_left())
    }
    #[getter]
    fn get_right(&self) -> PyResult<usize> {
        Ok(self.core.get_right())
    }
    #[getter]
    fn get_double(&self) -> PyResult<usize> {
        Ok(self.core.get_double())
    }
    #[getter]
    fn get_cl(&self) -> PyResult<usize> {
        Ok(self.core.get_cl())
    }
    #[getter]
    fn get_flag(&self) -> PyResult<usize> {
        Ok(self.core.get_flag())
    }
    #[getter]
    fn get_bbbv_solved(&self) -> PyResult<usize> {
        Ok(self.core.get_bbbv_solved().unwrap())
    }
    #[getter]
    fn get_lce(&self) -> PyResult<usize> {
        Ok(self.core.get_lce().unwrap())
    }
    #[getter]
    fn get_rce(&self) -> PyResult<usize> {
        Ok(self.core.get_rce().unwrap())
    }
    #[getter]
    fn get_dce(&self) -> PyResult<usize> {
        Ok(self.core.get_dce().unwrap())
    }
    #[getter]
    fn get_ce(&self) -> PyResult<usize> {
        Ok(self.core.get_ce().unwrap())
    }
    #[getter]
    fn get_left_s(&self) -> PyResult<f64> {
        Ok(self.core.get_left_s())
    }
    #[getter]
    fn get_right_s(&self) -> PyResult<f64> {
        Ok(self.core.get_right_s())
    }
    #[getter]
    fn get_double_s(&self) -> PyResult<f64> {
        Ok(self.core.get_double_s())
    }
    #[getter]
    fn get_cl_s(&self) -> PyResult<f64> {
        Ok(self.core.get_cl_s())
    }
    #[getter]
    fn get_flag_s(&self) -> PyResult<f64> {
        Ok(self.core.get_flag_s())
    }
    #[getter]
    fn get_path(&self) -> PyResult<f64> {
        Ok(self.core.get_path())
    }
    #[getter]
    fn get_ce_s(&self) -> PyResult<f64> {
        Ok(self.core.get_ce_s().unwrap())
    }
    #[getter]
    fn get_ioe(&self) -> PyResult<f64> {
        Ok(self.core.get_ioe().unwrap())
    }
    #[getter]
    fn get_thrp(&self) -> PyResult<f64> {
        Ok(self.core.get_thrp().unwrap())
    }
    #[getter]
    fn get_corr(&self) -> PyResult<f64> {
        Ok(self.core.get_corr().unwrap())
    }
    #[getter]
    fn get_events_len(&self) -> PyResult<usize> {
        Ok(self.core.video_action_state_recorder.len())
    }
    pub fn events_time(&self, index: usize) -> PyResult<f64> {
        Ok(self.core.video_action_state_recorder[index].time)
    }
    pub fn events_mouse(&self, index: usize) -> PyResult<String> {
        Ok(self.core.video_action_state_recorder[index].mouse.clone())
    }
    pub fn events_x(&self, index: usize) -> PyResult<u16> {
        Ok(self.core.video_action_state_recorder[index].x)
    }
    pub fn events_y(&self, index: usize) -> PyResult<u16> {
        Ok(self.core.video_action_state_recorder[index].y)
    }
    pub fn events_useful_level(&self, index: usize) -> PyResult<u8> {
        Ok(self.core.video_action_state_recorder[index].useful_level)
    }
    pub fn events_prior_game_board(&self, index: usize) -> PyResult<PyGameBoard> {
        let mut t = PyGameBoard::new(self.core.mine_num);
        t.set_core(
            self.core.game_board_stream
                [self.core.video_action_state_recorder[index].prior_game_board_id]
                .clone(),
        );
        Ok(t)
    }
    pub fn events_comments(&self, index: usize) -> PyResult<String> {
        Ok(self.core.video_action_state_recorder[index]
            .comments
            .clone())
    }
    pub fn events_mouse_state(&self, index: usize) -> PyResult<usize> {
        match self.core.video_action_state_recorder[index].mouse_state {
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
    #[getter]
    pub fn get_current_event_id(&self) -> PyResult<usize> {
        Ok(self.core.current_event_id)
    }
    #[setter]
    pub fn set_current_event_id(&mut self, id: usize) {
        self.core.current_event_id = id
    }
    // 这个方法与强可猜、弱可猜、埋雷有关
    #[setter]
    pub fn set_board(&mut self, board: Vec<Vec<i32>>) {
        self.core.set_board(board).unwrap();
    }
    #[getter]
    fn get_board(&self) -> PyResult<PySafeBoard> {
        let t = PySafeBoard::new(self.core.minesweeper_board.board.into_vec_vec());
        Ok(t)
    }
    #[getter]
    pub fn get_game_board(&self) -> PyResult<Vec<Vec<i32>>> {
        Ok(self.core.get_game_board())
    }
    #[getter]
    pub fn get_game_board_poss(&mut self) -> PyResult<Vec<Vec<f64>>> {
        Ok(self.core.get_game_board_poss())
    }
    #[getter]
    pub fn get_mouse_state(&self) -> PyResult<usize> {
        Ok(self.core.get_mouse_state())
    }
    /// 局面状态
    #[getter]
    pub fn get_game_board_state(&self) -> PyResult<usize> {
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
    pub fn get_x_y(&self) -> PyResult<(u16, u16)> {
        Ok(self.core.get_x_y().unwrap())
    }
    #[getter]
    pub fn get_checksum(&self) -> PyResult<[u8; 32]> {
        Ok(self.core.get_checksum().unwrap())
    }
    #[setter]
    pub fn set_video_playing_pix_size(&mut self, pix_size: u8) {
        self.core.set_video_playing_pix_size(pix_size);
    }
    #[setter]
    pub fn set_current_time(&mut self, time: f64) {
        self.core.set_current_time(time);
    }
    #[setter]
    pub fn set_use_question(&mut self, use_question: bool) {
        self.core.set_use_question(use_question).unwrap();
    }
    #[setter]
    pub fn set_use_cursor_pos_lim(&mut self, use_cursor_pos_lim: bool) {
        self.core.set_use_cursor_pos_lim(use_cursor_pos_lim).unwrap();
    }
    #[setter]
    pub fn set_use_auto_replay(&mut self, use_auto_replay: bool) {
        self.core.set_use_auto_replay(use_auto_replay).unwrap();
    }
    #[setter]
    pub fn set_is_official(&mut self, is_official: bool) {
        self.core.set_is_official(is_official).unwrap();
    }
    #[setter]
    pub fn set_is_fair(&mut self, is_fair: bool) {
        self.core.set_is_fair(is_fair).unwrap();
    }
    #[setter]
    pub fn set_mode(&mut self, mode: u16) {
        self.core.set_mode(mode).unwrap();
    }
    #[setter]
    pub fn set_software(&mut self, software: Vec<u8>) {
        self.core.set_software(software).unwrap();
    }
    #[setter]
    pub fn set_player_identifier(&mut self, player_identifier: Vec<u8>) {
        self.core.set_player_identifier(player_identifier).unwrap();
    }
    #[setter]
    pub fn set_race_identifier(&mut self, race_identifier: Vec<u8>) {
        self.core.set_race_identifier(race_identifier).unwrap();
    }
    #[setter]
    pub fn set_uniqueness_identifier(&mut self, uniqueness_identifier: Vec<u8>) {
        self.core
            .set_uniqueness_identifier(uniqueness_identifier)
            .unwrap();
    }
    #[setter]
    pub fn set_country(&mut self, country: Vec<u8>) {
        self.core.set_country(country).unwrap();
    }
    #[setter]
    pub fn set_device_uuid(&mut self, device_uuid: Vec<u8>) {
        self.core.set_device_uuid(device_uuid).unwrap();
    }
    #[setter]
    pub fn set_checksum(&mut self, checksum: [u8; 32]) {
        self.core.set_checksum(checksum).unwrap();
    }
    #[setter]
    pub fn set_pix_size(&mut self, pix_size: u8) {
        self.core.set_pix_size(pix_size).unwrap();
    }
}
