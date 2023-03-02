use itertools::Itertools;
use ms_toollib::*;
use pyo3::prelude::*;

#[pyclass(name = "MinesweeperBoard")]
pub struct PyMinesweeperBoard {
    pub core: MinesweeperBoard,
}

#[pymethods]
impl PyMinesweeperBoard {
    #[new]
    pub fn new(board: Vec<Vec<i32>>) -> PyMinesweeperBoard {
        let c = MinesweeperBoard::new(board.clone());
        PyMinesweeperBoard { core: c }
    }
    pub fn step(&mut self, e: &str, pos: (usize, usize)) {
        self.core.step(e, pos).unwrap();
    }
    pub fn reset(&mut self) {
        self.core.reset();
    }
    pub fn step_flow(&mut self, operation: Vec<(&str, (usize, usize))>) {
        self.core.step_flow(operation).unwrap();
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
    fn get_ce(&self) -> PyResult<usize> {
        Ok(self.core.ce)
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

#[pyclass(name = "AvfVideo")]
pub struct PyAvfVideo {
    pub core: AvfVideo,
}

#[pymethods]
impl PyAvfVideo {
    #[new]
    pub fn new(file_name: &str) -> PyAvfVideo {
        let c = AvfVideo::new(file_name);
        PyAvfVideo { core: c }
    }
    pub fn parse_video(&mut self) {
        self.core.parse_video().unwrap();
    }
    pub fn analyse(&mut self) {
        self.core.data.analyse();
    }
    pub fn analyse_for_features(&mut self, controller: Vec<&str>) {
        self.core.data.analyse_for_features(controller);
    }
    pub fn generate_evf_v0_raw_data(&mut self) {
        self.core.data.generate_evf_v0_raw_data();
    }
    pub fn save_to_evf_file(&self, file_name: &str) {
        self.core.data.save_to_evf_file(file_name);
    }
    #[getter]
    fn get_time(&self) -> PyResult<f64> {
        Ok(self.core.data.get_time())
    }
    #[getter]
    fn get_software(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.software.clone())
    }
    #[getter]
    fn get_row(&self) -> PyResult<usize> {
        Ok(self.core.data.height)
    }
    #[getter]
    fn get_column(&self) -> PyResult<usize> {
        Ok(self.core.data.width)
    }
    #[getter]
    fn get_level(&self) -> PyResult<u8> {
        Ok(self.core.data.level)
    }
    #[getter]
    fn get_mode(&self) -> PyResult<u16> {
        Ok(self.core.data.mode)
    }
    #[getter]
    fn get_is_completed(&self) -> PyResult<bool> {
        Ok(self.core.data.is_completed)
    }
    #[getter]
    fn get_is_offical(&self) -> PyResult<bool> {
        Ok(self.core.data.is_offical)
    }
    #[getter]
    fn get_is_fair(&self) -> PyResult<bool> {
        Ok(self.core.data.is_fair)
    }
    #[getter]
    fn get_mine_num(&self) -> PyResult<usize> {
        Ok(self.core.data.mine_num)
    }
    #[getter]
    fn get_player_designator(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.player_designator.clone())
    }
    #[getter]
    fn get_race_designator(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.race_designator.clone())
    }
    #[getter]
    fn get_uniqueness_designator(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.uniqueness_designator.clone())
    }
    #[getter]
    fn get_country(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.country.clone())
    }
    #[getter]
    fn get_bbbv(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.bbbv)
    }
    #[getter]
    fn get_start_time(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.start_time.clone())
    }
    #[getter]
    fn get_end_time(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.end_time.clone())
    }
    #[getter]
    fn get_op(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.op)
    }
    #[getter]
    fn get_isl(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.isl)
    }
    #[getter]
    fn get_hizi(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.hizi)
    }
    #[getter]
    fn get_cell0(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell0)
    }
    #[getter]
    fn get_cell1(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell1)
    }
    #[getter]
    fn get_cell2(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell2)
    }
    #[getter]
    fn get_cell3(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell3)
    }
    #[getter]
    fn get_cell4(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell4)
    }
    #[getter]
    fn get_cell5(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell5)
    }
    #[getter]
    fn get_cell6(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell6)
    }
    #[getter]
    fn get_cell7(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell7)
    }
    #[getter]
    fn get_cell8(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell8)
    }
    #[getter]
    fn get_rtime(&self) -> PyResult<f64> {
        Ok(self.core.data.get_rtime().unwrap())
    }
    #[getter]
    fn get_rtime_ms(&self) -> PyResult<u32> {
        Ok(self.core.data.get_rtime_ms().unwrap())
    }
    #[getter]
    fn get_etime(&self) -> PyResult<f64> {
        Ok(self.core.data.get_etime().unwrap())
    }
    #[getter]
    pub fn get_video_time(&self) -> PyResult<f64> {
        Ok(self.core.data.get_video_time().unwrap())
    }
    #[getter]
    fn get_bbbv_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_bbbv_s().unwrap())
    }
    #[getter]
    fn get_stnb(&self) -> PyResult<f64> {
        Ok(self.core.data.get_stnb().unwrap())
    }
    #[getter]
    fn get_rqp(&self) -> PyResult<f64> {
        Ok(self.core.data.get_rqp().unwrap())
    }
    #[getter]
    fn get_left(&self) -> PyResult<usize> {
        Ok(self.core.data.get_left())
    }
    #[getter]
    fn get_right(&self) -> PyResult<usize> {
        Ok(self.core.data.get_right())
    }
    #[getter]
    fn get_double(&self) -> PyResult<usize> {
        Ok(self.core.data.get_double())
    }
    #[getter]
    fn get_cl(&self) -> PyResult<usize> {
        Ok(self.core.data.get_cl())
    }
    #[getter]
    fn get_flag(&self) -> PyResult<usize> {
        Ok(self.core.data.get_flag())
    }
    #[getter]
    fn get_bbbv_solved(&self) -> PyResult<usize> {
        Ok(self.core.data.get_bbbv_solved().unwrap())
    }
    #[getter]
    fn get_ce(&self) -> PyResult<usize> {
        Ok(self.core.data.get_ce().unwrap())
    }
    #[getter]
    fn get_left_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_left_s())
    }
    #[getter]
    fn get_right_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_right_s())
    }
    #[getter]
    fn get_double_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_double_s())
    }
    #[getter]
    fn get_cl_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_cl_s())
    }
    #[getter]
    fn get_flag_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_flag_s())
    }
    #[getter]
    fn get_path(&self) -> PyResult<f64> {
        Ok(self.core.data.get_path())
    }
    #[getter]
    fn get_ce_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_ce_s().unwrap())
    }
    #[getter]
    fn get_ioe(&self) -> PyResult<f64> {
        Ok(self.core.data.get_ioe().unwrap())
    }
    #[getter]
    fn get_thrp(&self) -> PyResult<f64> {
        Ok(self.core.data.get_thrp().unwrap())
    }
    #[getter]
    fn get_corr(&self) -> PyResult<f64> {
        Ok(self.core.data.get_corr().unwrap())
    }

    #[getter]
    fn get_events_len(&self) -> PyResult<usize> {
        Ok(self.core.data.video_action_state_recorder.len())
    }
    pub fn events_time(&self, index: usize) -> PyResult<f64> {
        Ok(self.core.data.video_action_state_recorder[index].time)
    }
    pub fn events_mouse(&self, index: usize) -> PyResult<String> {
        Ok(self.core.data.video_action_state_recorder[index]
            .mouse
            .clone())
    }
    pub fn events_x(&self, index: usize) -> PyResult<u16> {
        Ok(self.core.data.video_action_state_recorder[index].x)
    }
    pub fn events_y(&self, index: usize) -> PyResult<u16> {
        Ok(self.core.data.video_action_state_recorder[index].y)
    }
    pub fn events_useful_level(&self, index: usize) -> PyResult<u8> {
        Ok(self.core.data.video_action_state_recorder[index].useful_level)
    }
    pub fn events_prior_game_board(&self, index: usize) -> PyResult<PyGameBoard> {
        let mut t = PyGameBoard::new(self.core.data.mine_num);
        t.set_core(
            self.core.data.game_board_stream
                [self.core.data.video_action_state_recorder[index].prior_game_board_id]
                .clone(),
        );
        Ok(t)
    }
    pub fn events_comments(&self, index: usize) -> PyResult<String> {
        Ok(self.core.data.video_action_state_recorder[index]
            .comments
            .clone())
    }
    pub fn events_mouse_state(&self, index: usize) -> PyResult<usize> {
        match self.core.data.video_action_state_recorder[index].mouse_state {
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
        Ok(self.core.data.current_event_id)
    }
    #[setter]
    pub fn set_current_event_id(&mut self, id: usize) {
        self.core.data.current_event_id = id
    }
    #[getter]
    pub fn get_game_board(&self) -> PyResult<Vec<Vec<i32>>> {
        Ok(self.core.data.get_game_board())
    }
    #[getter]
    pub fn get_game_board_poss(&mut self) -> PyResult<Vec<Vec<f64>>> {
        Ok(self.core.data.get_game_board_poss())
    }
    #[getter]
    pub fn get_mouse_state(&self) -> PyResult<usize> {
        match self.core.data.video_action_state_recorder[self.core.data.current_event_id]
            .mouse_state
        {
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
    /// 局面状态（录像播放器的局面状态始终等于1，没有ready、win、fail的概念）
    #[getter]
    pub fn get_game_board_state(&self) -> PyResult<usize> {
        Ok(1)
    }
    #[getter]
    pub fn get_x_y(&self) -> PyResult<(u16, u16)> {
        Ok(self.core.data.get_x_y().unwrap())
    }
    #[setter]
    pub fn set_video_playing_pix_size(&mut self, pix_size: u8) {
        self.core.data.set_video_playing_pix_size(pix_size);
    }
    #[setter]
    pub fn set_current_time(&mut self, time: f64) {
        self.core.data.set_current_time(time);
    }
}

#[pyclass(name = "RmvVideo")]
pub struct PyRmvVideo {
    pub core: RmvVideo,
}

#[pymethods]
impl PyRmvVideo {
    #[new]
    pub fn new(file_name: &str) -> PyRmvVideo {
        let c = RmvVideo::new(file_name);
        PyRmvVideo { core: c }
    }
    pub fn parse_video(&mut self) {
        self.core.parse_video().unwrap();
    }
    pub fn analyse(&mut self) {
        self.core.data.analyse();
    }
    pub fn analyse_for_features(&mut self, controller: Vec<&str>) {
        self.core.data.analyse_for_features(controller);
    }
    pub fn generate_evf_v0_raw_data(&mut self) {
        self.core.data.generate_evf_v0_raw_data();
    }
    pub fn save_to_evf_file(&self, file_name: &str) {
        self.core.data.save_to_evf_file(file_name);
    }
    #[getter]
    fn get_time(&self) -> PyResult<f64> {
        Ok(self.core.data.get_time())
    }
    #[getter]
    fn get_software(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.software.clone())
    }
    #[getter]
    fn get_row(&self) -> PyResult<usize> {
        Ok(self.core.data.height)
    }
    #[getter]
    fn get_column(&self) -> PyResult<usize> {
        Ok(self.core.data.width)
    }
    #[getter]
    fn get_level(&self) -> PyResult<u8> {
        Ok(self.core.data.level)
    }
    #[getter]
    fn get_mode(&self) -> PyResult<u16> {
        Ok(self.core.data.mode)
    }
    #[getter]
    fn get_is_completed(&self) -> PyResult<bool> {
        Ok(self.core.data.is_completed)
    }
    #[getter]
    fn get_is_offical(&self) -> PyResult<bool> {
        Ok(self.core.data.is_offical)
    }
    #[getter]
    fn get_is_fair(&self) -> PyResult<bool> {
        Ok(self.core.data.is_fair)
    }
    #[getter]
    fn get_mine_num(&self) -> PyResult<usize> {
        Ok(self.core.data.mine_num)
    }
    #[getter]
    fn get_player_designator(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.player_designator.clone())
    }
    #[getter]
    fn get_race_designator(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.race_designator.clone())
    }
    #[getter]
    fn get_uniqueness_designator(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.uniqueness_designator.clone())
    }
    #[getter]
    fn get_country(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.country.clone())
    }
    #[getter]
    fn get_bbbv(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.bbbv)
    }
    #[getter]
    fn get_start_time(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.start_time.clone())
    }
    #[getter]
    fn get_end_time(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.end_time.clone())
    }
    #[getter]
    fn get_op(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.op)
    }
    #[getter]
    fn get_isl(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.isl)
    }
    #[getter]
    fn get_hizi(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.hizi)
    }
    #[getter]
    fn get_cell0(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell0)
    }
    #[getter]
    fn get_cell1(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell1)
    }
    #[getter]
    fn get_cell2(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell2)
    }
    #[getter]
    fn get_cell3(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell3)
    }
    #[getter]
    fn get_cell4(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell4)
    }
    #[getter]
    fn get_cell5(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell5)
    }
    #[getter]
    fn get_cell6(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell6)
    }
    #[getter]
    fn get_cell7(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell7)
    }
    #[getter]
    fn get_cell8(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell8)
    }
    #[getter]
    fn get_rtime(&self) -> PyResult<f64> {
        Ok(self.core.data.get_rtime().unwrap())
    }
    #[getter]
    fn get_rtime_ms(&self) -> PyResult<u32> {
        Ok(self.core.data.get_rtime_ms().unwrap())
    }
    #[getter]
    fn get_etime(&self) -> PyResult<f64> {
        Ok(self.core.data.get_etime().unwrap())
    }
    #[getter]
    pub fn get_video_time(&self) -> PyResult<f64> {
        Ok(self.core.data.get_video_time().unwrap())
    }
    #[getter]
    fn get_bbbv_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_bbbv_s().unwrap())
    }
    #[getter]
    fn get_stnb(&self) -> PyResult<f64> {
        Ok(self.core.data.get_stnb().unwrap())
    }
    #[getter]
    fn get_rqp(&self) -> PyResult<f64> {
        Ok(self.core.data.get_rqp().unwrap())
    }
    #[getter]
    fn get_left(&self) -> PyResult<usize> {
        Ok(self.core.data.get_left())
    }
    #[getter]
    fn get_right(&self) -> PyResult<usize> {
        Ok(self.core.data.get_right())
    }
    #[getter]
    fn get_double(&self) -> PyResult<usize> {
        Ok(self.core.data.get_double())
    }
    #[getter]
    fn get_cl(&self) -> PyResult<usize> {
        Ok(self.core.data.get_cl())
    }
    #[getter]
    fn get_flag(&self) -> PyResult<usize> {
        Ok(self.core.data.get_flag())
    }
    #[getter]
    fn get_bbbv_solved(&self) -> PyResult<usize> {
        Ok(self.core.data.get_bbbv_solved().unwrap())
    }
    #[getter]
    fn get_ce(&self) -> PyResult<usize> {
        Ok(self.core.data.get_ce().unwrap())
    }
    #[getter]
    fn get_left_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_left_s())
    }
    #[getter]
    fn get_right_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_right_s())
    }
    #[getter]
    fn get_double_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_double_s())
    }
    #[getter]
    fn get_cl_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_cl_s())
    }
    #[getter]
    fn get_flag_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_flag_s())
    }
    #[getter]
    fn get_path(&self) -> PyResult<f64> {
        Ok(self.core.data.get_path())
    }
    #[getter]
    fn get_ce_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_ce_s().unwrap())
    }
    #[getter]
    fn get_ioe(&self) -> PyResult<f64> {
        Ok(self.core.data.get_ioe().unwrap())
    }
    #[getter]
    fn get_thrp(&self) -> PyResult<f64> {
        Ok(self.core.data.get_thrp().unwrap())
    }
    #[getter]
    fn get_corr(&self) -> PyResult<f64> {
        Ok(self.core.data.get_corr().unwrap())
    }

    #[getter]
    fn get_events_len(&self) -> PyResult<usize> {
        Ok(self.core.data.video_action_state_recorder.len())
    }
    pub fn events_time(&self, index: usize) -> PyResult<f64> {
        Ok(self.core.data.video_action_state_recorder[index].time)
    }
    pub fn events_mouse(&self, index: usize) -> PyResult<String> {
        Ok(self.core.data.video_action_state_recorder[index]
            .mouse
            .clone())
    }
    pub fn events_x(&self, index: usize) -> PyResult<u16> {
        Ok(self.core.data.video_action_state_recorder[index].x)
    }
    pub fn events_y(&self, index: usize) -> PyResult<u16> {
        Ok(self.core.data.video_action_state_recorder[index].y)
    }
    pub fn events_useful_level(&self, index: usize) -> PyResult<u8> {
        Ok(self.core.data.video_action_state_recorder[index].useful_level)
    }
    pub fn events_prior_game_board(&self, index: usize) -> PyResult<PyGameBoard> {
        let mut t = PyGameBoard::new(self.core.data.mine_num);
        t.set_core(
            self.core.data.game_board_stream
                [self.core.data.video_action_state_recorder[index].prior_game_board_id]
                .clone(),
        );
        Ok(t)
    }
    pub fn events_comments(&self, index: usize) -> PyResult<String> {
        Ok(self.core.data.video_action_state_recorder[index]
            .comments
            .clone())
    }
    pub fn events_mouse_state(&self, index: usize) -> PyResult<usize> {
        match self.core.data.video_action_state_recorder[index].mouse_state {
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
        Ok(self.core.data.current_event_id)
    }
    #[setter]
    pub fn set_current_event_id(&mut self, id: usize) {
        self.core.data.current_event_id = id
    }
    #[getter]
    pub fn get_game_board(&self) -> PyResult<Vec<Vec<i32>>> {
        Ok(self.core.data.get_game_board())
    }
    #[getter]
    pub fn get_game_board_poss(&mut self) -> PyResult<Vec<Vec<f64>>> {
        Ok(self.core.data.get_game_board_poss())
    }
    #[getter]
    pub fn get_mouse_state(&self) -> PyResult<usize> {
        match self.core.data.video_action_state_recorder[self.core.data.current_event_id]
            .mouse_state
        {
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
    /// 局面状态（录像播放器的局面状态始终等于1，没有ready、win、fail的概念）
    #[getter]
    pub fn get_game_board_state(&self) -> PyResult<usize> {
        Ok(1)
    }
    #[getter]
    pub fn get_x_y(&self) -> PyResult<(u16, u16)> {
        Ok(self.core.data.get_x_y().unwrap())
    }
    #[setter]
    pub fn set_video_playing_pix_size(&mut self, pix_size: u8) {
        self.core.data.set_video_playing_pix_size(pix_size);
    }
    #[setter]
    pub fn set_current_time(&mut self, time: f64) {
        self.core.data.set_current_time(time);
    }
}

#[pyclass(name = "EvfVideo")]
pub struct PyEvfVideo {
    pub core: EvfVideo,
}

#[pymethods]
impl PyEvfVideo {
    #[new]
    pub fn new(file_name: &str) -> PyEvfVideo {
        let c = EvfVideo::new(file_name);
        PyEvfVideo { core: c }
    }
    pub fn parse_video(&mut self) {
        self.core.parse_video().unwrap();
    }
    pub fn analyse(&mut self) {
        self.core.data.analyse();
    }
    pub fn analyse_for_features(&mut self, controller: Vec<&str>) {
        self.core.data.analyse_for_features(controller);
    }
    pub fn generate_evf_v0_raw_data(&mut self) {
        self.core.data.generate_evf_v0_raw_data();
    }
    pub fn save_to_evf_file(&self, file_name: &str) {
        self.core.data.save_to_evf_file(file_name);
    }
    #[getter]
    fn get_time(&self) -> PyResult<f64> {
        Ok(self.core.data.get_time())
    }
    #[getter]
    fn get_software(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.software.clone())
    }
    #[getter]
    fn get_row(&self) -> PyResult<usize> {
        Ok(self.core.data.height)
    }
    #[getter]
    fn get_column(&self) -> PyResult<usize> {
        Ok(self.core.data.width)
    }
    #[getter]
    fn get_level(&self) -> PyResult<u8> {
        Ok(self.core.data.level)
    }
    #[getter]
    fn get_mode(&self) -> PyResult<u16> {
        Ok(self.core.data.mode)
    }
    #[getter]
    fn get_is_completed(&self) -> PyResult<bool> {
        Ok(self.core.data.is_completed)
    }
    #[getter]
    fn get_is_offical(&self) -> PyResult<bool> {
        Ok(self.core.data.is_offical)
    }
    #[getter]
    fn get_is_fair(&self) -> PyResult<bool> {
        Ok(self.core.data.is_fair)
    }
    #[getter]
    fn get_mine_num(&self) -> PyResult<usize> {
        Ok(self.core.data.mine_num)
    }
    #[getter]
    fn get_player_designator(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.player_designator.clone())
    }
    #[getter]
    fn get_race_designator(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.race_designator.clone())
    }
    #[getter]
    fn get_uniqueness_designator(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.uniqueness_designator.clone())
    }
    #[getter]
    fn get_country(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.country.clone())
    }
    #[getter]
    fn get_bbbv(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.bbbv)
    }
    #[getter]
    fn get_start_time(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.start_time.clone())
    }
    #[getter]
    fn get_end_time(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.data.end_time.clone())
    }
    #[getter]
    fn get_op(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.op)
    }
    #[getter]
    fn get_isl(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.isl)
    }
    #[getter]
    fn get_hizi(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.hizi)
    }
    #[getter]
    fn get_cell0(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell0)
    }
    #[getter]
    fn get_cell1(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell1)
    }
    #[getter]
    fn get_cell2(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell2)
    }
    #[getter]
    fn get_cell3(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell3)
    }
    #[getter]
    fn get_cell4(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell4)
    }
    #[getter]
    fn get_cell5(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell5)
    }
    #[getter]
    fn get_cell6(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell6)
    }
    #[getter]
    fn get_cell7(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell7)
    }
    #[getter]
    fn get_cell8(&self) -> PyResult<usize> {
        Ok(self.core.data.static_params.cell8)
    }
    #[getter]
    fn get_rtime(&self) -> PyResult<f64> {
        Ok(self.core.data.get_rtime().unwrap())
    }
    #[getter]
    fn get_rtime_ms(&self) -> PyResult<u32> {
        Ok(self.core.data.get_rtime_ms().unwrap())
    }
    #[getter]
    fn get_etime(&self) -> PyResult<f64> {
        Ok(self.core.data.get_etime().unwrap())
    }
    #[getter]
    pub fn get_video_time(&self) -> PyResult<f64> {
        Ok(self.core.data.get_video_time().unwrap())
    }
    #[getter]
    fn get_bbbv_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_bbbv_s().unwrap())
    }
    #[getter]
    fn get_stnb(&self) -> PyResult<f64> {
        Ok(self.core.data.get_stnb().unwrap())
    }
    #[getter]
    fn get_rqp(&self) -> PyResult<f64> {
        Ok(self.core.data.get_rqp().unwrap())
    }
    #[getter]
    fn get_left(&self) -> PyResult<usize> {
        Ok(self.core.data.get_left())
    }
    #[getter]
    fn get_right(&self) -> PyResult<usize> {
        Ok(self.core.data.get_right())
    }
    #[getter]
    fn get_double(&self) -> PyResult<usize> {
        Ok(self.core.data.get_double())
    }
    #[getter]
    fn get_cl(&self) -> PyResult<usize> {
        Ok(self.core.data.get_cl())
    }
    #[getter]
    fn get_flag(&self) -> PyResult<usize> {
        Ok(self.core.data.get_flag())
    }
    #[getter]
    fn get_bbbv_solved(&self) -> PyResult<usize> {
        Ok(self.core.data.get_bbbv_solved().unwrap())
    }
    #[getter]
    fn get_ce(&self) -> PyResult<usize> {
        Ok(self.core.data.get_ce().unwrap())
    }
    #[getter]
    fn get_left_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_left_s())
    }
    #[getter]
    fn get_right_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_right_s())
    }
    #[getter]
    fn get_double_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_double_s())
    }
    #[getter]
    fn get_cl_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_cl_s())
    }
    #[getter]
    fn get_flag_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_flag_s())
    }
    #[getter]
    fn get_path(&self) -> PyResult<f64> {
        Ok(self.core.data.get_path())
    }
    #[getter]
    fn get_ce_s(&self) -> PyResult<f64> {
        Ok(self.core.data.get_ce_s().unwrap())
    }
    #[getter]
    fn get_ioe(&self) -> PyResult<f64> {
        Ok(self.core.data.get_ioe().unwrap())
    }
    #[getter]
    fn get_thrp(&self) -> PyResult<f64> {
        Ok(self.core.data.get_thrp().unwrap())
    }
    #[getter]
    fn get_corr(&self) -> PyResult<f64> {
        Ok(self.core.data.get_corr().unwrap())
    }

    #[getter]
    fn get_events_len(&self) -> PyResult<usize> {
        Ok(self.core.data.video_action_state_recorder.len())
    }
    pub fn events_time(&self, index: usize) -> PyResult<f64> {
        Ok(self.core.data.video_action_state_recorder[index].time)
    }
    pub fn events_mouse(&self, index: usize) -> PyResult<String> {
        Ok(self.core.data.video_action_state_recorder[index]
            .mouse
            .clone())
    }
    pub fn events_x(&self, index: usize) -> PyResult<u16> {
        Ok(self.core.data.video_action_state_recorder[index].x)
    }
    pub fn events_y(&self, index: usize) -> PyResult<u16> {
        Ok(self.core.data.video_action_state_recorder[index].y)
    }
    pub fn events_useful_level(&self, index: usize) -> PyResult<u8> {
        Ok(self.core.data.video_action_state_recorder[index].useful_level)
    }
    pub fn events_prior_game_board(&self, index: usize) -> PyResult<PyGameBoard> {
        let mut t = PyGameBoard::new(self.core.data.mine_num);
        t.set_core(
            self.core.data.game_board_stream
                [self.core.data.video_action_state_recorder[index].prior_game_board_id]
                .clone(),
        );
        Ok(t)
    }
    pub fn events_comments(&self, index: usize) -> PyResult<String> {
        Ok(self.core.data.video_action_state_recorder[index]
            .comments
            .clone())
    }
    pub fn events_mouse_state(&self, index: usize) -> PyResult<usize> {
        match self.core.data.video_action_state_recorder[index].mouse_state {
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
        Ok(self.core.data.current_event_id)
    }
    #[setter]
    pub fn set_current_event_id(&mut self, id: usize) {
        self.core.data.current_event_id = id
    }
    #[getter]
    pub fn get_game_board(&self) -> PyResult<Vec<Vec<i32>>> {
        Ok(self.core.data.get_game_board())
    }
    #[getter]
    pub fn get_game_board_poss(&mut self) -> PyResult<Vec<Vec<f64>>> {
        Ok(self.core.data.get_game_board_poss())
    }
    #[getter]
    pub fn get_mouse_state(&self) -> PyResult<usize> {
        match self.core.data.video_action_state_recorder[self.core.data.current_event_id]
            .mouse_state
        {
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
    /// 局面状态（录像播放器的局面状态始终等于1，没有ready、win、fail的概念）
    #[getter]
    pub fn get_game_board_state(&self) -> PyResult<usize> {
        Ok(1)
    }
    #[getter]
    pub fn get_x_y(&self) -> PyResult<(u16, u16)> {
        Ok(self.core.data.get_x_y().unwrap())
    }
    #[setter]
    pub fn set_video_playing_pix_size(&mut self, pix_size: u8) {
        self.core.data.set_video_playing_pix_size(pix_size);
    }
    #[setter]
    pub fn set_current_time(&mut self, time: f64) {
        self.core.data.set_current_time(time);
    }
}

#[pyclass(name = "BaseVideo")]
pub struct PyBaseVideo {
    pub core: BaseVideo,
}

#[pymethods]
impl PyBaseVideo {
    #[new]
    pub fn new(board: Vec<Vec<i32>>, cell_pixel_size: u8) -> PyBaseVideo {
        let c = BaseVideo::new_before_game(board, cell_pixel_size);
        PyBaseVideo { core: c }
    }
    pub fn analyse(&mut self) {
        self.core.analyse();
    }
    pub fn analyse_for_features(&mut self, controller: Vec<&str>) {
        self.core.analyse_for_features(controller);
    }
    pub fn generate_evf_v0_raw_data(&mut self) {
        self.core.generate_evf_v0_raw_data();
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
    fn get_is_offical(&self) -> PyResult<bool> {
        Ok(self.core.is_offical)
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
    fn get_player_designator(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.player_designator.clone())
    }
    #[getter]
    fn get_race_designator(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.race_designator.clone())
    }
    #[getter]
    fn get_uniqueness_designator(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.uniqueness_designator.clone())
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
    pub fn get_video_time(&self) -> PyResult<f64> {
        Ok(self.core.get_video_time().unwrap())
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
            self.core.game_board_stream[self.core.video_action_state_recorder[index].prior_game_board_id].clone(),
        );
        Ok(t)
    }
    pub fn events_comments(&self, index: usize) -> PyResult<String> {
        Ok(self.core.video_action_state_recorder[index].comments.clone())
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
    fn get_board(&self) -> PyResult<Vec<Vec<i32>>> {
        Ok(self.core.minesweeper_board.board.clone())
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
    #[setter]
    pub fn set_video_playing_pix_size(&mut self, pix_size: u8) {
        self.core.set_video_playing_pix_size(pix_size);
    }
    #[setter]
    pub fn set_current_time(&mut self, time: f64) {
        self.core.set_current_time(time);
    }
    #[setter]
    pub fn set_is_offical(&mut self, is_offical: bool) {
        self.core.set_is_offical(is_offical).unwrap();
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
    pub fn set_player_designator(&mut self, player_designator: Vec<u8>) {
        self.core.set_player_designator(player_designator).unwrap();
    }
    #[setter]
    pub fn set_race_designator(&mut self, race_designator: Vec<u8>) {
        self.core.set_race_designator(race_designator).unwrap();
    }
    #[setter]
    pub fn set_uniqueness_designator(&mut self, uniqueness_designator: Vec<u8>) {
        self.core.set_uniqueness_designator(uniqueness_designator).unwrap();
    }
    #[setter]
    pub fn set_country(&mut self, country: Vec<u8>) {
        self.core.set_country(country).unwrap();
    }
    #[setter]
    pub fn set_checksum(&mut self, checksum: [u8; 32]) {
        self.core.set_checksum(checksum).unwrap();
    }
    #[setter]
    pub fn set_pix_size(&mut self, pix_size: u8) {
        self.core.set_pix_size(pix_size);
    }
}

#[pyclass(name = "GameBoard")]
pub struct PyGameBoard {
    pub core: GameBoard,
}

// 这个干嘛的，忘了
impl PyGameBoard {
    fn set_core(&mut self, value: GameBoard) {
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
