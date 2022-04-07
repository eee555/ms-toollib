use crate::algorithms::{
    cal_possibility_onboard, mark_board, solve_direct, solve_enumerate, solve_minus,
};
use crate::analyse_methods::{
    analyse_high_risk_guess, analyse_jump_judge, analyse_mouse_trace, analyse_needless_guess,
    analyse_survive_poss, analyse_vision_transfer,
};
use crate::utils::{refresh_board, refresh_matrixs};
use std::cmp::{max, min};
use std::fs;

/// 局面状态机，侧重分析操作与局面的交互、推衍局面。在线地统计左右双击次数、ce次数、左键、右键、双击、当前解决的3BV。  
/// - 局限：不关注具体的线路，因此不能计算path等。  
/// - 注意：ce的计算与扫雷网是不同的，本工具箱中，重复标同一个雷只算一个ce，即反复标雷、取消标雷不算作ce。
/// 应用场景：强化学习训练AI、游戏复盘计算指标。不能处理高亮（18）、算法确定是雷（12）等标记。  
/// - 用python调用时的示例：
/// ```python
/// import ms_toollib as ms
/// board = [
///     [0, 0, 1, -1, 2, 1, 1, -1],
///     [0, 0, 2, 3, -1, 3, 3, 2],
///     [1, 1, 3, -1, 4, -1, -1, 2],
///     [2, -1, 4, -1, 3, 4, -1, 4],
///     [3, -1, 5, 2, 1, 3, -1, -1],
///     [3, -1, -1, 2, 1, 2, -1, 3],
///     [-1, 5, 4, -1, 1, 1, 2, 2],
///     [-1, 3, -1, 2, 1, 0, 1, -1],
///     ];
/// v = ms.MinesweeperBoard(board)
/// v.step('lc', (0, 0))
/// v.step('lr', (0, 0))
/// print('左键次数: ', v.left)
/// print('右键次数: ', v.right)
/// print('ce数: ', v.ces)
/// print('标雷数: ', v.flag)
/// print('解决3BV数: ', v.solved3BV)
/// print('局面: ', v.game_board)
/// ```
pub struct MinesweeperBoard {
    pub board: Vec<Vec<i32>>,
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
    pub row: usize,
    pub column: usize,
    pub mouse_state: MouseState,
    pub game_board_state: GameBoardState,
    // 指针，用于判断局面是否全部扫开
    pointer_x: usize,
    pointer_y: usize,
}

impl MinesweeperBoard {
    pub fn new(board: Vec<Vec<i32>>) -> MinesweeperBoard {
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
            game_board_state: GameBoardState::Ready,
            pointer_x: 0,
            pointer_y: 0,
        }
    }
    fn left_click(&mut self, x: usize, y: usize) -> Result<u8, ()> {
        self.left += 1;
        if self.game_board[x][y] != 10 {
            return Ok(0);
        }
        refresh_board(&self.board, &mut self.game_board, vec![(x, y)]);
        match self.board[x][y] {
            0 => {
                self.solved3BV += 1;
                self.ces += 1;
                // refresh_board(&self.board, &mut self.game_board, vec![(x, y)]);
                if self.is_win() {
                    self.game_board_state = GameBoardState::Win;
                }
                Ok(2)
            }
            -1 => {
                // refresh_board(&self.board, &mut self.game_board, vec![(x, y)]);
                self.game_board_state = GameBoardState::Loss;
                Ok(0)
            }
            _ => {
                // refresh_board(&self.board, &mut self.game_board, vec![(x, y)]);
                if self.num_is_3BV(x, y) {
                    self.solved3BV += 1;
                }
                self.ces += 1;
                if self.is_win() {
                    self.game_board_state = GameBoardState::Win;
                }
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
                    if self.game_board[i][j] == 10 {
                        chordingCells.push((i, j));
                        flagChordingUseful = true;
                        if self.board[i][j] > 0 {
                            if self.num_is_3BV(i, j) {
                                surround3BV += 1;
                            }
                        } else if self.board[i][j] == 0 {
                            flag_ch_op = true;
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
            for ch in &chordingCells {
                if self.board[ch.0][ch.1] == -1 {
                    self.game_board_state = GameBoardState::Loss;
                }
            }
            refresh_board(&self.board, &mut self.game_board, chordingCells);
            if self.is_win() {
                self.game_board_state = GameBoardState::Win;
            }
            Ok(2)
        } else {
            Ok(0)
        }
    }
    fn num_is_3BV(&self, x: usize, y: usize) -> bool {
        // 判断该大于0的数字是不是3BV
        // 如果是0，即使是3bv，依然返回false
        if self.board[x][y] <= 0 {
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
    /// 返回的值的含义是：0：没有任何作用的操作，例如左键数字、踩雷。  
    /// 1：推进了局面，但没有改变ai对局面的判断，特指标雷。  
    /// 2：改变局面的操作，左键、双击。  
    /// e的类型有6种，lc, lr, rc, rr, cc, cr；这和arbiter等软件是不同的，本工具箱兼容了他们  
    /// cc对应的是，按下第二个键，至于按的哪个不知道  
    /// crl对应的是，两个键都按下时，抬起左键(chording_release_left)，和lr等价，可用可不用  
    /// crr对应的是，两个键都按下时，抬起右键(chording_release_right)，和rr等价，可用可不用  
    /// 在严格的鼠标状态机中，有些情况是不可能的，例如右键没有抬起就按下两次，但在阿比特中就观察到这种事情。
    /// 因此此类情况不再简单地报出不可恢复地错误，而是若无其事地继续解析。  
    pub fn step(&mut self, e: &str, pos: (usize, usize)) -> Result<u8, ()> {
        match self.game_board_state {
            GameBoardState::Ready => match e {
                "lr" => match self.mouse_state {
                    // 鼠标状态与局面状态的耦合
                    MouseState::DownUp => self.game_board_state = GameBoardState::Playing,
                    _ => {}
                },
                _ => {}
            },
            GameBoardState::Playing => {}
            _ => return Ok(0),
        }
        // if pos.0 == self.row && pos.1 == self.column {
        //     // 发生这种事情，是由于阿比特会把点到局面外，改成点到最右下角
        //     self.mouse_state = MouseState::UpUp;
        //     return Ok(0);
        // }
        // if pos.0 >= self.row || pos.1 >= self.column {
        //     // 越界错误，未定义的行为，不可恢复的错误
        //     return Err(());
        // }
        match e {
            "lc" => match self.mouse_state {
                MouseState::UpUp => self.mouse_state = MouseState::DownUp,
                MouseState::UpDown => self.mouse_state = MouseState::Chording,
                MouseState::UpDownNotFlag => self.mouse_state = MouseState::ChordingNotFlag,
                // 以下情况其实是不可能的
                MouseState::DownUp => {}
                MouseState::DownUpAfterChording => {}
                MouseState::Chording => {}
                MouseState::ChordingNotFlag => {}
                MouseState::Undefined => self.mouse_state = MouseState::DownUp,
            },
            "lr" => match self.mouse_state {
                MouseState::DownUp => {
                    self.mouse_state = MouseState::UpUp;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.left_click(pos.0, pos.1);
                }
                MouseState::Chording => {
                    self.mouse_state = MouseState::UpDown;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.chording_click(pos.0, pos.1);
                }
                MouseState::DownUpAfterChording => self.mouse_state = MouseState::UpUp,
                MouseState::ChordingNotFlag => {
                    self.mouse_state = MouseState::UpDown;
                    self.right -= 1;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.chording_click(pos.0, pos.1);
                }
                // 以下情况其实是不可能的
                MouseState::UpDown => {}
                MouseState::UpDownNotFlag => {}
                MouseState::UpUp => self.mouse_state = MouseState::UpUp,
                MouseState::Undefined => self.mouse_state = MouseState::UpUp,
            },
            "rc" => match self.mouse_state {
                MouseState::UpUp => {
                    if pos.0 == self.row && pos.1 == self.column {
                        self.mouse_state = MouseState::UpDownNotFlag;
                        return Ok(0);
                    }
                    if self.game_board[pos.0][pos.1] < 10 {
                        self.mouse_state = MouseState::UpDownNotFlag;
                    } else {
                        self.mouse_state = MouseState::UpDown;
                    }
                    return self.right_click(pos.0, pos.1);
                }
                MouseState::DownUp => self.mouse_state = MouseState::Chording,
                MouseState::DownUpAfterChording => self.mouse_state = MouseState::Chording,
                // 以下情况其实是不可能的
                MouseState::UpDown => {}
                MouseState::UpDownNotFlag => {}
                MouseState::Chording => {}
                MouseState::ChordingNotFlag => {}
                MouseState::Undefined => self.mouse_state = MouseState::UpDown,
            },
            "rr" => match self.mouse_state {
                MouseState::UpDown => self.mouse_state = MouseState::UpUp,
                MouseState::UpDownNotFlag => self.mouse_state = MouseState::UpUp,
                MouseState::Chording => {
                    self.mouse_state = MouseState::DownUpAfterChording;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.chording_click(pos.0, pos.1);
                }
                MouseState::ChordingNotFlag => {
                    self.mouse_state = MouseState::DownUpAfterChording;
                    self.right -= 1;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.chording_click(pos.0, pos.1);
                }
                // 以下情况其实是不可能的
                MouseState::DownUp => {}
                MouseState::DownUpAfterChording => {}
                MouseState::UpUp => self.mouse_state = MouseState::UpUp,
                MouseState::Undefined => self.mouse_state = MouseState::UpUp,
            },
            "cc" => match self.mouse_state {
                MouseState::DownUp => self.mouse_state = MouseState::Chording,
                MouseState::DownUpAfterChording => self.mouse_state = MouseState::Chording,
                MouseState::UpDown => self.mouse_state = MouseState::Chording,
                MouseState::UpDownNotFlag => self.mouse_state = MouseState::ChordingNotFlag,
                _ => return Err(()),
            },
            "crl" => match self.mouse_state {
                MouseState::Chording => {
                    self.mouse_state = MouseState::UpDown;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.chording_click(pos.0, pos.1);
                }
                MouseState::ChordingNotFlag => {
                    self.mouse_state = MouseState::UpDown;
                    self.right -= 1;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.chording_click(pos.0, pos.1);
                }
                _ => return Err(()),
            },
            "crr" => match self.mouse_state {
                MouseState::Chording => {
                    self.mouse_state = MouseState::DownUpAfterChording;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.chording_click(pos.0, pos.1);
                }
                MouseState::ChordingNotFlag => {
                    self.mouse_state = MouseState::DownUpAfterChording;
                    self.right -= 1;
                    if pos.0 == self.row && pos.1 == self.column {
                        return Ok(0);
                    }
                    return self.chording_click(pos.0, pos.1);
                }
                _ => return Err(()),
            },
            _ => return Err(()),
        }
        Ok(0)
    }
    /// 直接分析整局的操作流，中间也可以停顿
    /// 开始游戏前的任何操作也都记录次数
    pub fn step_flow(&mut self, operation: Vec<(&str, (usize, usize))>) -> Result<(), ()> {
        for op in operation {
            self.step(op.0, op.1)?;
        }
        Ok(())
    }
    fn is_win(&mut self) -> bool {
        for j in self.pointer_y..self.column {
            if self.game_board[self.pointer_x][j] >= 10 && self.board[self.pointer_x][j] != -1 {
                self.pointer_y = j;
                return false;
            }
        }
        for i in self.pointer_x + 1..self.row {
            for j in 0..self.column {
                if self.game_board[i][j] >= 10 && self.board[i][j] != -1 {
                    self.pointer_x = i;
                    self.pointer_y = j;
                    return false;
                }
            }
        }
        true
    }
    // pub fn reset(&self) {
    //     // 重载，暂时没用不写
    // }
}

/// 鼠标状态机
#[derive(Debug, PartialEq, Clone)]
pub enum MouseState {
    UpUp,
    UpDown,
    /// 右键按下，且既没有标雷，也没有取消标雷的状态
    UpDownNotFlag,
    DownUp,
    /// 双键都按下的其他状态
    Chording,
    /// 双键都按下，且是在不可以右击的格子上、先按下右键
    ChordingNotFlag,
    /// 双击后先弹起右键，左键还没弹起的状态
    DownUpAfterChording,
    /// 未初始化的状态
    Undefined,
}

/// 游戏局面状态机
#[derive(Debug, PartialEq)]
pub enum GameBoardState {
    Ready,
    Playing,
    Loss,
    Win,
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

pub struct VideoEvent {
    pub time: f64,
    pub mouse: String,
    /// 单位是像素不是格
    pub x: u16,
    pub y: u16,
    /// 0代表完全没用，
    /// 1代表能仅推进局面但不改变对局面的后验判断，例如标雷和取消标雷
    /// 2代表改变对局面的后验判断的操作
    /// 和ce没有关系，仅用于控制计算
    pub useful_level: u8,
    /// 当前局面。鼠标move是不会存的。
    pub posteriori_game_board: GameBoard,
    // pub posteriori_game_board: Vec<Vec<i32>>,
    // pub posteriori_board_poss: Vec<Vec<f64>>,
    pub comments: String,
    /// 该操作完成以后的鼠标状态。和录像高亮有关。即使是鼠标move也会记录。
    pub mouse_state: MouseState,
}

pub struct StaticParams {
    pub bbbv: usize,
    pub openings: usize,
    pub islands: usize,
    pub hizi: usize,
    pub cell0: usize,
    pub cell1: usize,
    pub cell2: usize,
    pub cell3: usize,
    pub cell4: usize,
    pub cell5: usize,
    pub cell6: usize,
    pub cell7: usize,
    pub cell8: usize,
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
    pub ioe: f64,
    pub corr: f64,
    pub thrp: f64,
}

trait BaseParser {
    fn get_unsized_int(&mut self) -> Result<u8, ErrReadVideoReason>;
    fn get_unsized_int2(&mut self) -> Result<u16, ErrReadVideoReason>;
    fn get_unsized_int4(&mut self) -> Result<u32, ErrReadVideoReason>;
    fn get_char(&mut self) -> Result<char, ErrReadVideoReason>;
}

/// avf录像解析器。  
/// - 功能：解析avf格式的录像，有详细分析录像的方法。  
/// - 以下是在python中调用的示例。
/// ```python
/// v = ms.AvfVideo("arbiter_beg.avf");
/// v.parse_video()
/// v.analyse()
/// print(v.bbbv)
/// print(v.clicks)
/// print(v.clicks_s)
/// v.analyse_for_features(["high_risk_guess"])
/// for i in range(v.events_len):
///     print(v.events_time(i), v.events_x(i), v.events_y(i), v.events_mouse(i))
/// for i in range(v.events_len):
///     if v.events_useful_level(i) >= 2:
///         print(v.events_posteriori_game_board(i).poss)
/// ```
pub struct AvfVideo {
    pub file_name: String,
    pub width: usize,
    pub height: usize,
    pub mine_num: usize,
    pub marks: bool,
    pub level: usize,
    pub board: Vec<Vec<i32>>,
    pub events: Vec<VideoEvent>,
    /// 录像播放时的指针，播放哪一帧
    pub current_event_id: usize,
    pub player: String,
    /// 游戏起始时间和中止时间。不整理格式，读成字符串。
    /// 举例：在阿比特中，‘16.10.2021.22.24.23.9906’，意味2021年10月16日，下午10点24分23秒9906。
    pub start_time: String,
    pub end_time: String,
    video_data: Vec<u8>,
    offset: usize,
    pub static_params: StaticParams,
    pub dynamic_params: DynamicParams,
}

impl BaseParser for AvfVideo {
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
impl AvfVideo {
    #[cfg(any(feature = "py", feature = "rs"))]
    pub fn new(file_name: &str) -> AvfVideo {
        let video_data: Vec<u8> = fs::read(file_name).unwrap();
        // for i in 0..500 {
        //     print!("{:?}", video_data[i] as char);
        // }
        AvfVideo {
            file_name: file_name.to_string(),
            width: 0,
            height: 0,
            mine_num: 0,
            marks: false,
            level: 0,
            board: vec![],
            events: vec![],
            current_event_id: 0,
            player: "".to_string(),
            start_time: "".to_string(),
            end_time: "".to_string(),
            video_data: video_data,
            offset: 0,
            static_params: StaticParams {
                bbbv: 0,
                openings: 0,
                islands: 0,
                hizi: 0,
                cell0: 0,
                cell1: 0,
                cell2: 0,
                cell3: 0,
                cell4: 0,
                cell5: 0,
                cell6: 0,
                cell7: 0,
                cell8: 0,
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
                ioe: 0.0,
                thrp: 0.0,
                corr: 0.0,
            },
        }
    }
    #[cfg(feature = "js")]
    pub fn new(video_data: Vec<u8>) -> AvfVideo {
        // video_data = video_data.into_vec();
        AvfVideo {
            file_name: "".to_string(),
            width: 0,
            height: 0,
            mine_num: 0,
            marks: false,
            level: 0,
            board: vec![],
            events: vec![],
            current_event_id: 0,
            player: "".to_string(),
            video_data: video_data,
            offset: 0,
            static_params: StaticParams {
                bbbv: 0,
                openings: 0,
                islands: 0,
                hizi: 0,
                cell0: 0,
                cell1: 0,
                cell2: 0,
                cell3: 0,
                cell4: 0,
                cell5: 0,
                cell6: 0,
                cell7: 0,
                cell8: 0,
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
        loop {
            let v = self.get_char()?;
            match v {
                '|' => break,
                _ => self.start_time.push(v),
            }
        }
        // println!("666");
        // loop {
        //     let v = self.get_char()?;
        //     print!("{:?}", v as char);
        // }
        loop {
            let v = self.get_char()?;
            match v {
                '|' => break,
                _ => self.end_time.push(v),
            }
        }
        let v = self.get_char()?;
        let mut buffer: [char; 2];
        match v {
            '|' => buffer = ['\0', '|'],
            'B' => buffer = ['|', 'B'],
            _ => buffer = ['\0', '\0'],
        }
        loop {
            if buffer[0] == '|' && buffer[1] == 'B' {
                break;
            }
            buffer[0] = buffer[1];
            buffer[1] = self.get_char()?;
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
        // println!("777");
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
            // if buffer[0] != 1 {
            // println!("{:?}, {:?}", ((buffer[6] as u16) << 8 | buffer[2] as u16) as f64 - 1.0
            // + (buffer[4] as f64) / 100.0, buffer[0]);}
            self.events.push(VideoEvent {
                time: ((buffer[6] as u16) << 8 | buffer[2] as u16) as f64 - 1.0
                    + (buffer[4] as f64) / 100.0,
                mouse: match buffer[0] {
                    1 => "mv".to_string(),
                    3 => "lc".to_string(),
                    5 => "lr".to_string(),
                    9 => "rc".to_string(),
                    17 => "rr".to_string(),
                    33 => "mc".to_string(),
                    65 => "mr".to_string(),
                    145 => "rr".to_string(),
                    193 => "mr".to_string(),
                    11 => "sc".to_string(),
                    21 => "lr".to_string(),
                    _ => return Err(ErrReadVideoReason::InvalidVideoEvent),
                },
                // column: 0,
                // row: 0,
                x: (buffer[1] as u16) << 8 | buffer[3] as u16,
                y: (buffer[5] as u16) << 8 | buffer[7] as u16,
                useful_level: 0,
                posteriori_game_board: GameBoard::new(self.mine_num),
                comments: "".to_string(),
                mouse_state: MouseState::Undefined,
            });
            for i in 0..8 {
                buffer[i] = self.get_unsized_int()?;
            }
            if buffer[2] == 0 && buffer[6] == 0 {
                break;
            }
        }
        // 标识符
        while self.get_char()? != 'S' {}
        while self.get_char()? != 'k' {}
        while self.get_char()? != 'i' {}
        while self.get_char()? != 'n' {}
        while self.get_char()? != ':' {}
        while self.get_char()? != '\r' {}
        loop {
            let v = self.get_char()?;
            match v {
                '\r' => break,
                _ => self.player.push(v),
            }
        }
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
        let mut b = MinesweeperBoard::new(self.board.clone());
        for ide in 0..self.events.len() {
            if self.events[ide].mouse != "mv" {
                // println!("{:?}, {:?}", self.events[ide].time, self.events[ide].mouse);
                self.events[ide].useful_level = b
                    .step(
                        &self.events[ide].mouse,
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
            self.events[ide].mouse_state = b.mouse_state.clone();
        }
        self.dynamic_params.lefts = b.left;
        self.dynamic_params.lefts_s = b.left as f64 / self.dynamic_params.r_time;
        self.dynamic_params.rights = b.right;
        self.dynamic_params.rights_s = b.right as f64 / self.dynamic_params.r_time;
        self.dynamic_params.ces = b.ces;
        self.dynamic_params.ces_s = b.ces as f64 / self.dynamic_params.r_time;
        self.dynamic_params.chordings = b.chording;
        self.dynamic_params.clicks = b.left + b.right + b.chording;
        self.dynamic_params.clicks_s =
            self.dynamic_params.clicks as f64 / self.dynamic_params.r_time;
        self.dynamic_params.flags = b.flag;
        self.dynamic_params.bbbv_s = self.static_params.bbbv as f64 / self.dynamic_params.r_time;
        self.dynamic_params.rqp = self.dynamic_params.r_time * self.dynamic_params.r_time
            / self.static_params.bbbv as f64;
        if self.height == 8 && self.width == 8 && self.mine_num == 10 {
            self.dynamic_params.stnb = 47.22
                / (self.dynamic_params.r_time.powf(1.7) / self.static_params.bbbv as f64)
                * (b.solved3BV as f64 / self.static_params.bbbv as f64);
        } else if self.height == 16 && self.width == 16 && self.mine_num == 40 {
            self.dynamic_params.stnb = 153.73
                / (self.dynamic_params.r_time.powf(1.7) / self.static_params.bbbv as f64)
                * (b.solved3BV as f64 / self.static_params.bbbv as f64);
        } else if self.height == 16 && self.width == 30 && self.mine_num == 99 {
            self.dynamic_params.stnb = 435.001
                / (self.dynamic_params.r_time.powf(1.7) / self.static_params.bbbv as f64)
                * (b.solved3BV as f64 / self.static_params.bbbv as f64);
        } // 凡自定义的stnb都等于0
        self.dynamic_params.ioe = b.solved3BV as f64 / self.dynamic_params.clicks as f64;
        self.dynamic_params.corr = b.ces as f64 / self.dynamic_params.clicks as f64;
        self.dynamic_params.thrp = b.solved3BV as f64 / b.ces as f64;
    }
    pub fn analyse_for_features(&mut self, controller: Vec<&str>) {
        // 事件分析，返回一个向量，格式是event索引、字符串event的类型
        // error: 高风险的猜雷（猜对概率0.05）
        // feature: 跳判
        // error: 判雷失败
        // feature: 双线操作
        // feature: 破空（成功率0.98）
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
        let mut num = 0;
        for e in &self.events {
            if num < 200 {
                println!(
                    "time = {:?}, mouse = {:?}, x = {:?}, y = {:?}, level = {:?}",
                    e.time, e.mouse, e.x, e.y, e.useful_level
                );
            }
            num += 1;
            // if e.mouse != "mv" {
            //     println!(
            //         "time = {:?}, mouse = {:?}, x = {:?}, y = {:?}",
            //         e.time, e.mouse, e.x, e.y
            //     );
            //     e.posteriori_game_board
            //         .game_board
            //         .iter()
            //         .for_each(|v| println!("{:?}", v));
            //     e.posteriori_game_board
            //         .poss
            //         .iter()
            //         .for_each(|v| println!("{:?}", v));
            // }
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

// 再实现一些get、set方法
impl AvfVideo {
    /// 获取当前录像时刻的游戏局面
    pub fn get_game_board(&self) -> Vec<Vec<i32>> {
        let mut id = self.current_event_id;
        loop {
            if self.events[id].posteriori_game_board.game_board.is_empty() {
                id -= 1;
            } else {
                return self.events[id].posteriori_game_board.game_board.clone();
            }
        }
    }
    /// 获取当前录像时刻的局面概率
    pub fn get_game_board_poss(&mut self) -> Vec<Vec<f64>> {
        let mut id = self.current_event_id;
        loop {
            if self.events[id].useful_level < 2 {
                id -= 1;
                if id <= 0 {
                    let p = self.mine_num as f64 / (self.height * self.width) as f64;
                    return vec![vec![p; self.height]; self.width];
                }
            } else {
                return self.events[id].posteriori_game_board.get_poss().clone();
            }
        }
    }
    /// 按时间设置current_event_id；超出两端范围取两端。
    pub fn set_current_event_time(&mut self, time: f64) {
        if time > self.events[self.current_event_id].time {
            loop {
                self.current_event_id += 1;
                if self.current_event_id >= self.events.len() {
                    self.current_event_id -= 1;
                    return;
                }
                if self.events[self.current_event_id].time < time {
                    continue;
                } else {
                    if self.events[self.current_event_id].time - time
                        >= time - self.events[self.current_event_id - 1].time
                    {
                        self.current_event_id -= 1;
                        return;
                    } else {
                        return;
                    }
                }
            }
        } else {
            loop {
                if self.current_event_id == 0 {
                    return;
                }
                self.current_event_id -= 1;
                if self.events[self.current_event_id].time > time {
                    continue;
                } else {
                    if time - self.events[self.current_event_id].time
                        >= self.events[self.current_event_id + 1].time - time
                    {
                        self.current_event_id += 1;
                        return;
                    } else {
                        return;
                    }
                }
            }
        }
    }
}

/// 静态游戏局面的包装类。  
/// 所有计算过的属性都会保存在这里。  
#[derive(Clone)]
pub struct GameBoard {
    pub game_board: Vec<Vec<i32>>,
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
        self.enum_not_mine = solve_enumerate(&a_s, &x_s, &b_s).0;
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
