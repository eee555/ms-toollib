use crate::transfor::{json2vec, trans_opt, vec2json};
// use crate::web_sys;
use ms_toollib as ms;
use wasm_bindgen::prelude::*;

pub fn set_panic_hook() {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// 局面自动机
#[wasm_bindgen(inspectable)]
pub struct MinesweeperBoard {
    core: ms::MinesweeperBoard<Vec<Vec<i32>>>,
}

#[wasm_bindgen]
pub struct CursorPos(pub u16, pub u16);

#[wasm_bindgen]
impl MinesweeperBoard {
    pub fn new(board: &str) -> MinesweeperBoard {
        let board = json2vec(board);
        MinesweeperBoard {
            core: ms::MinesweeperBoard::new(board),
        }
    }
    pub fn step(&mut self, e: &str, x: u32, y: u32) {
        self.core.step(e, (x as usize, y as usize)).unwrap();
    }
    pub fn step_flow(&mut self, operation: &str) {
        let opera = trans_opt(operation);
        let operation = opera
            .iter()
            .map(|x| (&((*x).0)[..], ((*x).1 .0, (*x).1 .1)))
            .collect();
        self.core.step_flow(operation).unwrap();
    }
    // 这个方法与强可猜、弱可猜有关
    #[wasm_bindgen(setter)]
    pub fn set_board(&mut self, board: &str) {
        self.core.board = json2vec(board);
    }
    // 直接设置游戏局面是不安全的！但在一些游戏中，结束时需要修改再展示
    #[wasm_bindgen(setter)]
    pub fn set_game_board(&mut self, game_board: &str) {
        self.core.game_board = json2vec(game_board);
    }
    #[wasm_bindgen(getter)]
    pub fn get_board(&self) -> String {
        vec2json(&self.core.board)
    }
    #[wasm_bindgen(getter)]
    pub fn get_game_board(&self) -> String {
        vec2json(&self.core.game_board)
    }
    #[wasm_bindgen(getter)]
    pub fn get_left(&self) -> u32 {
        self.core.left as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_right(&self) -> u32 {
        self.core.right as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_double(&self) -> u32 {
        self.core.double as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_ce(&self) -> u32 {
        self.core.ce as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_flag(&self) -> u32 {
        self.core.flag as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_bbbv_solved(&self) -> u32 {
        self.core.bbbv_solved as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_row(&self) -> u32 {
        self.core.row as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_column(&self) -> u32 {
        self.core.column as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_game_board_state(&self) -> u32 {
        match self.core.game_board_state {
            ms::GameBoardState::Ready => 1,
            ms::GameBoardState::Playing => 2,
            ms::GameBoardState::Win => 3,
            ms::GameBoardState::Loss => 4,
            ms::GameBoardState::PreFlaging => 5,
            ms::GameBoardState::Display => 6,
        }
    }
    #[wasm_bindgen(getter)]
    pub fn get_mouse_state(&self) -> u32 {
        match self.core.mouse_state {
            ms::MouseState::UpUp => 1,
            ms::MouseState::UpDown => 2,
            ms::MouseState::UpDownNotFlag => 3,
            ms::MouseState::DownUp => 4,
            ms::MouseState::Chording => 5,
            ms::MouseState::ChordingNotFlag => 6,
            ms::MouseState::DownUpAfterChording => 7,
            ms::MouseState::Undefined => 8,
        }
    }
}

// 定义宏，生成所有类型录像的子类
macro_rules! generate_video {
    ($($some_video:ident),*) => {
        $(
            #[wasm_bindgen(inspectable)]
            struct $some_video {
                core: ms::$some_video,
            }
            #[wasm_bindgen]
            impl $some_video {
                pub fn new(data: Box<[u8]>, file_name: &str) -> $some_video {
                    set_panic_hook();
                    let data = data.into_vec();
                    $some_video {
                        core: ms::$some_video::new(data, file_name),
                    }
                }
                pub fn parse_video(&mut self) {
                    self.core.parse_video().unwrap();
                }
                pub fn analyse(&mut self) {
                    self.core.data.analyse();
                }
                #[wasm_bindgen(getter)]
                pub fn get_raw_data(&self) -> Vec<u8> {
                    self.core.data.get_raw_data().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_software(&self) -> Vec<u8> {
                    self.core.data.software.clone()
                }
                #[wasm_bindgen(getter)]
                pub fn get_row(&self) -> usize {
                    self.core.data.height
                }
                #[wasm_bindgen(getter)]
                pub fn get_column(&self) -> usize {
                    self.core.data.width
                }
                #[wasm_bindgen(getter)]
                pub fn get_level(&self) -> u8 {
                    self.core.data.level
                }
                #[wasm_bindgen(getter)]
                pub fn get_mode(&self) -> u16 {
                    self.core.data.mode
                }
                #[wasm_bindgen(getter)]
                pub fn get_is_completed(&self) -> bool {
                    self.core.data.is_completed
                }
                #[wasm_bindgen(getter)]
                pub fn get_is_offical(&self) -> bool {
                    self.core.data.is_offical
                }
                #[wasm_bindgen(getter)]
                pub fn get_is_fair(&self) -> bool {
                    self.core.data.is_fair
                }
                #[wasm_bindgen(getter)]
                pub fn get_mine_num(&self) -> usize {
                    self.core.data.mine_num
                }
                #[wasm_bindgen(getter)]
                pub fn get_player_designator(&self) -> Vec<u8> {
                    self.core.data.player_designator.clone()
                }
                #[wasm_bindgen(getter)]
                pub fn get_race_designator(&self) -> Vec<u8> {
                    self.core.data.race_designator.clone()
                }
                #[wasm_bindgen(getter)]
                pub fn get_uniqueness_designator(&self) -> Vec<u8> {
                    self.core.data.uniqueness_designator.clone()
                }
                #[wasm_bindgen(getter)]
                pub fn get_country(&self) -> Vec<u8> {
                    self.core.data.country.clone()
                }
                #[wasm_bindgen(getter)]
                pub fn get_bbbv(&self) -> usize {
                    self.core.data.static_params.bbbv
                }
                #[wasm_bindgen(getter)]
                pub fn get_start_time(&self) -> Vec<u8> {
                    self.core.data.start_time.clone()
                }
                #[wasm_bindgen(getter)]
                pub fn get_end_time(&self) -> Vec<u8> {
                    self.core.data.end_time.clone()
                }
                #[wasm_bindgen(getter)]
                pub fn get_op(&self) -> usize {
                    self.core.data.static_params.op
                }
                #[wasm_bindgen(getter)]
                pub fn get_isl(&self) -> usize {
                    self.core.data.static_params.isl
                }
                #[wasm_bindgen(getter)]
                pub fn get_hizi(&self) -> usize {
                    self.core.data.static_params.hizi
                }
                #[wasm_bindgen(getter)]
                pub fn get_cell0(&self) -> usize {
                    self.core.data.static_params.cell0
                }
                #[wasm_bindgen(getter)]
                pub fn get_cell1(&self) -> usize {
                    self.core.data.static_params.cell1
                }
                #[wasm_bindgen(getter)]
                pub fn get_cell2(&self) -> usize {
                    self.core.data.static_params.cell2
                }
                #[wasm_bindgen(getter)]
                pub fn get_cell3(&self) -> usize {
                    self.core.data.static_params.cell3
                }
                #[wasm_bindgen(getter)]
                pub fn get_cell4(&self) -> usize {
                    self.core.data.static_params.cell4
                }
                #[wasm_bindgen(getter)]
                pub fn get_cell5(&self) -> usize {
                    self.core.data.static_params.cell5
                }
                #[wasm_bindgen(getter)]
                pub fn get_cell6(&self) -> usize {
                    self.core.data.static_params.cell6
                }
                #[wasm_bindgen(getter)]
                pub fn get_cell7(&self) -> usize {
                    self.core.data.static_params.cell7
                }
                #[wasm_bindgen(getter)]
                pub fn get_cell8(&self) -> usize {
                    self.core.data.static_params.cell8
                }
                #[wasm_bindgen(getter)]
                pub fn get_rtime(&self) -> f64 {
                    self.core.data.get_rtime().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_rtime_ms(&self) -> u32 {
                    self.core.data.get_rtime_ms().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_etime(&self) -> f64 {
                    self.core.data.get_etime().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_video_time(&self) -> f64 {
                    self.core.data.get_video_time().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_bbbv_s(&self) -> f64 {
                    self.core.data.get_bbbv_s().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_stnb(&self) -> f64 {
                    self.core.data.get_stnb().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_rqp(&self) -> f64 {
                    self.core.data.get_rqp().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_left(&self) -> usize {
                    self.core.data.get_left()
                }
                #[wasm_bindgen(getter)]
                pub fn get_right(&self) -> usize {
                    self.core.data.get_right()
                }
                #[wasm_bindgen(getter)]
                pub fn get_double(&self) -> usize {
                    self.core.data.get_double()
                }
                #[wasm_bindgen(getter)]
                pub fn get_cl(&self) -> usize {
                    self.core.data.get_cl()
                }
                #[wasm_bindgen(getter)]
                pub fn get_flag(&self) -> usize {
                    self.core.data.get_flag()
                }
                #[wasm_bindgen(getter)]
                pub fn get_bbbv_solved(&self) -> usize {
                    self.core.data.get_bbbv_solved().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_ce(&self) -> usize {
                    self.core.data.get_ce().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_left_s(&self) -> f64 {
                    self.core.data.get_left_s()
                }
                #[wasm_bindgen(getter)]
                pub fn get_right_s(&self) -> f64 {
                    self.core.data.get_right_s()
                }
                #[wasm_bindgen(getter)]
                pub fn get_double_s(&self) -> f64 {
                    self.core.data.get_double_s()
                }
                #[wasm_bindgen(getter)]
                pub fn get_cl_s(&self) -> f64 {
                    self.core.data.get_cl_s()
                }
                #[wasm_bindgen(getter)]
                pub fn get_flag_s(&self) -> f64 {
                    self.core.data.get_flag_s()
                }
                #[wasm_bindgen(getter)]
                pub fn get_path(&self) -> f64 {
                    self.core.data.get_path()
                }
                #[wasm_bindgen(getter)]
                pub fn get_ce_s(&self) -> f64 {
                    self.core.data.get_ce_s().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_ioe(&self) -> f64 {
                    self.core.data.get_ioe().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_thrp(&self) -> f64 {
                    self.core.data.get_thrp().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_corr(&self) -> f64 {
                    self.core.data.get_corr().unwrap()
                }
                #[wasm_bindgen(getter)]
                pub fn get_events_len(&self) -> usize {
                    self.core.data.video_action_state_recorder.len()
                }
                pub fn events_time(&self, index: usize) -> f64 {
                    self.core.data.video_action_state_recorder[index].time
                }
                pub fn events_mouse(&self, index: usize) -> String {
                    self.core.data.video_action_state_recorder[index]
                        .mouse
                        .clone()
                }
                pub fn events_x(&self, index: usize) -> u16 {
                    self.core.data.video_action_state_recorder[index].x
                }
                pub fn events_y(&self, index: usize) -> u16 {
                    self.core.data.video_action_state_recorder[index].y
                }
                pub fn events_useful_level(&self, index: usize) -> u8 {
                    self.core.data.video_action_state_recorder[index].useful_level
                }
                // 这里用不到先不往下写
                // pub fn events_posteriori_game_board(&self, index: usize) -> PyGameBoard> {
                //     let mut t = PyGameBoard::new(self.core.mine_num);
                //     t.set_core(self.core.events[index].posteriori_game_board.clone());
                //     Ok(t)
                // }
                // pub fn events_comments(&self, index: usize) -> String {
                //     self.core.events[index].comments.clone()
                // }
                pub fn events_mouse_state(&self, index: usize) -> u32 {
                    match self.core.data.video_action_state_recorder[index].mouse_state {
                        ms::MouseState::UpUp => 1,
                        ms::MouseState::UpDown => 2,
                        ms::MouseState::UpDownNotFlag => 3,
                        ms::MouseState::DownUp => 4,
                        ms::MouseState::Chording => 5,
                        ms::MouseState::ChordingNotFlag => 6,
                        ms::MouseState::DownUpAfterChording => 7,
                        ms::MouseState::Undefined => 8,
                    }
                }
                #[wasm_bindgen(getter)]
                pub fn get_current_event_id(&self) -> usize {
                    self.core.data.current_event_id
                }
                #[wasm_bindgen(setter)]
                pub fn set_current_event_id(&mut self, id: usize) {
                    self.core.data.current_event_id = id
                }
                #[wasm_bindgen(getter)]
                pub fn get_game_board(&self) -> String {
                    vec2json(&self.core.data.get_game_board())
                }
                #[wasm_bindgen(getter)]
                pub fn get_game_board_poss(&mut self) -> String {
                    vec2json(&self.core.data.get_game_board_poss())
                }
                #[wasm_bindgen(getter)]
                pub fn get_mouse_state(&self) -> u32 {
                    match self.core.data.video_action_state_recorder[self.core.data.current_event_id]
                        .mouse_state
                    {
                        ms::MouseState::UpUp => 1,
                        ms::MouseState::UpDown => 2,
                        ms::MouseState::UpDownNotFlag => 3,
                        ms::MouseState::DownUp => 4,
                        ms::MouseState::Chording => 5,
                        ms::MouseState::ChordingNotFlag => 6,
                        ms::MouseState::DownUpAfterChording => 7,
                        ms::MouseState::Undefined => 8,
                    }
                }
                /// 局面状态
                #[wasm_bindgen(getter)]
                pub fn get_game_board_state(&self) -> usize {
                    match self.core.data.game_board_state {
                        ms::GameBoardState::Ready => 1,
                        ms::GameBoardState::Playing => 2,
                        ms::GameBoardState::Win => 3,
                        ms::GameBoardState::Loss => 4,
                        ms::GameBoardState::PreFlaging => 5,
                        ms::GameBoardState::Display => 6,
                    }
                }
                /// 返回当前光标的位置，播放录像用
                #[wasm_bindgen(getter)]
                pub fn get_x_y(&self) -> CursorPos {
                    let (x, y) = self.core.data.get_x_y().unwrap();
                    CursorPos(x, y)
                }
                #[wasm_bindgen(getter)]
                pub fn get_checksum(&self) -> Vec<u8> {
                    self.core.data.get_checksum().unwrap().to_vec()
                }
                #[wasm_bindgen(getter)]
                pub fn get_pix_size(&self) -> u8 {
                    self.core.data.get_pix_size().unwrap()
                }
                #[wasm_bindgen(setter)]
                pub fn set_current_time(&mut self, time: f64) {
                    self.core.data.set_current_time(time);
                }
                pub fn is_valid(&self) -> u8 {
                    self.core.data.is_valid()
                }
            }
        )*

    };
}
generate_video!(AvfVideo, EvfVideo, MvfVideo, RmvVideo);
