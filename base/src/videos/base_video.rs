// 录像相关的类，局面在board

use crate::board::GameBoard;
use crate::miscellaneous::s_to_ms;
use crate::utils::{cal3BV, refresh_board};
use crate::videos::analyse_methods::{
    analyse_high_risk_guess, analyse_jump_judge, analyse_mouse_trace, analyse_needless_guess,
    analyse_super_fl_local, analyse_survive_poss, analyse_vision_transfer,
};
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
/// v = ms.MinesweeperBoard(board) # 实例化后再用
/// v.step('lc', (0, 0)) # 左键按下
/// v.step('lr', (0, 0)) # 左键弹起
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
            board,
            row,
            column,
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
            Ok(3)
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
    /// e的类型有8种，lc（左键按下）, lr（左键抬起）, rc（右键按下）, rr（右键抬起）, mc（中键按下）, mr（中键抬起）,   
    ///     cc（双键按下，但不确定是哪个键），pf（在开始前预先标雷，而又失去了标记的过程）；这和arbiter是略微不同的。  
    /// ## 注意事项：
    /// - 在严格的鼠标状态机中，有些情况是不可能的，例如右键没有抬起就按下两次，但在阿比特中就观察到这种事情。因此此类情况不再报成不可恢复的错误，而是若无其事地继续解析。  
    pub fn step(&mut self, e: &str, pos: (usize, usize)) -> Result<u8, ()> {
        match self.game_board_state {
            GameBoardState::Ready => {
                if e == "lr" && self.mouse_state == MouseState::DownUp {
                    if pos.0 == self.row && pos.1 == self.column {
                        self.mouse_state = MouseState::UpUp;
                        return Ok(0);
                    }
                    if self.game_board[pos.0][pos.1] == 10 {
                        self.game_board_state = GameBoardState::Playing;
                    }
                }
            }
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
                    // println!("x={:?}, y={:?}", pos.0, pos.1);
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
                        // 点在界面外
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
            "mc" => {}
            "mr" => {
                return self.chording_click(pos.0, pos.1);
            }
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
            _ => {
                // println!("{:?}", e);
                return Err(());
            }
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
    /// 初始化。对应强化学习领域gym的api中的reset。
    pub fn reset(&mut self) {
        self.game_board = vec![vec![10; self.column]; self.row];
        self.board = vec![vec![0; self.column]; self.row];
        self.left = 0;
        self.right = 0;
        self.chording = 0;
        self.ces = 0;
        self.flag = 0;
        self.left = 0;
        self.solved3BV = 0;
        self.flagedList = vec![];
        self.mouse_state = MouseState::UpUp;
        self.game_board_state = GameBoardState::Ready;
        self.pointer_x = 0;
        self.pointer_y = 0;
    }
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
    FileIsNotRmv,
    FileIsEmpty,
    InvalidBoardSize,
    InvalidLevel,
    InvalidParams,
    InvalidVideoEvent,
    InvalidMinePosition,
}

// 局面活动（点击或移动）的结构体
pub struct VideoEvent {
    pub time: f64,
    /// 操作类型，这几种："mv", "lc", "lr", "rc", "rr", "mc", "mr", "pf"
    pub mouse: String,
    /// 距离左端有几像素。
    pub x: u16,
    /// 距离上端有几像素。
    pub y: u16,
    /// 0代表完全没用，
    /// 1代表能仅推进局面但不改变对局面的后验判断，例如标雷和取消标雷
    /// 2代表改变对局面的后验判断的操作，例如左键点开一个或一片格子，不包括双击
    /// 3代表有效、至少打开了一个格子的双击
    /// 和ce没有关系，仅用于控制计算
    pub useful_level: u8,
    /// 操作前的局面（先验局面）的索引。
    pub prior_game_board_id: usize,
    /// 操作后的局面（后验的局面）的索引。
    pub next_game_board_id: usize,
    pub comments: String,
    /// 该操作完成以后的鼠标状态。和录像高亮有关。即使是鼠标move也会记录。
    pub mouse_state: MouseState,
    /// 该操作完成以后，已解决的3BV。
    pub solved3BV: usize,
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
    /// 以毫秒为单位的精确时间
    pub r_time_ms: u32,
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

pub struct BaseVideo {
    /// 软件名
    pub software: String,
    /// 宽度，等同于column
    pub width: usize,
    /// 高度，等同于row
    pub height: usize,
    /// 雷数
    pub mine_num: usize,
    // pub win: bool,
    /// 最后是否扫完，初始是false，踩雷的话，分析完还是false
    pub is_completed: bool,
    /// 是否正式。不一定具备参考价值。例如，没有人能证明avf录像是否正式。
    pub is_offical: bool,
    /// 是否正式。不一定具备参考价值。一些录像没有这个字段。
    pub is_fair: bool,
    /// 是不是盲扫，初始是false，不是盲扫的话，分析完还是false
    pub nf: bool,
    /// 游戏模式。0是标准；1是upk；2是cheat；3是Density（我也不知道什么意思）；其他以后加
    pub mode: u16,
    /// 游戏难度（级别）。3是初级；4是中级；5是高级；6是自定义。
    pub level: u8,
    /// 逻辑上，方格像素的尺寸。传统扫雷一般是16，只有元扫雷会把逻辑尺寸和实际尺寸统一起来。
    pub cell_pixel_size: u8,
    pub board: Vec<Vec<i32>>,
    pub events: Vec<VideoEvent>,
    /// 游戏局面流，从一开始没有打开任何格子（包含玩家游戏前的标雷过程），到最后打开了所有
    pub game_board_stream: Vec<GameBoard>,
    /// 录像播放时的指针，播放哪一帧
    pub current_event_id: usize,
    /// 录像用户标识
    pub player_designator: String,
    /// 比赛标识
    pub race_designator: String,
    /// 唯一性标识
    pub uniqueness_designator: String,
    /// 游戏起始时间和终止时间。不整理格式，读成字符串。
    /// 举例：在阿比特中，‘16.10.2021.22.24.23.9906’，意味2021年10月16日，下午10点24分23秒9906。
    /// 维也纳扫雷中，‘1382834716’，代表以秒为单位的时间戳
    pub start_time: String,
    /// 维也纳扫雷中没有
    pub end_time: String,
    /// 国家。预留字段，暂时不能解析。
    pub country: String,
    /// 原始二进制数据
    raw_data: Vec<u8>,
    pub offset: usize,
    /// 可以计算静态指标（有些没写完）
    pub static_params: StaticParams,
    /// 可以动态静态指标（有些没写完）
    pub dynamic_params: DynamicParams,
    // /// 开始扫前，已经标上的雷。如果操作流中包含标这些雷的过程，
    // pub pre_flags: Vec<(usize, usize)>,
    ///校验码
    pub checksum: String,
    pub can_analyse: bool,
}

impl BaseVideo {
    pub fn get_u8(&mut self) -> Result<u8, ErrReadVideoReason> {
        let t = self.raw_data.get(self.offset);
        self.offset += 1;
        match t {
            Some(x) => Ok(*x),
            None => Err(ErrReadVideoReason::FileIsTooShort),
        }
    }
    /// 都是大端法
    pub fn get_u16(&mut self) -> Result<u16, ErrReadVideoReason> {
        let a = self.get_u8()?;
        let b = self.get_u8()?;
        Ok((a as u16) << 8 | (b as u16))
    }
    pub fn get_u24(&mut self) -> Result<u32, ErrReadVideoReason> {
        let a = self.get_u8()?;
        let b = self.get_u8()?;
        let c = self.get_u8()?;
        Ok((a as u32) << 16 | (b as u32) << 8 | (c as u32))
    }
    pub fn get_u32(&mut self) -> Result<u32, ErrReadVideoReason> {
        let a = self.get_u8()?;
        let b = self.get_u8()?;
        let c = self.get_u8()?;
        let d = self.get_u8()?;
        Ok((a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | (d as u32))
    }
    pub fn get_char(&mut self) -> Result<char, ErrReadVideoReason> {
        let a = self.get_u8()?;
        Ok(a as char)
    }
}

impl BaseVideo {
    #[cfg(any(feature = "py", feature = "rs"))]
    pub fn new_with_file(file_name: &str) -> BaseVideo {
        let raw_data: Vec<u8> = fs::read(file_name).unwrap();
        // for i in 0..500 {
        //     print!("{:?}", raw_data[i] as char);
        // }
        BaseVideo {
            software: "".to_string(),
            width: 0,
            height: 0,
            mine_num: 0,
            is_completed: false,
            is_offical: false,
            is_fair: false,
            nf: false,
            mode: 0,
            level: 0,
            cell_pixel_size: 16,
            board: vec![],
            events: vec![],
            game_board_stream: vec![],
            current_event_id: 0,
            player_designator: "".to_string(),
            race_designator: "".to_string(),
            uniqueness_designator: "".to_string(),
            start_time: "".to_string(),
            end_time: "".to_string(),
            country: "".to_string(),
            raw_data: raw_data,
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
                r_time_ms: 0,
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
            checksum: "".to_string(),
            can_analyse: false,
        }
    }
    /// 通过游戏数据的构造函数。比较复杂。用在游戏里，扫完生成录像文件。
    /// - 如果有游戏前的标雷，要用"pf"在events里标
    #[cfg(any(feature = "py", feature = "rs"))]
    pub fn new_with_data(
        software: &str,
        board: Vec<Vec<i32>>,
        operate_stream: Vec<(f64, &str, u16, u16)>,
        r_time: f64,
        player_designator: &str,
        race_designator: &str,
        uniqueness_designator: &str,
        start_time: &str,
        end_time: &str,
        country: &str,
        level: u8,
        mode: u16,
        cell_pixel_size: u8,
        is_completed: bool,
        is_offical: bool,
        is_fair: bool,
        checksum: &str,
    ) -> BaseVideo {
        let mut events = vec![];
        let bbbv = cal3BV(&board);
        let mine_num = board.iter().fold(0, |y, row| {
            y + row
                .iter()
                .fold(0, |yy, x| if *x == -1 { yy + 1 } else { yy })
        });
        for operation in operate_stream {
            events.push(VideoEvent {
                time: operation.0,
                mouse: operation.1.to_string(),
                x: operation.2,
                y: operation.3,
                useful_level: 0,
                prior_game_board_id: 0,
                next_game_board_id: 0,
                comments: "".to_string(),
                mouse_state: MouseState::Undefined,
                solved3BV: 0,
            })
        }
        BaseVideo {
            software: software.to_string(),
            width: board[0].len(),
            height: board.len(),
            mine_num,
            is_completed,
            is_offical,
            is_fair,
            nf: false,
            mode,
            level,
            cell_pixel_size,
            board,
            events,
            game_board_stream: vec![],
            current_event_id: 0,
            player_designator: player_designator.to_string(),
            race_designator: race_designator.to_string(),
            uniqueness_designator: uniqueness_designator.to_string(),
            start_time: start_time.to_string(),
            end_time: end_time.to_string(),
            country: country.to_string(),
            raw_data: vec![],
            offset: 0,
            static_params: StaticParams {
                bbbv,
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
                r_time,
                r_time_ms: s_to_ms(r_time),
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
            checksum: checksum.to_string(),
            can_analyse: true,
        }
    }
    #[cfg(feature = "js")]
    pub fn new(video_data: Vec<u8>) -> BaseVideo {
        // video_data = video_data.into_vec();
        BaseVideo {
            file_name: "".to_string(),
            width: 0,
            height: 0,
            mine_num: 0,
            win: false,
            nf: false,
            mode: 0,
            level: 0,
            board: vec![],
            events: vec![],
            game_board_stream: vec![],
            current_event_id: 0,
            player: "".to_string(),
            start_time: "".to_string(),
            end_time: "".to_string(),
            country: "".to_string(),
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
                r_time_ms: 0,
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
            pre_flags: vec![],
        }
    }
    /// 进行局面的推衍，计算基本的局面参数。不包含概率计算。
    /// - 对于avf录像，必须analyse以后才能正确获取是否扫完。
    pub fn analyse(&mut self) {
        // println!("{:?}, ", self.board);
        assert!(
            self.can_analyse,
            "调用parse_video或new_with_data方法前，不能调用analyse方法"
        );
        let mut b = MinesweeperBoard::new(self.board.clone());
        self.game_board_stream.push(GameBoard::new(self.mine_num));
        for ide in 0..self.events.len() {
            self.events[ide].prior_game_board_id = self.game_board_stream.len() - 1;
            if self.events[ide].mouse != "mv" {
                // println!("{:?}, {:?}", self.events[ide].time, self.events[ide].mouse);
                let u_level = b
                    .step(
                        &self.events[ide].mouse,
                        (
                            (self.events[ide].y / 16) as usize,
                            (self.events[ide].x / 16) as usize,
                        ),
                    )
                    .unwrap();
                self.events[ide].useful_level = u_level;
                if u_level >= 2 {
                    let mut g_b = GameBoard::new(self.mine_num);
                    g_b.set_game_board(&b.game_board);
                    self.game_board_stream.push(g_b);
                }
                // println!("{:?}", b.game_board_state);
                // println!("{:?}", b.solved3BV);
            }
            self.events[ide].next_game_board_id = self.game_board_stream.len() - 1;
            self.events[ide].mouse_state = b.mouse_state.clone();
            self.events[ide].solved3BV = b.solved3BV;
        }
        // println!("888");
        // println!("{:?}", b.solved3BV);
        self.is_completed = b.game_board_state == GameBoardState::Win;
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
    /// 传入要检查的事件，会把结果记在comments字段里。
    /// 可以传入high_risk_guess、jump_judge、needless_guess、mouse_trace、vision_transfer、survive_poss等。顺序不讲究。
    /// #### 检查录像中所有的教科书式的fl局部（python）
    /// ```python
    /// import ms_toollib as ms
    /// import re
    /// v = ms.AvfVideo("z.avf"); # 用文件名实例化
    /// v.parse_video()
    /// v.analyse()
    /// v.analyse_for_features(["super_fl_local"]) # 用哪些分析方法。分析结果会记录到events_comments字段里
    /// for i in range(v.events_len): # 鼠标事件包括鼠标移动、按下、抬起
    ///     c = v.events_comments(i) # 每一个鼠标事件，都有events_comments字段，记录了分析算法的结果
    ///     if c != '':
    ///         print('时间：', v.events_time(i), '事件：', c)
    ///         step_num = int(re.findall(r"(?<=步数).*?(?=\))", c)[0]) # 正则提取要打印几步
    ///         p = i
    ///         for j in range(step_num):
    ///             while v.events_useful_level(p) <= 0:
    ///                 # 用events_useful_level字段辅助过滤不要看的事件
    ///                 # 也可以用events_mouse_state字段来过滤
    ///                 p += 1
    ///             print('鼠标事件类型：', v.events_mouse(p),
    ///                 '第几行：', v.events_y(p)//16,
    ///                 '第几列：', v.events_x(p)//16)
    ///             p += 1
    /// ```
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
                "super_fl_local" => analyse_super_fl_local(self),
                _ => continue,
            };
        }
    }
    pub fn print_event(&self) {
        let mut num = 0;
        for e in &self.events {
            // if num < 800 {
            //     if e.mouse != "mv" {
            //         println!(
            //             "time = {:?}, mouse = {:?}, x = {:?}, y = {:?}, level = {:?}",
            //             e.time, e.mouse, e.x, e.y, e.useful_level
            //         );
            //     }
            // }
            if e.mouse != "mv" {
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
            //     e.prior_game_board
            //         .game_board
            //         .iter()
            //         .for_each(|v| println!("{:?}", v));
            //     e.prior_game_board
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
    pub fn print_raw_data(&self, n: usize) {
        for i in 0..n {
            let v = self.raw_data[i];
            print!("{:?}", v as char);
        }
        println!();
        for i in 0..n {
            let v = self.raw_data[i];
            print!("{:?}, ", v);
        }
    }
    // 再实现一些get、set方法
    /// 获取当前录像时刻的后验的游戏局面（就是说录像里看不到没打开之前的样子）
    pub fn get_game_board(&self) -> Vec<Vec<i32>> {
        self.game_board_stream[self.events[self.current_event_id].next_game_board_id as usize]
            .game_board
            .clone()
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
                return self.game_board_stream
                    [self.events[self.current_event_id].next_game_board_id as usize]
                    .poss
                    .clone();
                // return self.events[id].prior_game_board.get_poss().clone();
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

impl BaseVideo {
    /// 按evf标准，计算出原始二进制数据
    pub fn generate_evf_v0_raw_data(&mut self) {
        self.raw_data = vec![0, 0];
        if self.is_completed {
            self.raw_data[1] |= 0b1000_0000;
        }
        if self.is_offical {
            self.raw_data[1] |= 0b0100_0000;
        }
        if self.is_fair {
            self.raw_data[1] |= 0b0010_0000;
        }
        self.raw_data.push(self.height as u8);
        self.raw_data.push(self.width as u8);
        self.raw_data.push((self.mine_num >> 8).try_into().unwrap());
        self.raw_data
            .push((self.mine_num % 256).try_into().unwrap());
        self.raw_data.push(self.cell_pixel_size);
        self.raw_data.push((self.mode >> 8).try_into().unwrap());
        self.raw_data.push((self.mode % 256).try_into().unwrap());
        self.raw_data
            .push((self.static_params.bbbv >> 8).try_into().unwrap());
        self.raw_data
            .push((self.static_params.bbbv % 256).try_into().unwrap());
        self.raw_data
            .push((self.dynamic_params.r_time_ms >> 16).try_into().unwrap());
        self.raw_data.push(
            ((self.dynamic_params.r_time_ms >> 8) % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data
            .push((self.dynamic_params.r_time_ms % 256).try_into().unwrap());
        self.raw_data
            .append(&mut self.software.clone().as_bytes().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.player_designator.clone().as_bytes().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.race_designator.clone().as_bytes().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.uniqueness_designator.clone().as_bytes().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.start_time.clone().as_bytes().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.end_time.clone().as_bytes().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.country.clone().as_bytes().to_owned());
        self.raw_data.push(0);

        let mut byte = 0;
        let mut ptr = 0;
        for i in 0..self.height {
            for j in 0..self.width {
                byte <<= 1;
                if self.board[i][j] == -1 {
                    byte |= 1;
                }
                ptr += 1;
                if ptr == 8 {
                    self.raw_data.push(byte);
                    ptr = 0;
                    byte = 0;
                }
            }
        }
        byte <<= 7 - ptr;
        self.raw_data.push(byte);

        for event in &self.events {
            match event.mouse.as_str() {
                "mv" => self.raw_data.push(1),
                "lc" => self.raw_data.push(2),
                "lr" => self.raw_data.push(3),
                "rc" => self.raw_data.push(4),
                "rr" => self.raw_data.push(5),
                "mc" => self.raw_data.push(6),
                "mr" => self.raw_data.push(7),
                "pf" => self.raw_data.push(8),
                // 不可能出现，出现再说
                _ => {}
            }
            let t_ms = s_to_ms(event.time);
            self.raw_data.push((t_ms >> 16).try_into().unwrap());
            self.raw_data.push(((t_ms >> 8) % 256).try_into().unwrap());
            self.raw_data.push((t_ms % 256).try_into().unwrap());
            self.raw_data.push((event.x >> 8).try_into().unwrap());
            self.raw_data.push((event.x % 256).try_into().unwrap());
            self.raw_data.push((event.y >> 8).try_into().unwrap());
            self.raw_data.push((event.y % 256).try_into().unwrap());
        }
        if !self.checksum.is_empty() {
            self.raw_data.push(0);
            self.raw_data
                .append(&mut self.checksum.clone().as_bytes().to_owned());
        } else {
            self.raw_data.push(255);
        }
    }
    /// 存evf文件，自动加后缀，xxx.evf重复变成xxx(2).evf
    pub fn save_to_evf_file(&self, file_name: &str) {
        let file_exist =
            std::path::Path::new((file_name.to_string() + &(".evf".to_string())).as_str()).exists();
        if !file_exist {
            fs::write((file_name.to_string() + &(".evf".to_string())).as_str(), &self.raw_data).unwrap();
            return;
        } else {
            let mut id = 2;
            let mut format_name;
            loop {
                format_name = file_name.to_string() + &(format!("({}).evf", id).to_string());
                let new_file_name = format_name.as_str();
                let file_exist = std::path::Path::new(new_file_name).exists();
                if !file_exist {
                    fs::write(new_file_name, &self.raw_data).unwrap();
                    return;
                }
                id += 1;
            }
        }
    }
}
