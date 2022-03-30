use crate::transfor::{json2vec, trans_opt, vec2json};
use ms_toollib as ms;
use wasm_bindgen::prelude::*;

// 局面自动机
#[wasm_bindgen(inspectable)]
pub struct MinesweeperBoard {
    core: ms::MinesweeperBoard,
}
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
        let mut opera = trans_opt(operation);
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
    pub fn get_chording(&self) -> u32 {
        self.core.chording as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_ces(&self) -> u32 {
        self.core.ces as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_flag(&self) -> u32 {
        self.core.flag as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_solved3BV(&self) -> u32 {
        self.core.solved3BV as u32
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





#[wasm_bindgen(inspectable)]
pub struct AvfVideo {
    core: ms::AvfVideo,
}
#[wasm_bindgen]
impl AvfVideo {
    pub fn new(file_name: &str) -> AvfVideo {
        AvfVideo {
            core: ms::AvfVideo::new("C://Users//p//Documents//GitHub//ms_toollib//wasm//www//jze.avf"),
        }
    }
    pub fn parse_video(&mut self) {
        self.core.parse_video().unwrap();
    }
    pub fn analyse(&mut self) {
        self.core.analyse();
    }
    // 用不到先不写
    // pub fn analyse_for_features(&mut self, controller: Vec<&str>) {
    //     self.core.analyse_for_features(controller);
    // }
    #[wasm_bindgen(getter)]
    pub fn get_row(&self) -> u32 {
        self.core.height as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_column(&self) -> u32 {
        self.core.width as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_mine_num(&self) -> u32 {
        self.core.mine_num as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_bbbv(&self) -> u32 {
        self.core.static_params.bbbv as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_openings(&self) -> u32 {
        self.core.static_params.openings as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_islands(&self) -> u32 {
        self.core.static_params.islands as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_hizi(&self) -> u32 {
        self.core.static_params.hizi as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_cell0(&self) -> u32 {
        self.core.static_params.cell0 as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_cell1(&self) -> u32 {
        self.core.static_params.cell1 as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_cell2(&self) -> u32 {
        self.core.static_params.cell2 as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_cell3(&self) -> u32 {
        self.core.static_params.cell3 as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_cell4(&self) -> u32 {
        self.core.static_params.cell4 as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_cell5(&self) -> u32 {
        self.core.static_params.cell5 as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_cell6(&self) -> u32 {
        self.core.static_params.cell6 as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_cell7(&self) -> u32 {
        self.core.static_params.cell7 as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_cell8(&self) -> u32 {
        self.core.static_params.cell8 as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_r_time(&self) -> f64 {
        self.core.dynamic_params.r_time
    }
    #[wasm_bindgen(getter)]
    pub fn get_bbbv_s(&self) -> f64 {
        self.core.dynamic_params.bbbv_s
    }
    #[wasm_bindgen(getter)]
    pub fn get_stnb(&self) -> f64 {
        self.core.dynamic_params.stnb
    }
    #[wasm_bindgen(getter)]
    pub fn get_rqp(&self) -> f64 {
        self.core.dynamic_params.rqp
    }
    #[wasm_bindgen(getter)]
    pub fn get_lefts(&self) -> u32 {
        self.core.dynamic_params.lefts as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_rights(&self) -> u32 {
        self.core.dynamic_params.rights as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_chordings(&self) -> u32 {
        self.core.dynamic_params.chordings as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_clicks(&self) -> u32 {
        self.core.dynamic_params.clicks as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_flags(&self) -> u32 {
        self.core.dynamic_params.flags as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_ces(&self) -> u32 {
        self.core.dynamic_params.ces as u32
    }
    #[wasm_bindgen(getter)]
    pub fn get_lefts_s(&self) -> f64 {
        self.core.dynamic_params.lefts_s
    }
    #[wasm_bindgen(getter)]
    pub fn get_rights_s(&self) -> f64 {
        self.core.dynamic_params.rights_s
    }
    #[wasm_bindgen(getter)]
    pub fn get_chordings_s(&self) -> f64 {
        self.core.dynamic_params.chordings_s
    }
    #[wasm_bindgen(getter)]
    pub fn get_clicks_s(&self) -> f64 {
        self.core.dynamic_params.clicks_s
    }
    #[wasm_bindgen(getter)]
    pub fn get_ces_s(&self) -> f64 {
        self.core.dynamic_params.ces_s
    }
    #[wasm_bindgen(getter)]
    pub fn get_events_len(&self) -> u32 {
        self.core.events.len() as u32
    }
    pub fn events_time(&self, index: usize) -> f64 {
        self.core.events[index].time
    }
    pub fn events_mouse(&self, index: usize) -> String {
        self.core.events[index].mouse.clone()
    }
    pub fn events_x(&self, index: usize) -> u32 {
        self.core.events[index].x as u32
    }
    pub fn events_y(&self, index: usize) -> u32 {
        self.core.events[index].y as u32
    }
    pub fn events_useful_level(&self, index: usize) -> u32 {
        self.core.events[index].useful_level as u32
    }
    // 这里用不到先不往下写
    // pub fn events_posteriori_game_board(&self, index: usize) -> PyResult<PyGameBoard> {
    //     let mut t = PyGameBoard::new(self.core.mine_num);
    //     t.set_core(self.core.events[index].posteriori_game_board.clone());
    //     Ok(t)
    // }
    pub fn events_comments(&self, index: usize) -> String {
        self.core.events[index].comments.clone()
    }
    pub fn events_mouse_state(&self, index: usize) -> u32 {
        match self.core.events[index].mouse_state {
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
    pub fn get_current_event_id(&self) -> u32 {
        self.core.current_event_id as u32
    }
    #[wasm_bindgen(setter)]
    pub fn set_current_event_id(&mut self, id: usize) {
        self.core.current_event_id = id
    }
    #[wasm_bindgen(getter)]
    pub fn get_game_board(&self) -> String {
        vec2json(&self.core.get_game_board())
    }
    #[wasm_bindgen(getter)]
    pub fn get_game_board_poss(&mut self) -> String {
        vec2json(&self.core.get_game_board_poss())
    }
    #[wasm_bindgen(getter)]
    pub fn get_mouse_state(&self) -> u32 {
        match self.core.events[self.core.current_event_id].mouse_state {
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
    /// 局面状态（录像播放器的局面状态始终等于1，没有ready、win、fail的概念）
    #[wasm_bindgen(getter)]
    pub fn get_game_board_state(&self) -> u32 {
        1
    }
    /// 返回当前光标的位置，播放录像用，用不到先不写
    // #[wasm_bindgen(getter)]
    // pub fn get_x_y(&self) -> (u32, u32) {
    //     (self.core.events[self.core.current_event_id].x, self.core.events[self.core.current_event_id].y)
    // }
    #[wasm_bindgen(setter)]
    pub fn set_time(&mut self, time: f64) {
        self.core.set_current_event_time(time);
    }
}
