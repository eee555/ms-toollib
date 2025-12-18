// use crate::board;
use crate::transfor::{js_value_to_vec_vec, vec_vec_to_js_value};
use js_sys::Array;
use ms;
use ms::videos::NewSomeVideo2;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;


#[wasm_bindgen]
pub struct Board {
    core: ms::Board,
}

#[wasm_bindgen]
impl Board {
    #[wasm_bindgen(constructor)]
    pub fn new(board: JsValue) -> Board {
        let c = ms::Board::new(js_value_to_vec_vec(board));
        Board { core: c }
    }
    #[wasm_bindgen(getter = bbbv)]
    pub fn get_bbbv(&mut self) -> usize {
        self.core.get_bbbv()
    }
    #[wasm_bindgen(getter = op)]
    pub fn get_op(&mut self) -> usize {
        self.core.get_op()
    }
    #[wasm_bindgen(getter = isl)]
    pub fn get_isl(&mut self) -> usize {
        self.core.get_isl()
    }
    #[wasm_bindgen(getter = cell0)]
    pub fn get_cell0(&mut self) -> usize {
        self.core.get_cell0()
    }
    #[wasm_bindgen(getter = cell1)]
    pub fn get_cell1(&mut self) -> usize {
        self.core.get_cell1()
    }
    #[wasm_bindgen(getter = cell2)]
    pub fn get_cell2(&mut self) -> usize {
        self.core.get_cell2()
    }
    #[wasm_bindgen(getter = cell3)]
    pub fn get_cell3(&mut self) -> usize {
        self.core.get_cell3()
    }
    #[wasm_bindgen(getter = cell4)]
    pub fn get_cell4(&mut self) -> usize {
        self.core.get_cell4()
    }
    #[wasm_bindgen(getter = cell5)]
    pub fn get_cell5(&mut self) -> usize {
        self.core.get_cell5()
    }
    #[wasm_bindgen(getter = cell6)]
    pub fn get_cell6(&mut self) -> usize {
        self.core.get_cell6()
    }
    #[wasm_bindgen(getter = cell7)]
    pub fn get_cell7(&mut self) -> usize {
        self.core.get_cell7()
    }
    #[wasm_bindgen(getter = cell8)]
    pub fn get_cell8(&mut self) -> usize {
        self.core.get_cell8()
    }
}


#[wasm_bindgen]
pub struct GameBoard {
    core: ms::GameBoard,
}

#[wasm_bindgen]
impl GameBoard {
    #[wasm_bindgen(constructor)]
    pub fn new(mine_num: usize) -> GameBoard {
        let c = ms::GameBoard::new(mine_num);
        GameBoard { core: c }
    }
    #[wasm_bindgen(setter = game_board)]
    pub fn set_game_board(&mut self, js_board: JsValue) {
        let board = js_value_to_vec_vec(js_board);
        self.core.set_game_board(&board);
    }
    #[wasm_bindgen(getter = game_board)]
    pub fn get_game_board(&mut self) -> JsValue {
        vec_vec_to_js_value(self.core.game_board.clone())
    }
    #[wasm_bindgen(getter = poss)]
    pub fn get_poss(&mut self) -> JsValue {
        vec_vec_to_js_value(self.core.get_poss().to_vec())
    }
    #[wasm_bindgen(getter = basic_not_mine)]
    pub fn get_basic_not_mine(&mut self) -> JsValue {
        let ps = self.core.get_basic_not_mine().to_vec();
        let outer_array = Array::new_with_length(ps.len() as u32);
        for (i, (a, b)) in ps.iter().enumerate() {
            let inner_array = Array::new_with_length(2);
            inner_array.set(0, JsValue::from(*a));
            inner_array.set(1, JsValue::from(*b));
            outer_array.set(i as u32, inner_array.into());
        }
        outer_array.into()
    }
    #[wasm_bindgen(getter = basic_is_mine)]
    pub fn get_basic_is_mine(&mut self) -> JsValue {
        let ps = self.core.get_basic_is_mine().to_vec();
        let outer_array = Array::new_with_length(ps.len() as u32);
        for (i, (a, b)) in ps.iter().enumerate() {
            let inner_array = Array::new_with_length(2);
            inner_array.set(0, JsValue::from(*a));
            inner_array.set(1, JsValue::from(*b));
            outer_array.set(i as u32, inner_array.into());
        }
        outer_array.into()
    }
    #[wasm_bindgen(getter = enum_not_mine)]
    pub fn get_enum_not_mine(&mut self) -> JsValue {
        let ps = self.core.get_enum_not_mine().to_vec();
        let outer_array = Array::new_with_length(ps.len() as u32);
        for (i, (a, b)) in ps.iter().enumerate() {
            let inner_array = Array::new_with_length(2);
            inner_array.set(0, JsValue::from(*a));
            inner_array.set(1, JsValue::from(*b));
            outer_array.set(i as u32, inner_array.into());
        }
        outer_array.into()
    }
    #[wasm_bindgen(getter = enum_is_mine)]
    pub fn get_enum_is_mine(&mut self) -> JsValue {
        let ps = self.core.get_enum_is_mine().to_vec();
        let outer_array = Array::new_with_length(ps.len() as u32);
        for (i, (a, b)) in ps.iter().enumerate() {
            let inner_array = Array::new_with_length(2);
            inner_array.set(0, JsValue::from(*a));
            inner_array.set(1, JsValue::from(*b));
            outer_array.set(i as u32, inner_array.into());
        }
        outer_array.into()
    }
}

// 局面自动机
#[wasm_bindgen]
pub struct MinesweeperBoard {
    core: ms::MinesweeperBoard<Vec<Vec<i32>>>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct CursorPos {
    pub x: u16,
    pub y: u16,
}

#[wasm_bindgen]
impl MinesweeperBoard {
    #[wasm_bindgen(constructor)]
    pub fn new(board: JsValue) -> MinesweeperBoard {
        let board = js_value_to_vec_vec(board);
        MinesweeperBoard {
            core: ms::MinesweeperBoard::new(board),
        }
    }
    pub fn step(&mut self, e: &str, x: usize, y: usize) {
        self.core.step(e, (x, y)).unwrap();
    }
    pub fn step_flow(&mut self, js_operation: JsValue) {
        let array_operation = Array::from(&js_operation);
        let operation = array_operation
            .iter()
            .map(|item| {
                let item_array = Array::from(&item);
                let key = item_array.get(0);
                let value = Array::from(&item_array.get(1));
                let key_str = key.as_string().unwrap();
                let x = value.get(0).as_f64().unwrap() as usize;
                let y = value.get(1).as_f64().unwrap() as usize;

                (key_str, (x, y))
            })
            .collect();
        self.core.step_flow(&operation).unwrap();
    }
    // 这个方法与强可猜、弱可猜有关
    #[wasm_bindgen(setter = board)]
    pub fn set_board(&mut self, board: JsValue) {
        self.core.board = js_value_to_vec_vec(board);
    }
    // 直接设置游戏局面是不安全的！但在一些游戏中，结束时需要修改再展示
    #[wasm_bindgen(setter = game_board)]
    pub fn set_game_board(&mut self, js_board: JsValue) {
        self.core.game_board = js_value_to_vec_vec(js_board);
    }
    #[wasm_bindgen(getter = board)]
    pub fn get_board(&self) -> JsValue {
        vec_vec_to_js_value(self.core.board.clone())
    }
    #[wasm_bindgen(getter = game_board)]
    pub fn get_game_board(&self) -> JsValue {
        vec_vec_to_js_value(self.core.game_board.clone())
    }
    #[wasm_bindgen(getter = left)]
    pub fn get_left(&self) -> u32 {
        self.core.left as u32
    }
    #[wasm_bindgen(getter = right)]
    pub fn get_right(&self) -> u32 {
        self.core.right as u32
    }
    #[wasm_bindgen(getter = double)]
    pub fn get_double(&self) -> u32 {
        self.core.double as u32
    }
    #[wasm_bindgen(getter = ce)]
    pub fn get_ce(&self) -> u32 {
        (self.core.lce + self.core.rce + self.core.dce) as u32
    }
    #[wasm_bindgen(getter = lce)]
    pub fn get_lce(&self) -> u32 {
        self.core.lce as u32
    }
    #[wasm_bindgen(getter = rce)]
    pub fn get_rce(&self) -> u32 {
        self.core.rce as u32
    }
    #[wasm_bindgen(getter = dce)]
    pub fn get_dce(&self) -> u32 {
        self.core.dce as u32
    }
    #[wasm_bindgen(getter = flag)]
    pub fn get_flag(&self) -> u32 {
        self.core.flag as u32
    }
    #[wasm_bindgen(getter = bbbv_solved)]
    pub fn get_bbbv_solved(&self) -> u32 {
        self.core.bbbv_solved as u32
    }
    #[wasm_bindgen(getter = row)]
    pub fn get_row(&self) -> u32 {
        self.core.row as u32
    }
    #[wasm_bindgen(getter = column)]
    pub fn get_column(&self) -> u32 {
        self.core.column as u32
    }
    #[wasm_bindgen(getter = game_board_state)]
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
    #[wasm_bindgen(getter = mouse_state)]
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

#[wasm_bindgen]
pub struct KeyDynamicParams {
    core: ms::videos::base_video::KeyDynamicParams,
}

#[wasm_bindgen]
impl KeyDynamicParams {
    #[wasm_bindgen(getter = left)]
    pub fn get_left(&self) -> usize {
        self.core.left
    }
    #[wasm_bindgen(getter = right)]
    pub fn get_right(&self) -> usize {
        self.core.right
    }
    #[wasm_bindgen(getter = double)]
    pub fn get_double(&self) -> usize {
        self.core.double
    }
    #[wasm_bindgen(getter = lce)]
    pub fn get_lce(&self) -> usize {
        self.core.lce
    }
    #[wasm_bindgen(getter = rce)]
    pub fn get_rce(&self) -> usize {
        self.core.rce
    }
    #[wasm_bindgen(getter = dce)]
    pub fn get_dce(&self) -> usize {
        self.core.dce
    }
    #[wasm_bindgen(getter = flag)]
    pub fn get_flag(&self) -> usize {
        self.core.flag
    }
    #[wasm_bindgen(getter = bbbv_solved)]
    pub fn get_bbbv_solved(&self) -> usize {
        self.core.bbbv_solved
    }
    #[wasm_bindgen(getter = op_solved)]
    pub fn get_op_solved(&self) -> usize {
        self.core.op_solved
    }
    #[wasm_bindgen(getter = isl_solved)]
    pub fn get_isl_solved(&self) -> usize {
        self.core.isl_solved
    }
}

#[wasm_bindgen]
pub struct VideoActionStateRecorder {
    core: ms::videos::base_video::VideoActionStateRecorder,
}

#[wasm_bindgen]
impl VideoActionStateRecorder {
    #[wasm_bindgen(getter = time)]
    pub fn get_time(&self) -> f64 {
        self.core.time
    }
    #[wasm_bindgen(getter = x)]
    pub fn get_x(&self) -> u16 {
        self.core.x
    }
    #[wasm_bindgen(getter = y)]
    pub fn get_y(&self) -> u16 {
        self.core.y
    }
    #[wasm_bindgen(getter = mouse)]
    pub fn get_mouse(&self) -> String {
        self.core.mouse.clone()
    }
    #[wasm_bindgen(getter = useful_level)]
    pub fn get_useful_level(&self) -> u8 {
        self.core.useful_level
    }
    #[wasm_bindgen(getter = prior_game_board)]
    pub fn get_prior_game_board(&self) -> GameBoard {
        let t = self.core.prior_game_board.as_ref().unwrap().borrow();
        GameBoard { core: t.clone() }
    }
    #[wasm_bindgen(getter = next_game_board)]
    pub fn get_next_game_board(&self) -> GameBoard {
        let t = self.core.next_game_board.as_ref().unwrap().borrow();
        GameBoard { core: t.clone() }
    }
    #[wasm_bindgen(getter = comments)]
    pub fn get_comments(&self) -> String {
        self.core.comments.clone()
    }
    #[wasm_bindgen(getter = path)]
    pub fn get_path(&self) -> f64 {
        self.core.path
    }
    #[wasm_bindgen(getter = mouse_state)]
    pub fn get_mouse_state(&self) -> usize {
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
    #[wasm_bindgen(getter = key_dynamic_params)]
    pub fn get_key_dynamic_params(&self) -> KeyDynamicParams {
        KeyDynamicParams {
            core: self.core.key_dynamic_params.clone(),
        }
    }
}

// 定义宏，生成所有类型录像的子类
macro_rules! generate_video {
    ($($some_video:ident),*) => {
        $(
            #[wasm_bindgen(inspectable)]
            pub struct $some_video {
                core: ms::$some_video,
            }
            #[wasm_bindgen]
            impl $some_video {
                #[wasm_bindgen(constructor)]
                pub fn new(data: Box<[u8]>, file_name: &str) -> $some_video {
                    let data = data.into_vec();
                    $some_video {
                        core: ms::$some_video::new(data, file_name),
                    }
                }
                pub fn parse(&mut self) {
                    self.core.parse().unwrap();
                }
                pub fn analyse(&mut self) {
                    self.core.data.analyse();
                }
                #[wasm_bindgen(getter = raw_data)]
                pub fn get_raw_data(&self) -> Vec<u8> {
                    self.core.data.get_raw_data().unwrap()
                }
                #[wasm_bindgen(getter = software)]
                pub fn get_software(&self) -> String {
                    self.core.data.software.clone()
                }
                #[wasm_bindgen(getter = row)]
                pub fn get_row(&self) -> usize {
                    self.core.data.height
                }
                #[wasm_bindgen(getter = column)]
                pub fn get_column(&self) -> usize {
                    self.core.data.width
                }
                #[wasm_bindgen(getter = level)]
                pub fn get_level(&self) -> u8 {
                    self.core.data.level
                }
                #[wasm_bindgen(getter = mode)]
                pub fn get_mode(&self) -> u16 {
                    self.core.data.mode
                }
                #[wasm_bindgen(getter = is_completed)]
                pub fn get_is_completed(&self) -> bool {
                    self.core.data.is_completed
                }
                #[wasm_bindgen(getter = is_official)]
                pub fn get_is_official(&self) -> bool {
                    self.core.data.is_official
                }
                #[wasm_bindgen(getter = is_fair)]
                pub fn get_is_fair(&self) -> bool {
                    self.core.data.is_fair
                }
                #[wasm_bindgen(getter = mine_num)]
                pub fn get_mine_num(&self) -> usize {
                    self.core.data.mine_num
                }
                #[wasm_bindgen(getter = player_identifier)]
                pub fn get_player_identifier(&self) -> String {
                    self.core.data.player_identifier.clone()
                }
                #[wasm_bindgen(getter = race_identifier)]
                pub fn get_race_identifier(&self) -> String {
                    self.core.data.race_identifier.clone()
                }
                #[wasm_bindgen(getter = uniqueness_identifier)]
                pub fn get_uniqueness_identifier(&self) -> String {
                    self.core.data.uniqueness_identifier.clone()
                }
                #[wasm_bindgen(getter = country)]
                pub fn get_country(&self) -> String {
                    self.core.data.country.clone()
                }
                #[wasm_bindgen(getter = device_uuid)]
                pub fn get_device_uuid(&self) -> Vec<u8> {
                    self.core.data.device_uuid.clone()
                }
                #[wasm_bindgen(getter = bbbv)]
                pub fn get_bbbv(&self) -> usize {
                    self.core.data.static_params.bbbv
                }
                #[wasm_bindgen(getter = start_time)]
                pub fn get_start_time(&self) -> u64 {
                    self.core.data.start_time
                }
                #[wasm_bindgen(getter = end_time)]
                pub fn get_end_time(&self) -> u64 {
                    self.core.data.end_time
                }
                #[wasm_bindgen(getter = op)]
                pub fn get_op(&self) -> usize {
                    self.core.data.static_params.op
                }
                #[wasm_bindgen(getter = isl)]
                pub fn get_isl(&self) -> usize {
                    self.core.data.static_params.isl
                }
                #[wasm_bindgen(getter = hizi)]
                pub fn get_hizi(&self) -> usize {
                    self.core.data.static_params.hizi
                }
                #[wasm_bindgen(getter = cell0)]
                pub fn get_cell0(&self) -> usize {
                    self.core.data.static_params.cell0
                }
                #[wasm_bindgen(getter = cell1)]
                pub fn get_cell1(&self) -> usize {
                    self.core.data.static_params.cell1
                }
                #[wasm_bindgen(getter = cell2)]
                pub fn get_cell2(&self) -> usize {
                    self.core.data.static_params.cell2
                }
                #[wasm_bindgen(getter = cell3)]
                pub fn get_cell3(&self) -> usize {
                    self.core.data.static_params.cell3
                }
                #[wasm_bindgen(getter = cell4)]
                pub fn get_cell4(&self) -> usize {
                    self.core.data.static_params.cell4
                }
                #[wasm_bindgen(getter = cell5)]
                pub fn get_cell5(&self) -> usize {
                    self.core.data.static_params.cell5
                }
                #[wasm_bindgen(getter = cell6)]
                pub fn get_cell6(&self) -> usize {
                    self.core.data.static_params.cell6
                }
                #[wasm_bindgen(getter = cell7)]
                pub fn get_cell7(&self) -> usize {
                    self.core.data.static_params.cell7
                }
                #[wasm_bindgen(getter = cell8)]
                pub fn get_cell8(&self) -> usize {
                    self.core.data.static_params.cell8
                }
                #[wasm_bindgen(getter = rtime)]
                pub fn get_rtime(&self) -> f64 {
                    self.core.data.get_rtime().unwrap()
                }
                #[wasm_bindgen(getter = rtime_ms)]
                pub fn get_rtime_ms(&self) -> u32 {
                    self.core.data.get_rtime_ms().unwrap()
                }
                #[wasm_bindgen(getter = etime)]
                pub fn get_etime(&self) -> f64 {
                    self.core.data.get_etime().unwrap()
                }
                #[wasm_bindgen(getter = video_start_time)]
                pub fn get_video_start_time(&self) -> f64 {
                    self.core.data.get_video_start_time().unwrap()
                }
                #[wasm_bindgen(getter = video_end_time)]
                pub fn get_video_end_time(&self) -> f64 {
                    self.core.data.get_video_end_time().unwrap()
                }
                #[wasm_bindgen(getter = bbbv_s)]
                pub fn get_bbbv_s(&self) -> f64 {
                    self.core.data.get_bbbv_s().unwrap()
                }
                #[wasm_bindgen(getter = stnb)]
                pub fn get_stnb(&self) -> f64 {
                    self.core.data.get_stnb().unwrap()
                }
                #[wasm_bindgen(getter = rqp)]
                pub fn get_rqp(&self) -> f64 {
                    self.core.data.get_rqp().unwrap()
                }
                #[wasm_bindgen(getter = left)]
                pub fn get_left(&self) -> usize {
                    self.core.data.get_left()
                }
                #[wasm_bindgen(getter = right)]
                pub fn get_right(&self) -> usize {
                    self.core.data.get_right()
                }
                #[wasm_bindgen(getter = double)]
                pub fn get_double(&self) -> usize {
                    self.core.data.get_double()
                }
                #[wasm_bindgen(getter = cl)]
                pub fn get_cl(&self) -> usize {
                    self.core.data.get_cl()
                }
                #[wasm_bindgen(getter = flag)]
                pub fn get_flag(&self) -> usize {
                    self.core.data.get_flag()
                }
                #[wasm_bindgen(getter = bbbv_solved)]
                pub fn get_bbbv_solved(&self) -> usize {
                    self.core.data.get_bbbv_solved().unwrap()
                }
                #[wasm_bindgen(getter = lce)]
                pub fn get_lce(&self) -> usize {
                    self.core.data.get_lce().unwrap()
                }
                #[wasm_bindgen(getter = rce)]
                pub fn get_rce(&self) -> usize {
                    self.core.data.get_rce().unwrap()
                }
                #[wasm_bindgen(getter = dce)]
                pub fn get_dce(&self) -> usize {
                    self.core.data.get_dce().unwrap()
                }
                #[wasm_bindgen(getter = ce)]
                pub fn get_ce(&self) -> usize {
                    self.core.data.get_ce().unwrap()
                }
                #[wasm_bindgen(getter = left_s)]
                pub fn get_left_s(&self) -> f64 {
                    self.core.data.get_left_s()
                }
                #[wasm_bindgen(getter = right_s)]
                pub fn get_right_s(&self) -> f64 {
                    self.core.data.get_right_s()
                }
                #[wasm_bindgen(getter = double_s)]
                pub fn get_double_s(&self) -> f64 {
                    self.core.data.get_double_s()
                }
                #[wasm_bindgen(getter = cl_s)]
                pub fn get_cl_s(&self) -> f64 {
                    self.core.data.get_cl_s()
                }
                #[wasm_bindgen(getter = flag_s)]
                pub fn get_flag_s(&self) -> f64 {
                    self.core.data.get_flag_s()
                }
                #[wasm_bindgen(getter = path)]
                pub fn get_path(&self) -> f64 {
                    self.core.data.get_path()
                }
                #[wasm_bindgen(getter = ce_s)]
                pub fn get_ce_s(&self) -> f64 {
                    self.core.data.get_ce_s().unwrap()
                }
                #[wasm_bindgen(getter = ioe)]
                pub fn get_ioe(&self) -> f64 {
                    self.core.data.get_ioe().unwrap()
                }
                #[wasm_bindgen(getter = thrp)]
                pub fn get_thrp(&self) -> f64 {
                    self.core.data.get_thrp().unwrap()
                }
                #[wasm_bindgen(getter = corr)]
                pub fn get_corr(&self) -> f64 {
                    self.core.data.get_corr().unwrap()
                }
                #[wasm_bindgen(getter = events)]
                pub fn get_events(&self) -> JsValue {
                    let array = Array::new();
                    for i in self.core.data.video_action_state_recorder.clone(){
                        let v = VideoActionStateRecorder{core: i};
                        array.push(&JsValue::from(v));
                    }
                    array.into()
                }
                // #[wasm_bindgen(getter = game_board_stream)]
                // pub fn get_game_board_stream(&self) -> JsValue {
                //     let array = Array::new();
                //     for i in self.core.data.game_board_stream.clone(){
                //         let v = GameBoard { core: i };
                //         array.push(&JsValue::from(v));
                //     }
                //     array.into()
                // }
                #[wasm_bindgen(getter = current_event_id)]
                pub fn get_current_event_id(&self) -> usize {
                    self.core.data.current_event_id
                }
                #[wasm_bindgen(setter = current_event_id)]
                pub fn set_current_event_id(&mut self, id: usize) {
                    self.core.data.current_event_id = id
                }
                #[wasm_bindgen(getter = game_board)]
                pub fn get_game_board(&self) -> JsValue {
                    vec_vec_to_js_value(self.core.data.get_game_board().clone())
                }
                #[wasm_bindgen(getter = game_board_poss)]
                pub fn get_game_board_poss(&mut self) -> JsValue {
                    vec_vec_to_js_value(self.core.data.get_game_board_poss().clone())
                }
                #[wasm_bindgen(getter = mouse_state)]
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
                #[wasm_bindgen(getter = game_board_state)]
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
                #[wasm_bindgen(getter = x_y)]
                pub fn get_x_y(&self) -> CursorPos {
                    let (x, y) = self.core.data.get_x_y().unwrap();
                    CursorPos{x, y}
                }
                #[wasm_bindgen(getter = checksum)]
                pub fn get_checksum(&self) -> Vec<u8> {
                    self.core.data.get_checksum().unwrap().to_vec()
                }
                #[wasm_bindgen(getter = pix_size)]
                pub fn get_pix_size(&self) -> u8 {
                    self.core.data.get_pix_size().unwrap()
                }
                #[wasm_bindgen(setter = current_time)]
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
