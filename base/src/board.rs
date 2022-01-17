use crate::algorithms::{
    cal_possibility_onboard, mark_board, solve_direct, solve_enumerate, solve_minus,
};
use crate::analyse_methods::{
    analyse_high_risk_guess, analyse_jump_judge, analyse_mouse_trace, analyse_needless_guess,
    analyse_vision_transfer, analyse_survive_poss
};
use crate::utils::{refresh_board, refresh_matrixs};
use std::cmp::{max, min};
use std::fs;

/// 局面状态机，分析操作与局面的交互、推衍局面。在线地统计左右双击次数、ce次数、左键、右键、双击、当前解决的3BV。  
/// - 注意：ce的计算与扫雷网是不同的，本工具箱中，重复标同一个雷只算一个ce，即反复标雷、取消标雷不算作ce
pub struct MinesweeperBoard<'a> {
    board: &'a Vec<Vec<i32>>,
    /// 局面
    pub game_board: Vec<Vec<i32>>,
    flagedList: Vec<(usize, usize)>, // 记录哪些雷曾经被标过，则再标这些雷不记为ce
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
    row: usize,
    column: usize,
    mouse_state: MouseState,
}

impl MinesweeperBoard<'_> {
    pub fn new(board: &Vec<Vec<i32>>) -> MinesweeperBoard {
        let row = board.len();
        let column = board[0].len();
        MinesweeperBoard {
            board: board,
            row: row,
            column: column,
            game_board: vec![vec![10; column]; row],
            left: 0,
            right: 0,
            chording: 0,
            ces: 0,
            flag: 0,
            solved3BV: 0,
            flagedList: vec![],
            mouse_state: MouseState::UpUp,
        }
    }
    fn left_click(&mut self, x: usize, y: usize) -> Result<u8, ()> {
        self.left += 1;
        if self.game_board[x][y] != 10 {
            return Ok(0);
        }
        match self.board[x][y] {
            0 => {
                self.solved3BV += 1;
                self.ces += 1;
                refresh_board(&self.board, &mut self.game_board, vec![(x, y)]);
                Ok(2)
            }
            -1 => Err(()),
            _ => {
                refresh_board(&self.board, &mut self.game_board, vec![(x, y)]);
                if self.num_is_3BV(x, y) {
                    self.solved3BV += 1;
                }
                self.ces += 1;
                Ok(2)
            }
        }
    }
    fn right_click(&mut self, x: usize, y: usize) -> Result<u8, ()> {
        self.right += 1;
        if self.game_board[x][y] < 10 {
            return Ok(0);
        } else {
            if self.board[x][y] != -1 {
                match self.game_board[x][y] {
                    10 => {
                        self.game_board[x][y] = 11;
                        self.flag += 1;
                    }
                    11 => {
                        self.game_board[x][y] = 10;
                        self.flag -= 1;
                    }
                    _ => return Err(()),
                }
            } else {
                match self.game_board[x][y] {
                    10 => {
                        self.game_board[x][y] = 11;
                        self.flag += 1;
                        if !self.flagedList.contains(&(x, y)) {
                            self.ces += 1;
                        }
                        self.flagedList.push((x, y));
                    }
                    11 => {
                        self.game_board[x][y] = 10;
                        self.flag -= 1;
                    }
                    _ => return Err(()),
                }
            }
            Ok(1)
        }
    }
    fn chording_click(&mut self, x: usize, y: usize) -> Result<u8, ()> {
        self.chording += 1;
        if self.game_board[x][y] == 0 || self.game_board[x][y] >= 8 {
            return Ok(0);
        }
        let mut flagChordingUseful = false; // 双击有效的基础上，周围是否有未打开的格子
        let mut chordingCells = vec![]; // 未打开的格子的集合
        let mut flagedNum = 0; // 双击点周围的标雷数
        let mut surround3BV = 0; // 周围的3BV
        let mut flag_ch_op = false; // 是否通过双击开空了：一次双击最多打开一个空
        for i in max(1, x) - 1..min(self.row, x + 2) {
            for j in max(1, y) - 1..min(self.column, y + 2) {
                if i != x || j != y {
                    if self.game_board[i][j] == 11 {
                        flagedNum += 1
                    }
                    if self.game_board[i][j] == 10 && self.board[i][j] != -1 {
                        if self.board[i][j] == 0 {
                            flag_ch_op = true;
                        }
                        flagChordingUseful = true;
                        chordingCells.push((i, j));
                        if self.num_is_3BV(i, j) {
                            surround3BV += 1;
                        }
                    }
                }
            }
        }
        if flagedNum == self.game_board[x][y] && flagChordingUseful {
            self.ces += 1;
            self.solved3BV += surround3BV;
            if flag_ch_op {
                self.solved3BV += 1;
            }
            refresh_board(&self.board, &mut self.game_board, chordingCells);
            Ok(2)
        } else {
            Ok(0)
        }
    }
    fn num_is_3BV(&self, x: usize, y: usize) -> bool {
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
    pub fn step(&mut self, e: &str, pos: (usize, usize)) -> Result<u8, ()> {
        if pos.0 == self.row && pos.1 == self.column {
            self.mouse_state = MouseState::UpUp;
            return Ok(0u8)
        }
        match e {
            "lc" => match self.mouse_state {
                MouseState::UpUp => self.mouse_state = MouseState::DownUp,
                MouseState::UpDown => self.mouse_state = MouseState::Chording,
                MouseState::UpDownNotFlag => self.mouse_state = MouseState::ChordingNotFlag,
                _ => return Err(()),
            },
            "lr" => match self.mouse_state {
                MouseState::DownUp => {
                    self.mouse_state = MouseState::UpUp;
                    return self.left_click(pos.0, pos.1);
                }
                MouseState::Chording => {
                    self.mouse_state = MouseState::UpDown;
                    return self.chording_click(pos.0, pos.1);
                }
                MouseState::DownUpAfterChording => self.mouse_state = MouseState::UpUp,
                MouseState::ChordingNotFlag => {
                    self.mouse_state = MouseState::UpDown;
                    self.right -= 1;
                    return self.chording_click(pos.0, pos.1);
                }
                _ => return Err(()),
            },
            "rc" => match self.mouse_state {
                MouseState::UpUp => {
                    if self.game_board[pos.0][pos.1] < 10 {
                        self.mouse_state = MouseState::UpDownNotFlag;
                    } else {
                        self.mouse_state = MouseState::UpDown;
                    }
                    return self.right_click(pos.0, pos.1);
                }
                MouseState::DownUp => self.mouse_state = MouseState::Chording,
                MouseState::DownUpAfterChording => self.mouse_state = MouseState::Chording,
                _ => return Err(()),
            },
            "rr" => match self.mouse_state {
                MouseState::UpDown => self.mouse_state = MouseState::UpUp,
                MouseState::UpDownNotFlag => self.mouse_state = MouseState::UpUp,
                MouseState::Chording => {
                    self.mouse_state = MouseState::DownUpAfterChording;
                    return self.chording_click(pos.0, pos.1);
                }
                MouseState::ChordingNotFlag => {
                    self.mouse_state = MouseState::DownUpAfterChording;
                    self.right -= 1;
                    return self.chording_click(pos.0, pos.1);
                }
                _ => return Err(()),
            },
            _ => return Err(()),
        }
        Ok(0)
    }
    pub fn step_flow(&mut self, operation: Vec<(&str, (usize, usize))>) -> Result<(), ()> {
        // 直接分析整局的操作流，中间也可以停顿
        for op in operation {
            self.step(op.0, op.1)?;
        }
        Ok(())
    }
    // pub fn reset(&self) {
    //     // 重载，暂时没用不写
    // }
}

// #[derive(Debug, PartialEq)]
// pub struct UpUp;

#[derive(Debug, PartialEq)]
enum MouseState {
    // 鼠标状态机
    UpUp,
    UpDown,
    UpDownNotFlag, // 右键按下，且没有标雷的状态
    DownUp,
    Chording,            // 双键都按下，的其他状态
    ChordingNotFlag,     // 双键都按下，且是在不可以右击的格子上、先按下右键
    DownUpAfterChording, //双击后先弹起右键，左键还没弹起的状态
}

#[derive(Debug)]
pub enum ErrReadVideoReason {
    CanNotFindFile,
    FileIsTooShort,
    FileIsEmpty,
    FileHasJustOneByte,
    InvalidBoardSize,
    InvalidLevel,
    InvalidParams,
    InvalidVideoEvent,
}

pub struct VideoEvent<'a> {
    pub time: f64,
    pub mouse: &'a str,
    column: usize,
    row: usize,
    pub x: u16, // 单位是像素不是格
    pub y: u16,
    pub useful_level: u8, // 0代表完全没用，
    // 1代表能仅推进局面但不改变对局面的后验判断，例如标雷和取消标雷
    // 2代表改变对局面的后验判断的操作
    // 和ce没有关系，仅用于控制计算
    pub posteriori_game_board: GameBoard,
    // pub posteriori_game_board: Vec<Vec<i32>>,
    // pub posteriori_board_poss: Vec<Vec<f64>>,
    pub comments: String,
}

pub struct StaticParams {
    pub bbbv: usize,
    pub openings: usize,
    pub islands: usize,
    pub hizi: usize,
    pub cell_0: usize,
    pub cell_1: usize,
    pub cell_2: usize,
    pub cell_3: usize,
    pub cell_4: usize,
    pub cell_5: usize,
    pub cell_6: usize,
    pub cell_7: usize,
    pub cell_8: usize,
}

pub struct DynamicParams {
    pub r_time: f64,
    pub bbbv_s: f64,
    pub stnb: f64,
    pub rqp: f64,
    pub lefts: usize,
    pub rights: usize,
    pub chordings: usize,
    pub clicks: usize,
    pub flags: usize,
    pub ces: usize,
    pub lefts_s: f64,
    pub rights_s: f64,
    pub chordings_s: f64,
    pub clicks_s: f64,
    pub ces_s: f64,
}

trait BaseParser {
    fn get_unsized_int(&mut self) -> Result<u8, ErrReadVideoReason>;
    fn get_unsized_int2(&mut self) -> Result<u16, ErrReadVideoReason>;
    fn get_unsized_int4(&mut self) -> Result<u32, ErrReadVideoReason>;
    fn get_char(&mut self) -> Result<char, ErrReadVideoReason>;
}

/// avf录像解析器。  
/// - 功能：解析avf格式的录像，有详细分析录像的方法。  
pub struct AvfVideo<'a> {
    file_name: &'a str,
    width: usize,
    height: usize,
    mine_num: usize,
    marks: bool,
    level: usize,
    board: Vec<Vec<i32>>,
    pub events: Vec<VideoEvent<'a>>,
    player: &'a str,
    video_data: Vec<u8>,
    offset: usize,
    pub static_params: StaticParams,
    pub dynamic_params: DynamicParams,
}

impl BaseParser for AvfVideo<'_> {
    fn get_unsized_int(&mut self) -> Result<u8, ErrReadVideoReason> {
        let t = self.video_data.get(self.offset);
        self.offset += 1;
        match t {
            Some(x) => Ok(*x),
            None => Err(ErrReadVideoReason::FileIsTooShort),
        }
    }
    fn get_unsized_int2(&mut self) -> Result<u16, ErrReadVideoReason> {
        let a = self.get_unsized_int()?;
        let b = self.get_unsized_int()?;
        Ok((a as u16) << 8 | (b as u16))
    }
    fn get_unsized_int4(&mut self) -> Result<u32, ErrReadVideoReason> {
        let a = self.get_unsized_int()?;
        let b = self.get_unsized_int()?;
        let c = self.get_unsized_int()?;
        let d = self.get_unsized_int()?;
        Ok((a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | (d as u32))
    }
    fn get_char(&mut self) -> Result<char, ErrReadVideoReason> {
        let a = self.get_unsized_int()?;
        Ok(a as char)
    }
}
impl AvfVideo<'_> {
    pub fn new(file_name: &str) -> AvfVideo {
        let video_data: Vec<u8> = fs::read(file_name).unwrap();
        // for i in 42642 - 500..42641 {
        //     print!("{:?}", video_data[i] as char);
        // }
        AvfVideo {
            file_name: file_name,
            width: 0,
            height: 0,
            mine_num: 0,
            marks: false,
            level: 0,
            board: vec![],
            events: vec![],
            player: "",
            video_data: video_data,
            offset: 0,
            static_params: StaticParams {
                bbbv: 0,
                openings: 0,
                islands: 0,
                hizi: 0,
                cell_0: 0,
                cell_1: 0,
                cell_2: 0,
                cell_3: 0,
                cell_4: 0,
                cell_5: 0,
                cell_6: 0,
                cell_7: 0,
                cell_8: 0,
            },
            dynamic_params: DynamicParams {
                r_time: 0.0,
                bbbv_s: 0.0,
                stnb: 0.0,
                rqp: 0.0,
                lefts: 0,
                rights: 0,
                chordings: 0,
                flags: 0,
                clicks: 0,
                ces: 0,
                lefts_s: 0.0,
                rights_s: 0.0,
                chordings_s: 0.0,
                clicks_s: 0.0,
                ces_s: 0.0,
            },
        }
    }
    pub fn parse_video(&mut self) -> Result<(), ErrReadVideoReason> {
        match self.get_unsized_int() {
            Ok(_) => {}
            Err(_) => return Err(ErrReadVideoReason::FileIsEmpty),
        };
        match self.get_unsized_int4() {
            Ok(_) => {}
            Err(_) => return Err(ErrReadVideoReason::FileHasJustOneByte),
        };
        self.level = self.get_unsized_int()? as usize;
        match self.level {
            3 => {
                self.width = 8;
                self.height = 8;
                self.mine_num = 10;
            }
            4 => {
                self.width = 16;
                self.height = 16;
                self.mine_num = 40;
            }
            5 => {
                self.width = 30;
                self.height = 16;
                self.mine_num = 99;
            }
            6 => {
                self.width = self.get_unsized_int()? as usize + 1;
                self.height = self.get_unsized_int()? as usize + 1;
                self.width = self.get_unsized_int2()? as usize;
            }
            _ => return Err(ErrReadVideoReason::InvalidLevel),
        }
        self.board = vec![vec![0; self.width]; self.height];
        for _ in 0..self.mine_num {
            let c = self.get_unsized_int()? as usize;
            let d = self.get_unsized_int()? as usize;
            self.board[c - 1][d - 1] = -1;
        }

        for x in 0..self.height {
            for y in 0..self.width {
                if self.board[x][y] == -1 {
                    for j in max(1, x) - 1..min(self.height, x + 2) {
                        for k in max(1, y) - 1..min(self.width, y + 2) {
                            if self.board[j][k] >= 0 {
                                self.board[j][k] += 1;
                            }
                        }
                    }
                }
            }
        } // 算数字
        let mut buffer: [char; 3] = ['\0', '\0', '\0'];
        loop {
            buffer[0] = buffer[1];
            buffer[1] = buffer[2];
            buffer[2] = self.get_char()?;
            if buffer[0] == '['
                && (buffer[1] == '0' || buffer[1] == '1' || buffer[1] == '2' || buffer[1] == '3')
                && buffer[2] == '|'
            {
                break;
            }
        }
        let mut buffer: [char; 2] = ['\0', '\0'];
        loop {
            buffer[0] = buffer[1];
            buffer[1] = self.get_char()?;
            if buffer[0] == '|' && buffer[1] == 'B' {
                break;
            }
        }
        let mut s: String = "".to_string();
        loop {
            let v = self.get_char()?;
            match v {
                'T' => break,
                _ => s.push(v),
            }
        }
        self.static_params.bbbv = match s.parse() {
            Ok(v) => v,
            Err(_) => return Err(ErrReadVideoReason::InvalidParams),
        };
        let mut s: String = "".to_string();
        loop {
            let v = self.get_char()?;
            match v {
                ']' => break,
                _ => s.push(v),
            }
        }
        self.dynamic_params.r_time = match s.parse::<f64>() {
            Ok(v) => v - 1.0,
            Err(_) => return Err(ErrReadVideoReason::InvalidParams),
        };
        let mut buffer = [0u8; 8];
        while buffer[2] != 1 || buffer[1] > 1 {
            buffer[0] = buffer[1];
            buffer[1] = buffer[2];
            buffer[2] = self.get_unsized_int()?;
        }
        for i in 3..8 {
            buffer[i] = self.get_unsized_int()?;
        }
        loop {
            self.events.push(VideoEvent {
                time: ((buffer[6] as u16) << 8 | buffer[2] as u16) as f64 - 1.0
                    + (buffer[4] as f64) / 100.0,
                mouse: match buffer[0] {
                    1 => "mv",
                    3 => "lc",
                    5 => "lr",
                    9 => "rc",
                    17 => "rr",
                    33 => "mc",
                    65 => "mr",
                    145 => "rc",
                    193 => "mr",
                    11 => "sc",
                    21 => "lr",
                    _ => return Err(ErrReadVideoReason::InvalidVideoEvent),
                },
                column: 0,
                row: 0,
                x: (buffer[1] as u16) << 8 | buffer[3] as u16,
                y: (buffer[5] as u16) << 8 | buffer[7] as u16,
                useful_level: 0,
                posteriori_game_board: GameBoard::new(self.mine_num),
                comments: "".to_string(),
            });
            for i in 0..8 {
                buffer[i] = self.get_unsized_int()?;
            }
            if buffer[2] == 0 && buffer[6] == 0 {
                break;
            }
        }
        // 后面还有皮肤和标识符，暂时就不解析了
        // for i in 0..1000 {
        //     for j in 0..8 {
        //         print!("{:?},", self.get_char().unwrap() as u8);
        //     }
        //     println!("");
        // }
        Ok(())
    }
    /// 进行局面的推衍，计算基本的局面参数。不包含概率计算。
    pub fn analyse(&mut self) {
        let mut b = MinesweeperBoard::new(&self.board);
        for ide in 0..self.events.len() {
            if self.events[ide].mouse != "mv" {
                self.events[ide].useful_level = b
                    .step(
                        self.events[ide].mouse,
                        (
                            (self.events[ide].y / 16) as usize,
                            (self.events[ide].x / 16) as usize,
                        ),
                    )
                    .unwrap();
                self.events[ide]
                    .posteriori_game_board
                    .set_game_board(&b.game_board);
            }
        }
        self.dynamic_params.lefts = b.left;
        self.dynamic_params.rights = b.right;
        self.dynamic_params.ces = b.ces;
        self.dynamic_params.chordings = b.chording;
        self.dynamic_params.flags = b.flag;
    }
    pub fn analyse_for_features(&mut self, controller: Vec<&str>) {
        // 事件分析，返回一个向量，格式是event索引、字符串event的类型
        // error: 高风险的猜雷（猜对概率0.05）
        // good: 跳判
        // error: 判雷失败
        // good: 双线操作
        // good: 破空（成功率0.98）
        // error: 鼠标大幅绕路(500%)
        // warning：鼠标绕路(200%)
        // warning: 可以判雷时选择猜雷
        // suspect: 点击速度过快(0.01)
        // suspect: 鼠标移动过快(2)
        //
        for o in controller {
            match o {
                "high_risk_guess" => analyse_high_risk_guess(self),
                "jump_judge" => analyse_jump_judge(self),
                "needless_guess" => analyse_needless_guess(self),
                "mouse_trace" => analyse_mouse_trace(self),
                "vision_transfer" => analyse_vision_transfer(self),
                "survive_poss" => analyse_survive_poss(self),
                _ => continue,
            };
        }
    }
    pub fn print_event(&self) {
        for e in &self.events {
            if e.mouse != "mv" {
                println!("time = {:?}, mouse = {:?}, x = {:?}, y = {:?}", e.time, e.mouse, e.x, e.y);
                e.posteriori_game_board
                    .game_board
                    .iter()
                    .for_each(|v| println!("{:?}", v));
                e.posteriori_game_board
                    .poss
                    .iter()
                    .for_each(|v| println!("{:?}", v));
            }
        }
    }
    pub fn print_comments(&self) {
        for i in &self.events {
            if !i.comments.is_empty() {
                println!("{:?} => {:?}", i.time, i.comments);
            }
        }
    }
}

/// 静态游戏局面的包装类
/// 所有计算过的属性都会保存在这里
pub struct GameBoard {
    game_board: Vec<Vec<i32>>,
    game_board_marked: Vec<Vec<i32>>,
    mine_num: usize,
    poss: Vec<Vec<f64>>,
    is_marked: bool, // game_board_marked是否被完全标记过
    has_poss: bool,  // 是否已经计算过概率
    basic_not_mine: Vec<(usize, usize)>,
    basic_is_mine: Vec<(usize, usize)>,
    enum_not_mine: Vec<(usize, usize)>,
    enum_is_mine: Vec<(usize, usize)>,
}

impl GameBoard {
    pub fn new(mine_num: usize) -> GameBoard {
        GameBoard {
            game_board: vec![],
            game_board_marked: vec![],
            mine_num: mine_num,
            poss: vec![],
            is_marked: false,
            has_poss: false,
            basic_is_mine: vec![],
            basic_not_mine: vec![],
            enum_is_mine: vec![],
            enum_not_mine: vec![],
        }
    }
    pub fn set_game_board(&mut self, board: &Vec<Vec<i32>>) {
        let mut game_board_marked = board.clone();
        for i in 0..game_board_marked.len() {
            for j in 0..game_board_marked[0].len() {
                if game_board_marked[i][j] > 10 {
                    game_board_marked[i][j] = 10;
                }
            }
        }
        self.game_board = board.clone();
        self.game_board_marked = game_board_marked;
    }
    fn mark(&mut self) {
        // 一旦被标记，那么就会用3大判雷引擎都分析一遍
        // 相关参数都会计算并记录下来，is_marked也会改成true
        let (mut a_s, mut x_s, mut b_s, _, _) = refresh_matrixs(&self.game_board_marked);
        let mut ans = solve_direct(&mut a_s, &mut x_s, &mut b_s, &mut self.game_board_marked).0;
        self.basic_not_mine.append(&mut ans);

        let mut ans = solve_minus(&mut a_s, &mut x_s, &mut b_s, &mut self.game_board_marked).0;
        self.basic_not_mine.append(&mut ans);
        for i in &self.basic_not_mine {
            self.game_board_marked[i.0][i.1] = 12;
        }
        for i in 0..self.game_board_marked.len() {
            for j in 0..self.game_board_marked[0].len() {
                if self.game_board_marked[i][j] == 11 {
                    self.basic_is_mine.push((i, j));
                }
            }
        }
        self.enum_not_mine = solve_enumerate(&a_s, &x_s, &b_s, 40).0;
        // println!("yyyyyyyyyyyyyyyyy");
        for i in 0..self.game_board_marked.len() {
            for j in 0..self.game_board_marked[0].len() {
                if self.game_board_marked[i][j] == 11 && !self.basic_is_mine.contains(&(i, j)) {
                    self.enum_is_mine.push((i, j));
                }
            }
        }
        self.is_marked = true;
    }
    pub fn get_poss(&mut self) -> &Vec<Vec<f64>> {
        if !self.has_poss {
            if !self.is_marked {
                self.mark();
                self.is_marked = true;
            }
            self.poss = cal_possibility_onboard(&self.game_board_marked, self.mine_num as f64)
                .unwrap()
                .0;
            self.has_poss = true;
        }
        &self.poss
    }
    pub fn get_basic_not_mine(&mut self) -> &Vec<(usize, usize)> {
        if !self.is_marked {
            self.mark();
            self.is_marked = true;
        }
        &self.basic_not_mine
    }
    pub fn get_basic_is_mine(&mut self) -> &Vec<(usize, usize)> {
        if !self.is_marked {
            self.mark();
            self.is_marked = true;
        }
        &self.basic_is_mine
    }
    pub fn get_enum_not_mine(&mut self) -> &Vec<(usize, usize)> {
        if !self.is_marked {
            self.mark();
            self.is_marked = true;
        }
        &self.enum_not_mine
    }
    pub fn get_enum_is_mine(&mut self) -> &Vec<(usize, usize)> {
        if !self.is_marked {
            self.mark();
            self.is_marked = true;
        }
        &self.enum_is_mine
    }
}
