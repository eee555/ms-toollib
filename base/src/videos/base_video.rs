// 录像相关的类，局面在board

use crate::board::GameBoard;
use crate::cal_cell_nums;
use crate::miscellaneous::s_to_ms;
#[cfg(any(feature = "py", feature = "rs"))]
use crate::miscellaneous::time_ms_between;
use crate::utils::{cal_bbbv, cal_isl, cal_op};
use crate::videos::analyse_methods::{
    analyse_high_risk_guess, analyse_jump_judge, analyse_mouse_trace, analyse_needless_guess,
    analyse_super_fl_local, analyse_survive_poss, analyse_vision_transfer,
};
use core::panic;
use std::fs;
#[cfg(any(feature = "py", feature = "rs"))]
use std::time::{Instant, SystemTime, UNIX_EPOCH};
#[cfg(feature = "js")]
use web_sys::console;

use crate::safe_board::BoardSize;
#[cfg(any(feature = "py", feature = "rs"))]
use crate::safe_board::SafeBoard;

use crate::{GameBoardState, MinesweeperBoard, MouseState};

/// 读录像文件失败的原因
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
    VersionBackward,
}

/// 局面活动（点击或移动）
// pub struct Event {
//     pub time: f64,
//     /// 操作类型，这几种："mv", "lc", "lr", "rc", "rr", "mc", "mr", "pf"
//     pub mouse: String,
//     /// 距离左端有几像素。
//     pub x: u16,
//     /// 距离上端有几像素。
//     pub y: u16,
// }

/// 录像里的局面活动（点击或移动）、指标状态(该活动完成后的)、先验后验局面索引
pub struct VideoActionStateRecorder {
    /// 相对时间，从0开始，大于rtime
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
    /// 4代表踩雷并失败
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
    // pub solved3BV: usize,
    /// 指标状态(该活动完成后的、后验的), mv也会记录，浪费了空间
    pub key_dynamic_params: KeyDynamicParams,
    pub path: f64,
}

impl Default for VideoActionStateRecorder {
    fn default() -> Self {
        VideoActionStateRecorder {
            time: 0.0,
            mouse: "".to_string(),
            x: 0,
            y: 0,
            useful_level: 0,
            prior_game_board_id: 0,
            next_game_board_id: 0,
            comments: "".to_string(),
            mouse_state: MouseState::Undefined,
            key_dynamic_params: KeyDynamicParams::default(),
            path: 0.0,
        }
    }
}

pub struct StaticParams {
    pub bbbv: usize,
    pub op: usize,
    pub isl: usize,
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
    /// 鼠标回报率
    pub fps: usize,
}

impl Default for StaticParams {
    fn default() -> Self {
        StaticParams {
            bbbv: 0,
            op: 0,
            isl: 0,
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
            fps: 0,
        }
    }
}

/// 侧重实时记录中间过程、中间状态
/// 每个鼠标事件都会存一个，mv也存，存在浪费
pub struct KeyDynamicParams {
    pub left: usize,
    pub right: usize,
    pub double: usize,
    // ce = lce + rce + dce
    pub lce: usize,
    pub rce: usize,
    pub dce: usize,
    pub flag: usize,
    pub bbbv_solved: usize,
    pub op_solved: usize,
    pub isl_solved: usize,
}

impl Default for KeyDynamicParams {
    fn default() -> Self {
        KeyDynamicParams {
            left: 0,
            right: 0,
            double: 0,
            lce: 0,
            rce: 0,
            dce: 0,
            flag: 0,
            bbbv_solved: 0,
            op_solved: 0,
            isl_solved: 0,
        }
    }
}

/// 游戏动态类指标，侧重保存最终结果
pub struct GameDynamicParams {
    /// 最终时间成绩，不是时间的函数
    pub rtime: f64,
    /// 以毫秒为单位的精确时间
    pub rtime_ms: u32,
    pub left: usize,
    pub right: usize,
    pub double: usize,
    pub cl: usize,
    pub flag: usize,
    pub left_s: f64,
    pub right_s: f64,
    pub double_s: f64,
    pub cl_s: f64,
    pub flag_s: f64,
    /// 四舍五入折算到16像素边长，最终路径长度
    pub path: usize,
}

impl Default for GameDynamicParams {
    fn default() -> Self {
        GameDynamicParams {
            rtime: 0.0,
            rtime_ms: 0,
            left: 0,
            right: 0,
            double: 0,
            cl: 0,
            flag: 0,
            left_s: 0.0,
            right_s: 0.0,
            double_s: 0.0,
            cl_s: 0.0,
            flag_s: 0.0,
            path: 0,
        }
    }
}

/// 录像动态类指标，侧重保存最终结果
pub struct VideoDynamicParams {
    pub etime: f64,
    pub bbbv_s: f64,
    pub bbbv_solved: usize,
    pub stnb: f64,
    pub rqp: f64,
    pub qg: f64,
    pub lce: usize,
    pub rce: usize,
    pub dce: usize,
    pub ce: usize,
    pub ce_s: f64,
    pub ioe: f64,
    pub corr: f64,
    pub thrp: f64,
    pub op_solved: usize,
    pub isl_solved: usize,
}

impl Default for VideoDynamicParams {
    fn default() -> Self {
        VideoDynamicParams {
            etime: 0.0,
            bbbv_s: 0.0,
            bbbv_solved: 0,
            stnb: 0.0,
            rqp: 0.0,
            qg: 0.0,
            lce: 0,
            rce: 0,
            dce: 0,
            ce: 0,
            ce_s: 0.0,
            ioe: 0.0,
            corr: 0.0,
            thrp: 0.0,
            op_solved: 0,
            isl_solved: 0,
        }
    }
}

pub struct BaseVideo<T> {
    /// 软件名
    pub software: Vec<u8>,
    /// 宽度，等同于column
    pub width: usize,
    /// 高度，等同于row
    pub height: usize,
    /// 雷数
    pub mine_num: usize,
    /// 是否扫完: 软件证明这局是完成的，即没有踩雷、时间等所有数值没有溢出。其余条件一概不保证。  
    /// 初始是false，踩雷的话，分析完还是false
    pub is_completed: bool,
    /// 是否正式: 软件证明这局是正式的，一定扫完，包括没有用软件筛选3BV、没有看概率、是标准模式、时间等所有数值没有溢出。  
    /// 不一定包括是否满足排名网站对于3BV的额外限制。例如，没有人能证明avf录像是否正式。
    pub is_official: bool,
    /// 是否公平完成。软件证明这局是公平完成的，一定扫完，比如没有用软件筛选3BV、没有看概率、时间等所有数值没有溢出。  
    /// 公平完成和正式的区别是，只有标准游戏模式可以是正式的，而upk、无猜等模式不正式，但可以是公平完成的。  
    /// 如果是正式的，则一定是公平完成的。
    pub is_fair: bool,
    /// 是否使用了问号，true为使用。
    pub use_question: bool,
    /// 是否限制了光标的位置，true为限制（只要开启，无论如何限制）。
    pub use_cursor_pos_lim: bool,
    /// 是否使用了触雷重开，true为使用（只要开启，无论完成度条件是什么）。
    pub use_auto_replay: bool,
    /// 是不是盲扫，初始是false，不是盲扫的话，分析完还是false
    pub nf: bool,
    /// 游戏模式。0->标准、1->upk；2->cheat；3->Density（来自Viennasweeper、clone软件）、4->win7、5->经典无猜、6->强无猜、7->弱无猜、8->准无猜、9->强可猜、10->弱可猜
    pub mode: u16,
    /// 游戏难度（级别）。3是初级；4是中级；5是高级；6是自定义。
    pub level: u8,
    /// 逻辑上，方格像素的尺寸。传统扫雷一般是16，只有元扫雷会把逻辑尺寸和实际尺寸统一起来。
    pub cell_pixel_size: u8,
    /// 仅在存录像时用到。貌似可以不用。
    pub board: T,
    /// 局面状态机
    pub minesweeper_board: MinesweeperBoard<T>,
    /// 录像状态。貌似用不到。
    pub game_board_state: GameBoardState,
    /// 动作、状态记录器。用于播放录像时的指标，例如solved_bbbv。
    /// 假如不改局面，游戏时用不到。假如改局面，最后需要重新推演一遍。
    pub video_action_state_recorder: Vec<VideoActionStateRecorder>,
    /// 游戏局面流，从一开始没有打开任何格子（包含玩家游戏前的标雷过程），到最后打开了所有
    pub game_board_stream: Vec<GameBoard>,
    /// 录像开始的时间（区别于游戏开始的时间），由计时器控制，仅游戏时用
    #[cfg(any(feature = "py", feature = "rs"))]
    pub video_start_instant: Instant,
    /// 第一次有效的左键抬起的时间，由计时器控制，仅游戏时用, new_before_game方法里用到，真正开始的时间
    #[cfg(any(feature = "py", feature = "rs"))]
    // pub game_start_instant: Instant,
    pub game_start_ms: u32,
    /// 第一次有效的左键抬起的时间，格式不同，只在录像播放模式用到
    delta_time: f64,
    /// 当前时间，仅录像播放时用。有负数。
    pub current_time: f64,
    /// 当前指针指向
    pub current_event_id: usize,
    /// 录像用户标识
    pub player_identifier: Vec<u8>,
    /// 比赛标识
    pub race_identifier: Vec<u8>,
    /// 唯一性标识
    pub uniqueness_identifier: Vec<u8>,
    /// 游戏起始时间和终止时间。不整理格式，读成字符串。
    /// 举例：在阿比特中，‘16.10.2021.22.24.23.9906’，意味2021年10月16日，下午10点24分23秒9906。
    /// 维也纳扫雷中，‘1382834716’，代表以秒为单位的时间戳
    pub start_time: Vec<u8>,
    /// 维也纳扫雷中没有
    pub end_time: Vec<u8>,
    /// 国家。预留字段，暂时不能解析。
    pub country: Vec<u8>,
    /// 设备信息相关的uuid。例如在元扫雷中，长度为32。
    pub device_uuid: Vec<u8>,
    /// 原始二进制数据
    raw_data: Vec<u8>,
    /// 解析二进制文件数据时的指针
    pub offset: usize,
    /// 静态指标
    pub static_params: StaticParams,
    /// 最终的游戏动态指标
    game_dynamic_params: GameDynamicParams,
    /// 最终的录像动态指标
    video_dynamic_params: VideoDynamicParams,
    // /// 最终的路径长度
    // pub path: usize,
    // /// 开始扫前，已经标上的雷。如果操作流中包含标这些雷的过程，
    // pub pre_flags: Vec<(usize, usize)>,
    ///校验码
    pub checksum: [u8; 32],
    pub can_analyse: bool,
    // 游戏前标的雷数
    // new_before_game方法里用到，真正开始的时间
    // net_start_time: f64,
    // 允许设置最终成绩，解析录像文件时用
    allow_set_rtime: bool,
    // 是否有checksum
    has_checksum: bool,
    // 播放录像文件时用，按几倍放大来播放，涉及回报的鼠标位置
    video_playing_pix_size_k: f64,
    // 最后一次局面内的光标位置，用于计算path
    last_in_board_pos: (u16, u16),
    // 在last_in_board_pos位置，path的值，用于计算path
    last_in_board_pos_path: f64,
}

impl Default for BaseVideo<Vec<Vec<i32>>> {
    fn default() -> Self {
        BaseVideo {
            software: vec![],
            width: 0,
            height: 0,
            mine_num: 0,
            is_completed: false,
            is_official: false,
            is_fair: false,
            use_question: false,
            use_cursor_pos_lim: false,
            use_auto_replay: false,
            nf: false,
            mode: 0,
            level: 0,
            cell_pixel_size: 16,
            board: vec![],
            minesweeper_board: MinesweeperBoard::default(),
            game_board_state: GameBoardState::Display,
            video_action_state_recorder: vec![],
            game_board_stream: vec![],
            #[cfg(any(feature = "py", feature = "rs"))]
            video_start_instant: Instant::now(),
            #[cfg(any(feature = "py", feature = "rs"))]
            game_start_ms: 0,
            delta_time: 0.0,
            current_time: 0.0,
            current_event_id: 0,
            player_identifier: vec![],
            race_identifier: vec![],
            uniqueness_identifier: vec![],
            start_time: vec![],
            end_time: vec![],
            country: vec![],
            device_uuid: vec![],
            raw_data: vec![],
            offset: 0,
            static_params: StaticParams::default(),
            game_dynamic_params: GameDynamicParams::default(),
            video_dynamic_params: VideoDynamicParams::default(),
            checksum: [0; 32],
            can_analyse: false,
            // net_start_time: 0.0,
            has_checksum: false,
            allow_set_rtime: false,
            video_playing_pix_size_k: 1.0,
            last_in_board_pos: (u16::MAX, u16::MAX),
            last_in_board_pos_path: 0.0,
        }
    }
}

#[cfg(any(feature = "py", feature = "rs"))]
impl Default for BaseVideo<SafeBoard> {
    fn default() -> Self {
        BaseVideo {
            software: vec![],
            width: 0,
            height: 0,
            mine_num: 0,
            is_completed: false,
            is_official: false,
            is_fair: false,
            use_question: false,
            use_cursor_pos_lim: false,
            use_auto_replay: false,
            nf: false,
            mode: 0,
            level: 0,
            cell_pixel_size: 16,
            board: SafeBoard::new(vec![]),
            minesweeper_board: MinesweeperBoard::<SafeBoard>::default(),
            game_board_state: GameBoardState::Display,
            video_action_state_recorder: vec![],
            game_board_stream: vec![],
            video_start_instant: Instant::now(),
            game_start_ms: 0,
            delta_time: 0.0,
            current_time: 0.0,
            current_event_id: 0,
            player_identifier: vec![],
            race_identifier: vec![],
            uniqueness_identifier: vec![],
            start_time: vec![],
            end_time: vec![],
            country: vec![],
            device_uuid: vec![],
            raw_data: vec![],
            offset: 0,
            static_params: StaticParams::default(),
            game_dynamic_params: GameDynamicParams::default(),
            video_dynamic_params: VideoDynamicParams::default(),
            checksum: [0; 32],
            can_analyse: false,
            // net_start_time: 0.0,
            has_checksum: false,
            allow_set_rtime: false,
            video_playing_pix_size_k: 1.0,
            last_in_board_pos: (u16::MAX, u16::MAX),
            last_in_board_pos_path: 0.0,
        }
    }
}

impl BaseVideo<Vec<Vec<i32>>> {
    /// 重置游戏状态等，不重置标识等很多局都不会变的数据。点脸等，重开。
    pub fn reset(&mut self, row: usize, column: usize, pix_size: u8) {
        self.game_board_stream.clear();
        self.minesweeper_board = MinesweeperBoard::<Vec<Vec<i32>>>::new(vec![vec![0; column]; row]);
        self.width = column;
        self.height = row;
        self.cell_pixel_size = pix_size;
        self.video_action_state_recorder.clear();
        self.game_board_stream.clear();
        self.raw_data.clear();
        self.static_params = StaticParams::default();
        self.game_dynamic_params = GameDynamicParams::default();
        self.video_dynamic_params = VideoDynamicParams::default();
        self.game_board_state = GameBoardState::Ready;
        self.last_in_board_pos = (u16::MAX, u16::MAX);
        self.last_in_board_pos_path = 0.0;
    }
    /// 进行局面的推衍，计算基本的局面参数，记录所有中间过程。不包含概率计算。
    /// - 对于avf录像，必须analyse以后才能正确获取是否扫完。
    pub fn analyse(&mut self) {
        // println!("{:?}, ", self.board);
        assert!(
            self.can_analyse,
            "调用parse_video或扫完前，不能调用analyse方法"
        );
        // self.minesweeper_board
        let mut b = MinesweeperBoard::<Vec<Vec<i32>>>::new(self.board.clone());
        let mut first_game_board = GameBoard::new(self.mine_num);
        first_game_board.set_game_board(&vec![vec![10; self.width]; self.height]);
        self.game_board_stream.push(first_game_board);
        for ide in 0..self.video_action_state_recorder.len() {
            // 控制svi的生命周期
            let svi = &mut self.video_action_state_recorder[ide];
            svi.prior_game_board_id = self.game_board_stream.len() - 1;
            if svi.mouse != "mv" {
                let old_state = b.game_board_state;
                // println!(">>>  {:?}, {:?}", svi.mouse, b.mouse_state);
                let u_level = b
                    .step(
                        &svi.mouse,
                        (
                            (svi.y / self.cell_pixel_size as u16) as usize,
                            (svi.x / self.cell_pixel_size as u16) as usize,
                        ),
                    )
                    .unwrap();
                // println!("     {:?}, {:?}", svi.mouse, b.mouse_state);
                svi.useful_level = u_level;
                if u_level >= 1 {
                    let mut g_b = GameBoard::new(self.mine_num);
                    g_b.set_game_board(&b.game_board);
                    self.game_board_stream.push(g_b);
                    if old_state != GameBoardState::Playing {
                        self.delta_time = svi.time;
                    }
                    // println!("{:?}, {:?}", self.game_board_stream.len(), svi.mouse);
                }
            }
            svi.next_game_board_id = self.game_board_stream.len() - 1;
            svi.mouse_state = b.mouse_state.clone();
            svi.key_dynamic_params.left = b.left;
            svi.key_dynamic_params.right = b.right;
            svi.key_dynamic_params.bbbv_solved = b.bbbv_solved;
            svi.key_dynamic_params.double = b.double;
            svi.key_dynamic_params.lce = b.lce;
            svi.key_dynamic_params.rce = b.rce;
            svi.key_dynamic_params.dce = b.dce;
            svi.key_dynamic_params.flag = b.flag;
            // 这两个很难搞
            svi.key_dynamic_params.op_solved = 0;
            svi.key_dynamic_params.isl_solved = 0;
            let svi = &self.video_action_state_recorder[ide];
            // 在下述状态中计算path
            if b.game_board_state == GameBoardState::Playing
                || b.game_board_state == GameBoardState::Win
                || b.game_board_state == GameBoardState::Loss
            {
                if self.last_in_board_pos == (u16::MAX, u16::MAX) {
                    // 第一下操作不可能是在局面外的
                    // 初始化只执行一次
                    self.last_in_board_pos = (svi.y, svi.x);
                    self.last_in_board_pos_path = 0.0;
                }
                if svi.y >= self.height as u16 * self.cell_pixel_size as u16
                    && svi.x >= self.width as u16 * self.cell_pixel_size as u16
                {
                    self.video_action_state_recorder[ide].path = self.last_in_board_pos_path;
                    // 也等于self.video_action_state_recorder[ide - 1].path
                } else {
                    let svi = &mut self.video_action_state_recorder[ide];
                    svi.path = self.last_in_board_pos_path
                        + ((svi.y as f64 - self.last_in_board_pos.0 as f64).powf(2.0)
                            + (svi.x as f64 - self.last_in_board_pos.1 as f64).powf(2.0))
                        .powf(0.5)
                            * 16.0
                            / (self.cell_pixel_size as f64);
                    self.last_in_board_pos = (svi.y, svi.x);
                    self.last_in_board_pos_path = svi.path;
                }
            }
        }
        self.is_completed = b.game_board_state == GameBoardState::Win;
        self.nf = b.right == 0;
        self.game_dynamic_params.left = b.left;
        self.game_dynamic_params.left_s = b.left as f64 / self.game_dynamic_params.rtime;
        self.game_dynamic_params.right = b.right;
        self.game_dynamic_params.right_s = b.right as f64 / self.game_dynamic_params.rtime;
        // println!("---{:?}", b.bbbv_solved);
        self.video_dynamic_params.bbbv_solved = b.bbbv_solved;
        self.video_dynamic_params.lce = b.lce;
        self.video_dynamic_params.rce = b.rce;
        self.video_dynamic_params.dce = b.dce;
        self.video_dynamic_params.ce = b.lce + b.rce + b.dce;
        self.video_dynamic_params.ce_s =
            (b.lce + b.rce + b.dce) as f64 / self.game_dynamic_params.rtime;
        self.game_dynamic_params.double = b.double;
        self.game_dynamic_params.cl = b.left + b.right + b.double;
        self.game_dynamic_params.cl_s =
            self.game_dynamic_params.cl as f64 / self.game_dynamic_params.rtime;
        self.game_dynamic_params.flag = b.flag;
        self.video_dynamic_params.bbbv_s =
            self.static_params.bbbv as f64 / self.game_dynamic_params.rtime;
        self.video_dynamic_params.rqp = self.game_dynamic_params.rtime
            * self.game_dynamic_params.rtime
            / self.static_params.bbbv as f64;
        if self.height == 8 && self.width == 8 && self.mine_num == 10 {
            self.video_dynamic_params.stnb = 47.22
                / (self.game_dynamic_params.rtime.powf(1.7) / self.static_params.bbbv as f64)
                * (b.bbbv_solved as f64 / self.static_params.bbbv as f64).powf(0.5);
        } else if self.height == 16 && self.width == 16 && self.mine_num == 40 {
            self.video_dynamic_params.stnb = 153.73
                / (self.game_dynamic_params.rtime.powf(1.7) / self.static_params.bbbv as f64)
                * (b.bbbv_solved as f64 / self.static_params.bbbv as f64).powf(0.5);
        } else if self.height == 16 && self.width == 30 && self.mine_num == 99 {
            self.video_dynamic_params.stnb = 435.001
                / (self.game_dynamic_params.rtime.powf(1.7) / self.static_params.bbbv as f64)
                * (b.bbbv_solved as f64 / self.static_params.bbbv as f64).powf(0.5);
        } // 凡自定义的stnb都等于0
        self.video_dynamic_params.ioe = b.bbbv_solved as f64 / self.game_dynamic_params.cl as f64;
        self.video_dynamic_params.corr =
            (b.lce + b.rce + b.dce) as f64 / self.game_dynamic_params.cl as f64;
        self.video_dynamic_params.thrp = b.bbbv_solved as f64 / (b.lce + b.rce + b.dce) as f64;
        // 最后，计算静态指标
        self.cal_static_params();
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
}

#[cfg(any(feature = "py", feature = "rs"))]
impl BaseVideo<SafeBoard> {
    /// 重置游戏状态等，不重置标识等很多局都不会变的数据。点脸等，重开。
    pub fn reset(&mut self, row: usize, column: usize, pix_size: u8) {
        self.game_board_stream.clear();
        self.minesweeper_board =
            MinesweeperBoard::<SafeBoard>::new(SafeBoard::new(vec![vec![0; column]; row]));
        self.width = column;
        self.height = row;
        self.cell_pixel_size = pix_size;
        self.video_action_state_recorder.clear();
        self.game_board_stream.clear();
        self.raw_data.clear();
        self.static_params = StaticParams::default();
        self.game_dynamic_params = GameDynamicParams::default();
        self.video_dynamic_params = VideoDynamicParams::default();
        self.game_board_state = GameBoardState::Ready;
        self.last_in_board_pos = (u16::MAX, u16::MAX);
        self.last_in_board_pos_path = 0.0;
    }
    /// 两种情况调用：游戏开始前、可猜模式（尺寸相等，雷数不检查）
    pub fn set_board(&mut self, board: Vec<Vec<i32>>) -> Result<u8, ()> {
        assert!(!board.is_empty());
        match self.game_board_state {
            GameBoardState::Ready | GameBoardState::PreFlaging => {
                if self.width != board[0].len() || self.height != board.len() {
                    return Err(());
                }
            }
            GameBoardState::Playing => {
                // 不再限制模式，以防过于严格。
                // if self.mode != 6 && self.mode != 7 && self.mode != 8 && self.mode != 9 && self.mode != 10 {
                //     return Err(());
                // }
                if self.width != board[0].len() || self.height != board.len() {
                    return Err(());
                }
            }
            GameBoardState::Display | GameBoardState::Win | GameBoardState::Loss => return Err(()),
        }
        self.mine_num = board.iter().fold(0, |y, row| {
            y + row
                .iter()
                .fold(0, |yy, x| if *x == -1 { yy + 1 } else { yy })
        });
        self.height = board.len();
        self.width = board[0].len();
        if self.height == 8 && self.width == 8 && self.mine_num == 10 {
            self.level = 3;
        } else if self.height == 16 && self.width == 16 && self.mine_num == 40 {
            self.level = 4;
        } else if self.height == 16 && self.width == 30 && self.mine_num == 99 {
            self.level = 5;
        } else {
            self.level = 6;
        }
        self.board = SafeBoard::new(board.clone());

        if self.game_board_state == GameBoardState::Playing {
            self.minesweeper_board.set_board(self.board.clone());
        } else {
            self.minesweeper_board.board = self.board.clone();
        }

        Ok(0)
    }
}

impl<T> BaseVideo<T> {
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

pub trait NewBaseVideo<T> {
    fn new(file_name: T) -> Self;
}

pub trait NewBaseVideo2<T, U> {
    fn new(board: T, cell_pixel_size: U) -> Self;
}

impl NewBaseVideo<Vec<u8>> for BaseVideo<Vec<Vec<i32>>> {
    fn new(raw_data: Vec<u8>) -> Self {
        BaseVideo {
            raw_data,
            allow_set_rtime: true,
            ..BaseVideo::default()
        }
    }
}

/// 通过文件名构造。
#[cfg(any(feature = "py", feature = "rs"))]
impl NewBaseVideo<&str> for BaseVideo<Vec<Vec<i32>>> {
    fn new(file_name: &str) -> Self {
        let raw_data: Vec<u8> = fs::read(file_name).unwrap();
        BaseVideo {
            raw_data,
            allow_set_rtime: true,
            ..BaseVideo::default()
        }
    }
}

/// 游戏前实例化，游戏中不断调用step方法来维护。
#[cfg(any(feature = "py", feature = "rs"))]
impl NewBaseVideo2<Vec<Vec<i32>>, u8> for BaseVideo<SafeBoard> {
    fn new(board: Vec<Vec<i32>>, cell_pixel_size: u8) -> BaseVideo<SafeBoard> {
        let bbbv = cal_bbbv(&board);
        // 这里算出来的雷数不一定准
        let mine_num = board.iter().fold(0, |y, row| {
            y + row
                .iter()
                .fold(0, |yy, x| if *x == -1 { yy + 1 } else { yy })
        });
        let board = SafeBoard::new(board);
        let board_clone = board.clone();
        BaseVideo {
            width: board.get_column(),
            height: board.get_row(),
            mine_num,
            cell_pixel_size,
            board,
            minesweeper_board: MinesweeperBoard::<SafeBoard>::new(board_clone),
            game_board_state: GameBoardState::Ready,
            static_params: StaticParams {
                bbbv,
                ..StaticParams::default()
            },
            ..BaseVideo::<SafeBoard>::default()
        }
    }
}

impl<T> BaseVideo<T> {
    /// 通过文件名构造。
    // #[cfg(any(feature = "py", feature = "rs"))]
    // pub fn new_with_file(file_name: &str) -> BaseVideo<Vec<Vec<i32>>> {
    //     let raw_data: Vec<u8> = fs::read(file_name).unwrap();
    //     // for i in 0..500 {
    //     //     print!("{:?}", raw_data[i] as char);
    //     // }
    //     BaseVideo {
    //         raw_data,
    //         allow_set_rtime: true,
    //         ..BaseVideo::default()
    //     }
    // }
    // /// 游戏前实例化，游戏中不断调用step方法来维护。
    // #[cfg(any(feature = "py", feature = "rs"))]
    // pub fn new_before_game(board: Vec<Vec<i32>>, cell_pixel_size: u8) -> BaseVideo<SafeBoard> {
    //     let bbbv = cal_bbbv(&board);
    //     // 这里算出来的雷数不一定准
    //     let mine_num = board.iter().fold(0, |y, row| {
    //         y + row
    //             .iter()
    //             .fold(0, |yy, x| if *x == -1 { yy + 1 } else { yy })
    //     });
    //     let board = SafeBoard::new(board);
    //     let board_clone = board.clone();
    //     BaseVideo {
    //         width: board.get_column(),
    //         height: board.get_row(),
    //         mine_num,
    //         cell_pixel_size,
    //         board,
    //         minesweeper_board: MinesweeperBoard::<SafeBoard>::new(board_clone),
    //         game_board_state: GameBoardState::Ready,
    //         static_params: StaticParams {
    //             bbbv,
    //             ..StaticParams::default()
    //         },
    //         ..BaseVideo::<SafeBoard>::default()
    //     }
    // }
    // #[cfg(feature = "js")]
    // pub fn new(raw_data: Vec<u8>) -> BaseVideo<Vec<Vec<i32>>> {
    //     // video_data = video_data.into_vec();
    //     BaseVideo {
    //         raw_data,
    //         allow_set_rtime: true,
    //         ..BaseVideo::default()
    //     }
    // }
    /// 步进
    /// - pos的单位是像素，(距离上方，距离左侧)
    /// - 如果操作发生在界外，要求转换成pos=(row*pixsize, column*pixsize)
    #[cfg(any(feature = "py", feature = "rs"))]
    pub fn step(&mut self, e: &str, pos: (usize, usize)) -> Result<u8, ()>
    where
        T: std::ops::Index<usize> + BoardSize + std::fmt::Debug,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        // 第一时间获取时间戳
        let step_instant = Instant::now();
        let mut time_ms = time_ms_between(step_instant, self.video_start_instant);
        let mut time = time_ms as f64 / 1000.0;
        // 获胜、失败、播放模式下，直接返回
        let old_state = self.minesweeper_board.game_board_state;
        if old_state == GameBoardState::Loss
            || old_state == GameBoardState::Win
            || old_state == GameBoardState::Display
        {
            return Ok(0);
        }
        // 运行局面状态机
        let x = pos.0 / self.cell_pixel_size as usize;
        let y = pos.1 / self.cell_pixel_size as usize;
        let a = self.minesweeper_board.step(e, (x, y))?;
        // 获取新的状态
        self.game_board_state = self.minesweeper_board.game_board_state;
        match self.game_board_state {
            GameBoardState::Ready => {
                self.game_board_stream.clear();
                self.video_action_state_recorder.clear();
                return Ok(0);
            }
            GameBoardState::PreFlaging => {
                if old_state != GameBoardState::PreFlaging {
                    self.video_start_instant = step_instant;
                    // time_ms = time_ms_between(step_instant, self.video_start_instant);
                    time_ms = 0;
                    time = 0.0;
                }
            }
            GameBoardState::Playing => {
                if old_state != GameBoardState::Playing {
                    if old_state == GameBoardState::Ready {
                        self.video_start_instant = step_instant;
                        time_ms = time_ms_between(step_instant, self.video_start_instant);
                        time = time_ms as f64 / 1000.0;
                    }
                    self.game_start_ms = time_ms;
                    // 高精度的时间戳，单位为微秒
                    // https://doc.rust-lang.org/std/time/struct.Instant.html
                    self.start_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_micros()
                        .to_string()
                        .into_bytes();
                }
            }
            // 不可能
            GameBoardState::Display => {}
            GameBoardState::Loss => {
                self.end_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros()
                    .to_string()
                    .into_bytes();
                // 点一下左键可能直接获胜，但不可能直接失败
                let t_ms = time_ms - self.game_start_ms;
                self.is_completed = false;
                // 这是和录像时间成绩有关
                let t = t_ms as f64 / 1000.0;
                self.static_params.bbbv = cal_bbbv(&self.board);
                self.game_dynamic_params.rtime = t;
                self.game_dynamic_params.rtime_ms = t_ms;
                self.video_dynamic_params.etime =
                    t / self.minesweeper_board.bbbv_solved as f64 * self.static_params.bbbv as f64;
                self.gather_params_after_game(t);

                if self.minesweeper_board.board_changed {
                    // 此处是解决可猜模式中由于局面更改，扫完后，ce、bbbv_solved等计算不对，需要重来一遍
                    self.minesweeper_board.reset();
                    for action_state in &self.video_action_state_recorder {
                        let x = action_state.y as usize / self.cell_pixel_size as usize;
                        let y = action_state.x as usize / self.cell_pixel_size as usize;
                        self.minesweeper_board.step(&action_state.mouse, (x, y))?;
                    }
                    let x = pos.0 / self.cell_pixel_size as usize;
                    let y = pos.1 / self.cell_pixel_size as usize;
                    self.minesweeper_board.step(e, (x, y))?;
                }
            }
            GameBoardState::Win => {
                self.end_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros()
                    .to_string()
                    .into_bytes();
                if old_state == GameBoardState::PreFlaging {
                    // 点一左键下就直接获胜的情况
                    self.start_time = self.end_time.clone();
                    self.game_start_ms = time_ms;
                    // println!("334")
                }
                let t_ms = time_ms - self.game_start_ms;
                self.is_completed = true;
                let t = t_ms as f64 / 1000.0;
                self.static_params.bbbv = cal_bbbv(&self.board);
                self.game_dynamic_params.rtime = t;
                self.game_dynamic_params.rtime_ms = t_ms;
                self.video_dynamic_params.etime = t;
                self.gather_params_after_game(t);

                if self.minesweeper_board.board_changed {
                    // 此处是解决可猜模式中由于局面更改，扫完后，ce、bbbv_solved等计算不对，需要重来一遍
                    self.minesweeper_board.reset();
                    for action_state in &self.video_action_state_recorder {
                        let x = action_state.y as usize / self.cell_pixel_size as usize;
                        let y = action_state.x as usize / self.cell_pixel_size as usize;
                        self.minesweeper_board.step(&action_state.mouse, (x, y))?;
                    }
                    let x = pos.0 / self.cell_pixel_size as usize;
                    let y = pos.1 / self.cell_pixel_size as usize;
                    self.minesweeper_board.step(e, (x, y))?;
                }
            }
        }
        // 维护path，挺复杂的
        let mut path = 0.0;
        if self.game_board_state == GameBoardState::Playing
            || self.game_board_state == GameBoardState::Win
            || self.game_board_state == GameBoardState::Loss
        {
            if self.last_in_board_pos == (u16::MAX, u16::MAX) {
                self.last_in_board_pos = (pos.0 as u16, pos.1 as u16);
                self.last_in_board_pos_path = 0.0;
            }
            if pos.0 >= self.height * self.cell_pixel_size as usize
                && pos.1 >= self.width * self.cell_pixel_size as usize
            {
                path = self.last_in_board_pos_path;
            } else {
                path = self.last_in_board_pos_path
                    + ((pos.0 as f64 - self.last_in_board_pos.0 as f64).powf(2.0)
                        + (pos.1 as f64 - self.last_in_board_pos.1 as f64).powf(2.0))
                    .powf(0.5)
                        * 16.0
                        / (self.cell_pixel_size as f64);
                self.last_in_board_pos = (pos.0 as u16, pos.1 as u16);
                self.last_in_board_pos_path = path;
            }
        }
        if self.game_board_stream.is_empty()
            && (self.game_board_state == GameBoardState::PreFlaging
                || self.game_board_state == GameBoardState::Playing
                || self.game_board_state == GameBoardState::Win
                || self.game_board_state == GameBoardState::Loss)
        {
            // 维护第一个先验局面（和path无关）
            let mut g_b = GameBoard::new(self.mine_num);
            g_b.set_game_board(&vec![vec![10; self.width]; self.height]);
            self.game_board_stream.push(g_b);
            path = 0.0;
        }
        // self.current_time = time;
        let prior_game_board_id;
        let next_game_board_id;
        if a >= 1 {
            let mut g_b = GameBoard::new(self.mine_num);
            g_b.set_game_board(&self.minesweeper_board.game_board);
            self.game_board_stream.push(g_b);
            next_game_board_id = self.game_board_stream.len() - 1;
            prior_game_board_id = self.game_board_stream.len() - 2;
        } else {
            next_game_board_id = self.game_board_stream.len() - 1;
            prior_game_board_id = self.game_board_stream.len() - 1;
        }
        self.video_action_state_recorder
            .push(VideoActionStateRecorder {
                time,
                mouse: e.to_string(),
                x: pos.1 as u16,
                y: pos.0 as u16,
                next_game_board_id,
                prior_game_board_id,
                useful_level: a,
                comments: "".to_string(),
                mouse_state: self.minesweeper_board.mouse_state,
                key_dynamic_params: KeyDynamicParams {
                    left: self.minesweeper_board.left,
                    right: self.minesweeper_board.right,
                    double: self.minesweeper_board.double,
                    lce: self.minesweeper_board.lce,
                    rce: self.minesweeper_board.rce,
                    dce: self.minesweeper_board.dce,
                    flag: self.minesweeper_board.flag,
                    bbbv_solved: self.minesweeper_board.bbbv_solved,
                    op_solved: 0,
                    isl_solved: 0,
                },
                path,
            });
        // println!("push: {:?}, {:?}, ({:?}, {:?})", time, e, pos.0, pos.1);
        Ok(0)
    }

    /// 获胜后标上所有的雷，没获胜则返回
    #[cfg(any(feature = "py", feature = "rs"))]
    pub fn win_then_flag_all_mine(&mut self) {
        if self.minesweeper_board.game_board_state != GameBoardState::Win {
            return;
        }
        self.minesweeper_board.game_board.iter_mut().for_each(|x| {
            x.iter_mut().for_each(|xx| {
                if *xx == 10 {
                    *xx = 11
                }
            })
        });
    }
    /// 失败后展示所有的雷，没失败则返回
    /// 这件事状态机不会自动做，因为有些模式失败后不标出所有的雷，如强无猜
    #[cfg(any(feature = "py", feature = "rs"))]
    pub fn loss_then_open_all_mine(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        if self.minesweeper_board.game_board_state != GameBoardState::Loss {
            return;
        }
        for i in 0..self.height {
            for j in 0..self.width {
                if self.minesweeper_board.board[i][j] == -1
                    && self.minesweeper_board.game_board[i][j] == 10
                {
                    self.minesweeper_board.game_board[i][j] = 16;
                }
            }
        }
    }
    /// 游戏结束后，计算一批指标
    #[cfg(any(feature = "py", feature = "rs"))]
    fn gather_params_after_game(&mut self, t: f64)
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        self.game_dynamic_params.left = self.minesweeper_board.left;
        self.game_dynamic_params.right = self.minesweeper_board.right;
        self.game_dynamic_params.double = self.minesweeper_board.double;
        self.game_dynamic_params.flag = self.minesweeper_board.flag;
        self.game_dynamic_params.cl = self.minesweeper_board.left
            + self.game_dynamic_params.right
            + self.game_dynamic_params.double;
        self.game_dynamic_params.left_s = self.minesweeper_board.left as f64 / t;
        self.game_dynamic_params.right_s = self.minesweeper_board.right as f64 / t;
        self.game_dynamic_params.double_s = self.minesweeper_board.double as f64 / t;
        self.game_dynamic_params.flag_s = self.minesweeper_board.flag as f64 / t;
        self.game_dynamic_params.cl_s = self.game_dynamic_params.cl as f64 / t;
        self.cal_static_params();
    }
    // 计算除fps以外的静态指标
    fn cal_static_params(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        let cell_nums = cal_cell_nums(&self.board);
        self.static_params.cell0 = cell_nums[0];
        self.static_params.cell1 = cell_nums[1];
        self.static_params.cell2 = cell_nums[2];
        self.static_params.cell3 = cell_nums[3];
        self.static_params.cell4 = cell_nums[4];
        self.static_params.cell5 = cell_nums[5];
        self.static_params.cell6 = cell_nums[6];
        self.static_params.cell7 = cell_nums[7];
        self.static_params.cell8 = cell_nums[8];
        self.static_params.op = cal_op(&self.board);
        self.static_params.isl = cal_isl(&self.board);
    }

    pub fn print_event(&self) {
        let mut num = 0;
        for e in &self.video_action_state_recorder {
            if num < 800 {
                if e.mouse != "mv" {
                    println!(
                        "time = {:?}, mouse = {:?}, x = {:?}, y = {:?}, level = {:?}",
                        e.time, e.mouse, e.x, e.y, e.useful_level
                    );
                }
            }
            // println!(
            //     "time = {:?}, mouse = {:?}, x = {:?}, y = {:?}, level = {:?}",
            //     e.time, e.mouse, e.x, e.y, e.useful_level
            // );
            // if e.mouse != "mv" {
            //     println!(
            //         "my_board.step_flow(vec![({:?}, ({:?}, {:?}))]).unwrap();",
            //         // "video.step({:?}, ({:?}, {:?})).unwrap();",
            //         e.mouse,
            //         e.y / self.cell_pixel_size as u16,
            //         e.x / self.cell_pixel_size as u16
            //     );
            // }
            // num += 1;
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
        for i in &self.video_action_state_recorder {
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
}
impl<T> BaseVideo<T> {
    // 再实现一些get、set方法
    pub fn set_pix_size(&mut self, pix_size: u8) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Ready
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Loss
        {
            return Err(());
        }
        self.cell_pixel_size = pix_size;
        Ok(0)
    }
    /// 获取当前录像时刻的后验的游戏局面
    pub fn get_game_board(&self) -> Vec<Vec<i32>> {
        if self.game_board_state == GameBoardState::Display {
            return self.game_board_stream[self.video_action_state_recorder[self.current_event_id]
                .next_game_board_id as usize]
                .game_board
                .clone();
        } else {
            return self.minesweeper_board.game_board.clone();
        }
    }
    /// 获取当前录像时刻的局面概率
    pub fn get_game_board_poss(&mut self) -> Vec<Vec<f64>> {
        let mut id = self.current_event_id;
        loop {
            if self.video_action_state_recorder[id].useful_level < 2 {
                id -= 1;
                if id <= 0 {
                    let p = self.mine_num as f64 / (self.height * self.width) as f64;
                    return vec![vec![p; self.height]; self.width];
                }
            } else {
                // println!("{:?}, {:?}",self.current_event_id, self.video_action_state_recorder.len());
                return self.game_board_stream[self.video_action_state_recorder
                    [self.current_event_id]
                    .next_game_board_id as usize]
                    .get_poss()
                    .to_vec();
                // return self.events[id].prior_game_board.get_poss().clone();
            }
        }
    }
    // 录像解析时，设置游戏时间，时间成绩。
    // 什么意思？
    pub fn set_rtime(&mut self, time: f64) -> Result<u8, ()> {
        if !self.allow_set_rtime {
            return Err(());
        }
        self.game_dynamic_params.rtime = time;
        self.game_dynamic_params.rtime_ms = s_to_ms(time);
        self.allow_set_rtime = false;
        Ok(0)
    }
    /// 用于(游戏时)计数器上显示的时间,和arbiter一致
    #[cfg(any(feature = "py", feature = "rs"))]
    pub fn get_time(&self) -> f64 {
        match self.game_board_state {
            GameBoardState::Playing => {
                let now = Instant::now();
                // return now.duration_since(self.game_start_instant).as_millis() as f64 / 1000.0;
                let time_ms = now.duration_since(self.video_start_instant).as_millis() as u32;
                return (time_ms - self.game_start_ms) as f64 / 1000.0;
            }
            GameBoardState::PreFlaging => {
                let now = Instant::now();
                return now.duration_since(self.video_start_instant).as_millis() as f64 / 1000.0;
            }
            GameBoardState::Loss | GameBoardState::Win => self.game_dynamic_params.rtime,
            GameBoardState::Ready => 0.0,
            GameBoardState::Display => self.current_time,
        }
    }
    pub fn get_rtime(&self) -> Result<f64, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Display
        {
            return Err(());
        }
        Ok(self.game_dynamic_params.rtime)
    }
    pub fn get_rtime_ms(&self) -> Result<u32, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Display
        {
            return Err(());
        }
        Ok(self.game_dynamic_params.rtime_ms)
    }
    /// 录像播放器时间的开始值
    /// 理论上：video_start_time = -self.delta_time
    pub fn get_video_start_time(&self) -> Result<f64, ()> {
        if self.game_board_state != GameBoardState::Display {
            return Err(());
        }
        Ok(-self.delta_time)
    }
    /// 录像播放器时间的结束值
    /// 理论上：video_end_time = rtime
    pub fn get_video_end_time(&self) -> Result<f64, ()> {
        if self.game_board_state != GameBoardState::Display {
            return Err(());
        }
        Ok(self.video_action_state_recorder.last().unwrap().time - self.delta_time)
        // Ok(self.game_dynamic_params.rtime)
    }
    /// 录像播放时，按时间设置current_time；超出两端范围取两端。
    /// 游戏时不要调用。
    pub fn set_current_time(&mut self, mut time: f64) {
        self.current_time = time;
        if self.current_time < self.get_video_start_time().unwrap() {
            self.current_time = self.get_video_start_time().unwrap()
        }
        if self.current_time > self.get_video_end_time().unwrap() {
            self.current_time = self.get_video_end_time().unwrap()
        }
        time += self.delta_time;
        if time > self.video_action_state_recorder[self.current_event_id].time {
            loop {
                if self.current_event_id >= self.video_action_state_recorder.len() - 1 {
                    // 最后一帧
                    break;
                }
                self.current_event_id += 1;
                if self.video_action_state_recorder[self.current_event_id].time <= time {
                    continue;
                } else {
                    self.current_event_id -= 1;
                    break;
                }
            }
        } else {
            loop {
                if self.current_event_id == 0 {
                    break;
                }
                self.current_event_id -= 1;
                if self.video_action_state_recorder[self.current_event_id].time > time {
                    continue;
                } else {
                    break;
                }
            }
        }
        // self.current_time =
        //     self.video_action_state_recorder[self.current_event_id].time - self.delta_time;
    }
    /// 设置current_event_id
    pub fn set_current_event_id(&mut self, id: usize) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Display {
            return Err(());
        };
        self.current_event_id = id;
        self.current_time = self.video_action_state_recorder[id].time - self.delta_time;
        Ok(0)
    }
    pub fn set_use_question(&mut self, use_question: bool) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.use_question = use_question;
        Ok(0)
    }
    pub fn set_use_cursor_pos_lim(&mut self, use_cursor_pos_lim: bool) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.use_cursor_pos_lim = use_cursor_pos_lim;
        Ok(0)
    }
    pub fn set_use_auto_replay(&mut self, use_auto_replay: bool) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.use_auto_replay = use_auto_replay;
        Ok(0)
    }
    pub fn set_is_official(&mut self, is_official: bool) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.is_official = is_official;
        Ok(0)
    }
    pub fn set_is_fair(&mut self, is_fair: bool) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.is_fair = is_fair;
        Ok(0)
    }
    /// 可猜模式必须在ready时设置模式，其它模式扫完再设置也可以
    pub fn set_mode(&mut self, mode: u16) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Ready
        {
            return Err(());
        };
        self.mode = mode;
        Ok(0)
    }
    pub fn set_software(&mut self, software: Vec<u8>) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Ready
        {
            return Err(());
        };
        self.software = software;
        Ok(0)
    }

    pub fn set_player_identifier(&mut self, player_identifier: Vec<u8>) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.player_identifier = player_identifier;
        Ok(0)
    }
    pub fn set_race_identifier(&mut self, race_identifier: Vec<u8>) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.race_identifier = race_identifier;
        Ok(0)
    }
    pub fn set_uniqueness_identifier(&mut self, uniqueness_identifier: Vec<u8>) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.uniqueness_identifier = uniqueness_identifier;
        Ok(0)
    }
    /// 拟弃用，会自动记录
    pub fn set_start_time(&mut self, start_time: Vec<u8>) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.start_time = start_time;
        Ok(0)
    }
    /// 拟弃用，会自动记录
    pub fn set_end_time(&mut self, end_time: Vec<u8>) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.end_time = end_time;
        Ok(0)
    }
    pub fn set_country(&mut self, country: Vec<u8>) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.country = country;
        Ok(0)
    }
    pub fn set_device_uuid(&mut self, device_uuid: Vec<u8>) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Loss
        {
            return Err(());
        }
        self.device_uuid = device_uuid;
        Ok(0)
    }
    /// 在生成二进制数据前得出checksum，则用这个
    pub fn set_checksum(&mut self, checksum: [u8; 32]) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        if !self.has_checksum {
            *self.raw_data.last_mut().unwrap() = 0;
            self.raw_data
                .append(&mut checksum.clone().to_vec().to_owned());
            self.checksum = checksum;
            self.has_checksum = true;
            return Ok(0);
        } else {
            let ptr = self.raw_data.len() - 32;
            for i in 0..32 {
                self.raw_data[ptr + i] = checksum[i];
            }
            return Ok(0);
        }
    }
    pub fn get_raw_data(&self) -> Result<Vec<u8>, ()> {
        if self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Display
        {
            return Err(());
        }
        Ok(self.raw_data.clone())
    }
    pub fn get_left(&self) -> usize {
        match self.game_board_state {
            GameBoardState::Display => {
                self.video_action_state_recorder[self.current_event_id]
                    .key_dynamic_params
                    .left
            }
            _ => self.minesweeper_board.left,
        }
    }
    pub fn get_right(&self) -> usize {
        match self.game_board_state {
            GameBoardState::Display => {
                self.video_action_state_recorder[self.current_event_id]
                    .key_dynamic_params
                    .right
            }
            _ => self.minesweeper_board.right,
        }
    }
    pub fn get_double(&self) -> usize {
        match self.game_board_state {
            GameBoardState::Display => {
                self.video_action_state_recorder[self.current_event_id]
                    .key_dynamic_params
                    .double
            }
            _ => self.minesweeper_board.double,
        }
    }
    pub fn get_cl(&self) -> usize {
        self.get_left() + self.get_right() + self.get_double()
    }
    pub fn get_flag(&self) -> usize {
        match self.game_board_state {
            GameBoardState::Display => {
                self.video_action_state_recorder[self.current_event_id]
                    .key_dynamic_params
                    .flag
            }
            _ => self.minesweeper_board.flag,
        }
    }
    pub fn get_left_s(&self) -> f64 {
        match self.game_board_state {
            GameBoardState::Display => {
                if self.current_time < 0.00099 {
                    return 0.0;
                }
                self.get_left() as f64 / self.current_time
            }
            GameBoardState::Loss | GameBoardState::Win => self.game_dynamic_params.left_s,
            GameBoardState::PreFlaging | GameBoardState::Ready => 0.0,
            #[cfg(any(feature = "py", feature = "rs"))]
            GameBoardState::Playing => {
                let now = Instant::now();
                let t_ms = now.duration_since(self.video_start_instant).as_millis() as u32;
                self.get_left() as f64 * 1000.0 / (t_ms - self.game_start_ms) as f64
            }
            #[cfg(any(feature = "js"))]
            GameBoardState::Playing => 0.0,
        }
    }
    pub fn get_right_s(&self) -> f64 {
        match self.game_board_state {
            GameBoardState::Display => {
                if self.current_time < 0.00099 {
                    return 0.0;
                }
                self.get_right() as f64 / self.current_time
            }
            GameBoardState::Loss | GameBoardState::Win => self.game_dynamic_params.right_s,
            GameBoardState::PreFlaging | GameBoardState::Ready => 0.0,
            #[cfg(any(feature = "py", feature = "rs"))]
            GameBoardState::Playing => {
                let now = Instant::now();
                let t_ms = now.duration_since(self.video_start_instant).as_millis() as u32;
                self.get_right() as f64 * 1000.0 / (t_ms - self.game_start_ms) as f64
            }
            #[cfg(any(feature = "js"))]
            GameBoardState::Playing => 0.0,
        }
    }
    pub fn get_double_s(&self) -> f64 {
        match self.game_board_state {
            GameBoardState::Display => {
                if self.current_time < 0.00099 {
                    return 0.0;
                }
                self.get_double() as f64 / self.current_time
            }
            GameBoardState::Loss | GameBoardState::Win => self.game_dynamic_params.double_s,
            GameBoardState::PreFlaging | GameBoardState::Ready => 0.0,
            #[cfg(any(feature = "py", feature = "rs"))]
            GameBoardState::Playing => {
                let now = Instant::now();
                let t_ms = now.duration_since(self.video_start_instant).as_millis() as u32;
                self.get_double() as f64 * 1000.0 / (t_ms - self.game_start_ms) as f64
            }
            #[cfg(any(feature = "js"))]
            GameBoardState::Playing => 0.0,
        }
    }
    pub fn get_cl_s(&self) -> f64 {
        match self.game_board_state {
            GameBoardState::Display => {
                if self.current_time < 0.00099 {
                    return 0.0;
                }
                self.get_cl() as f64 / self.current_time
            }
            GameBoardState::Loss | GameBoardState::Win => self.game_dynamic_params.cl_s,
            GameBoardState::PreFlaging | GameBoardState::Ready => 0.0,
            #[cfg(any(feature = "py", feature = "rs"))]
            GameBoardState::Playing => {
                let now = Instant::now();
                let t_ms = now.duration_since(self.video_start_instant).as_millis() as u32;
                self.get_cl() as f64 * 1000.0 / (t_ms - self.game_start_ms) as f64
            }
            #[cfg(any(feature = "js"))]
            GameBoardState::Playing => 0.0,
        }
    }
    pub fn get_flag_s(&self) -> f64 {
        match self.game_board_state {
            GameBoardState::Display => {
                if self.current_time < 0.00099 {
                    return 0.0;
                }
                self.get_flag() as f64 / self.current_time
            }
            GameBoardState::Loss | GameBoardState::Win => self.game_dynamic_params.flag_s,
            GameBoardState::PreFlaging | GameBoardState::Ready => 0.0,
            #[cfg(any(feature = "py", feature = "rs"))]
            GameBoardState::Playing => {
                let now = Instant::now();
                let t_ms = now.duration_since(self.video_start_instant).as_millis() as u32;
                self.get_flag() as f64 * 1000.0 / (t_ms - self.game_start_ms) as f64
            }
            #[cfg(any(feature = "js"))]
            GameBoardState::Playing => 0.0,
        }
    }
    pub fn get_path(&self) -> f64 {
        if self.video_action_state_recorder.is_empty() {
            return 0.0;
        }
        if self.game_board_state == GameBoardState::Display {
            self.video_action_state_recorder[self.current_event_id].path
        } else {
            self.video_action_state_recorder.last().unwrap().path
        }
    }
    pub fn get_etime(&self) -> Result<f64, ()> {
        let bbbv_solved = self.get_bbbv_solved()?;
        if bbbv_solved == 0 {
            return Ok(0.0);
        }
        if self.game_board_state == GameBoardState::Display {
            Ok(self.current_time / bbbv_solved as f64 * self.static_params.bbbv as f64)
        } else {
            let t = self.game_dynamic_params.rtime;
            Ok(t / bbbv_solved as f64 * self.static_params.bbbv as f64)
        }
    }
    pub fn get_bbbv_s(&self) -> Result<f64, ()> {
        let bbbv_solved = self.get_bbbv_solved()?;
        if self.game_board_state == GameBoardState::Display {
            if self.current_time < 0.00099 {
                return Ok(0.0);
            }
            return Ok(bbbv_solved as f64 / self.current_time);
        }
        Ok(bbbv_solved as f64 / self.game_dynamic_params.rtime)
    }
    pub fn get_bbbv_solved(&self) -> Result<usize, ()> {
        match self.game_board_state {
            GameBoardState::Display => Ok(self.video_action_state_recorder[self.current_event_id]
                .key_dynamic_params
                .bbbv_solved),
            // GameBoardState::Win | GameBoardState::Loss => Ok(self
            //     .video_dynamic_params
            //     .bbbv_solved),
            GameBoardState::Win | GameBoardState::Loss => Ok(self
                .video_action_state_recorder
                .last()
                .unwrap()
                .key_dynamic_params
                .bbbv_solved),
            _ => Err(()),
        }
    }
    pub fn get_stnb(&self) -> Result<f64, ()> {
        let bbbv_solved = self.get_bbbv_solved()?;
        // println!("self.current_time:{:?}", self.current_time);
        // println!("self.game_board_state:{:?}", self.game_board_state);
        if self.game_board_state == GameBoardState::Display && self.current_time < 0.00099 {
            return Ok(0.0);
        }
        let c;
        match (self.height, self.width, self.mine_num) {
            (8, 8, 10) => c = 47.299,
            (16, 16, 40) => c = 153.73,
            (16, 30, 99) => c = 435.001,
            _ => return Ok(0.0),
        }

        if self.game_board_state == GameBoardState::Display {
            // let t = self.current_time - self.delta_time;
            // println!("t:{:?}", t);
            Ok(
                c * self.static_params.bbbv as f64 / self.current_time.powf(1.7)
                    * (bbbv_solved as f64 / self.static_params.bbbv as f64).powf(0.5),
            )
        } else {
            Ok(
                c * self.static_params.bbbv as f64 / self.game_dynamic_params.rtime.powf(1.7)
                    * ((bbbv_solved as f64 / self.static_params.bbbv as f64).powf(0.5)),
            )
        }
    }
    pub fn get_rqp(&self) -> Result<f64, ()> {
        let bbbv_solved = self.get_bbbv_solved()?;
        if bbbv_solved == 0 {
            return Ok(0.0);
        }
        Ok(self.current_time.powf(2.0) / bbbv_solved as f64)
    }
    pub fn get_qg(&self) -> Result<f64, ()> {
        let bbbv_solved = self.get_bbbv_solved()?;
        if bbbv_solved == 0 {
            return Ok(0.0);
        }
        Ok(self.current_time.powf(1.7) / bbbv_solved as f64)
    }
    pub fn get_lce(&self) -> Result<usize, ()> {
        match self.game_board_state {
            GameBoardState::Display => Ok(self.video_action_state_recorder[self.current_event_id]
                .key_dynamic_params
                .lce),
            GameBoardState::Win | GameBoardState::Loss => Ok(self
                .video_action_state_recorder
                .last()
                .unwrap()
                .key_dynamic_params
                .lce),
            _ => Err(()),
        }
    }
    pub fn get_rce(&self) -> Result<usize, ()> {
        match self.game_board_state {
            GameBoardState::Display => Ok(self.video_action_state_recorder[self.current_event_id]
                .key_dynamic_params
                .rce),
            GameBoardState::Win | GameBoardState::Loss => Ok(self
                .video_action_state_recorder
                .last()
                .unwrap()
                .key_dynamic_params
                .rce),
            _ => Err(()),
        }
    }
    pub fn get_dce(&self) -> Result<usize, ()> {
        match self.game_board_state {
            GameBoardState::Display => Ok(self.video_action_state_recorder[self.current_event_id]
                .key_dynamic_params
                .dce),
            GameBoardState::Win | GameBoardState::Loss => Ok(self
                .video_action_state_recorder
                .last()
                .unwrap()
                .key_dynamic_params
                .dce),
            _ => Err(()),
        }
    }
    pub fn get_ce(&self) -> Result<usize, ()> {
        match self.game_board_state {
            GameBoardState::Display => {
                let p = &self.video_action_state_recorder[self.current_event_id].key_dynamic_params;
                Ok(p.lce + p.rce + p.dce)
            }
            GameBoardState::Win | GameBoardState::Loss => {
                let p = &self
                    .video_action_state_recorder
                    .last()
                    .unwrap()
                    .key_dynamic_params;
                Ok(p.lce + p.rce + p.dce)
            }
            _ => Err(()),
        }
    }
    pub fn get_ce_s(&self) -> Result<f64, ()> {
        let ce = self.get_ce()?;
        if self.current_time < 0.00099 {
            return Ok(0.0);
        }
        Ok(ce as f64 / self.current_time)
    }
    pub fn get_corr(&self) -> Result<f64, ()> {
        let ce = self.get_ce()?;
        let cl = self.get_cl();
        if cl == 0 {
            return Ok(0.0);
        }
        Ok(ce as f64 / cl as f64)
    }
    pub fn get_thrp(&self) -> Result<f64, ()> {
        let ce = self.get_ce()?;
        let bbbv_solved = self.get_bbbv_solved().unwrap();
        if ce == 0 {
            return Ok(0.0);
        }
        Ok(bbbv_solved as f64 / ce as f64)
    }
    pub fn get_ioe(&self) -> Result<f64, ()> {
        let bbbv_solved = self.get_bbbv_solved()?;
        let cl = self.get_cl();
        if cl == 0 {
            return Ok(0.0);
        }
        Ok(bbbv_solved as f64 / cl as f64)
    }
    pub fn get_op_solved(&self) -> Result<usize, ()> {
        if self.game_board_state != GameBoardState::Display
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Loss
        {
            return Err(());
        };
        Ok(self.video_action_state_recorder[self.current_event_id]
            .key_dynamic_params
            .op_solved)
    }
    pub fn get_isl_solved(&self) -> Result<usize, ()> {
        if self.game_board_state != GameBoardState::Display
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Loss
        {
            return Err(());
        };
        Ok(self.video_action_state_recorder[self.current_event_id]
            .key_dynamic_params
            .isl_solved)
    }
    /// 跨语言调用时，不能传递枚举体用这个
    pub fn get_mouse_state(&self) -> usize {
        let m_s;
        if self.game_board_state == GameBoardState::Display {
            m_s = self.video_action_state_recorder[self.current_event_id].mouse_state;
        } else {
            m_s = self.minesweeper_board.mouse_state;
        }
        match m_s {
            MouseState::UpUp => 1,
            MouseState::UpDown => 2,
            MouseState::UpDownNotFlag => 3,
            MouseState::DownUp => 4,
            MouseState::Chording => 5,
            MouseState::ChordingNotFlag => 6,
            MouseState::DownUpAfterChording => 7,
            MouseState::Undefined => 8,
        }
    }
    pub fn get_checksum(&self) -> Result<[u8; 32], ()> {
        if self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Display
        {
            return Err(());
        }
        Ok(self.checksum.clone())
    }
    // 录像播放时，返回鼠标的坐标
    pub fn get_x_y(&self) -> Result<(u16, u16), ()> {
        if self.game_board_state != GameBoardState::Display {
            return Err(());
        };
        let mut k = 0;
        loop {
            if self.video_action_state_recorder[self.current_event_id - k].x
                < self.cell_pixel_size as u16 * self.width as u16
            {
                return Ok((
                    (self.video_action_state_recorder[self.current_event_id - k].x as f64
                        * self.video_playing_pix_size_k) as u16,
                    (self.video_action_state_recorder[self.current_event_id - k].y as f64
                        * self.video_playing_pix_size_k) as u16,
                ));
            }
            k += 1;
        }
    }
    // 返回录像文件里记录的方格尺寸。flop_new播放器里会用到。这是因为元扫雷和flop播放器的播放机制不同。
    pub fn get_pix_size(&self) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Display {
            return Err(());
        };
        Ok(self.cell_pixel_size)
    }
    // 录像播放时，设置按何种像素播放，涉及鼠标位置回报
    pub fn set_video_playing_pix_size(&mut self, pix_size: u8) {
        if self.game_board_state != GameBoardState::Display {
            panic!("");
        };
        self.video_playing_pix_size_k = pix_size as f64 / self.cell_pixel_size as f64;
    }
}

// 和文件操作相关的一些方法
#[cfg(any(feature = "py", feature = "rs"))]
impl<T> BaseVideo<T> {
    /// 按evf v0.0-0.1标准，编码出原始二进制数据
    pub fn generate_evf_v0_raw_data(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        self.raw_data = vec![0, 0];
        if self.is_completed {
            self.raw_data[1] |= 0b1000_0000;
        }
        if self.is_official {
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
        // println!("fff: {:?}", self.game_dynamic_params.rtime_ms);
        self.raw_data.push(
            (self.game_dynamic_params.rtime_ms >> 16)
                .try_into()
                .unwrap(),
        );
        self.raw_data.push(
            ((self.game_dynamic_params.rtime_ms >> 8) % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data.push(
            (self.game_dynamic_params.rtime_ms % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data.append(&mut self.software.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.player_identifier.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.race_identifier.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.uniqueness_identifier.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.start_time.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data.append(&mut self.end_time.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data.append(&mut self.country.clone().to_owned());
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
        if ptr > 0 {
            byte <<= 8 - ptr;
            self.raw_data.push(byte);
        }

        for event in &self.video_action_state_recorder {
            // println!("{:?}: '{:?}', ({:?}, {:?})", event.time, event.mouse.as_str(), event.x, event.y);
            match event.mouse.as_str() {
                "mv" => self.raw_data.push(1),
                "lc" => self.raw_data.push(2),
                "lr" => self.raw_data.push(3),
                "rc" => self.raw_data.push(4),
                "rr" => self.raw_data.push(5),
                "mc" => self.raw_data.push(6),
                "mr" => self.raw_data.push(7),
                "pf" => self.raw_data.push(8),
                "cc" => self.raw_data.push(9),
                // 不可能出现，出现再说
                _ => self.raw_data.push(99),
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
        if self.has_checksum {
            self.raw_data.push(0);
            self.raw_data
                .append(&mut self.checksum.clone().to_vec().to_owned());
        } else {
            self.raw_data.push(255);
        }
    }
    /// 按evf v0.2标准，编码出原始二进制数据
    pub fn generate_evf_v2_raw_data(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        self.raw_data = vec![0, 0];
        if self.is_completed {
            self.raw_data[1] |= 0b1000_0000;
        }
        if self.is_official {
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
        // println!("fff: {:?}", self.game_dynamic_params.rtime_ms);
        self.raw_data.push(
            (self.game_dynamic_params.rtime_ms >> 16)
                .try_into()
                .unwrap(),
        );
        self.raw_data.push(
            ((self.game_dynamic_params.rtime_ms >> 8) % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data.push(
            (self.game_dynamic_params.rtime_ms % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data.append(&mut self.software.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.player_identifier.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.race_identifier.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.uniqueness_identifier.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.start_time.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data.append(&mut self.end_time.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data.append(&mut self.country.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.device_uuid.clone().to_owned());
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
        if ptr > 0 {
            byte <<= 8 - ptr;
            self.raw_data.push(byte);
        }

        for event in &self.video_action_state_recorder {
            // println!("{:?}: '{:?}', ({:?}, {:?})", event.time, event.mouse.as_str(), event.x, event.y);
            match event.mouse.as_str() {
                "mv" => self.raw_data.push(1),
                "lc" => self.raw_data.push(2),
                "lr" => self.raw_data.push(3),
                "rc" => self.raw_data.push(4),
                "rr" => self.raw_data.push(5),
                "mc" => self.raw_data.push(6),
                "mr" => self.raw_data.push(7),
                "pf" => self.raw_data.push(8),
                "cc" => self.raw_data.push(9),
                // 不可能出现，出现再说
                _ => self.raw_data.push(99),
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
        if self.has_checksum {
            self.raw_data.push(0);
            self.raw_data
                .append(&mut self.checksum.clone().to_vec().to_owned());
        } else {
            self.raw_data.push(255);
        }
    }
    /// 按evf v0.3标准，编码出原始二进制数据
    pub fn generate_evf_v3_raw_data(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        self.raw_data = vec![3, 0, 0];
        if self.is_completed {
            self.raw_data[1] |= 0b1000_0000;
        }
        if self.is_official {
            self.raw_data[1] |= 0b0100_0000;
        }
        if self.is_fair {
            self.raw_data[1] |= 0b0010_0000;
        }
        if self.get_right() == 0 {
            self.raw_data[1] |= 0b0001_0000;
        }
        if self.use_question {
            self.raw_data[2] |= 0b1000_0000;
        }
        if self.use_cursor_pos_lim {
            self.raw_data[2] |= 0b0100_0000;
        }
        if self.use_auto_replay {
            self.raw_data[2] |= 0b0010_0000;
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
        // println!("fff: {:?}", self.game_dynamic_params.rtime_ms);
        self.raw_data.push(
            (self.game_dynamic_params.rtime_ms >> 16)
                .try_into()
                .unwrap(),
        );
        self.raw_data.push(
            ((self.game_dynamic_params.rtime_ms >> 8) % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data.push(
            (self.game_dynamic_params.rtime_ms % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data.append(&mut self.software.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.player_identifier.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.race_identifier.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.uniqueness_identifier.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.start_time.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data.append(&mut self.end_time.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data.append(&mut self.country.clone().to_owned());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.device_uuid.clone().to_owned());
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
        if ptr > 0 {
            byte <<= 8 - ptr;
            self.raw_data.push(byte);
        }

        for event in &self.video_action_state_recorder {
            // println!("{:?}: '{:?}', ({:?}, {:?})", event.time, event.mouse.as_str(), event.x, event.y);
            match event.mouse.as_str() {
                "mv" => self.raw_data.push(1),
                "lc" => self.raw_data.push(2),
                "lr" => self.raw_data.push(3),
                "rc" => self.raw_data.push(4),
                "rr" => self.raw_data.push(5),
                "mc" => self.raw_data.push(6),
                "mr" => self.raw_data.push(7),
                "pf" => self.raw_data.push(8),
                "cc" => self.raw_data.push(9),
                // 不可能出现，出现再说
                _ => self.raw_data.push(99),
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
        if self.has_checksum {
            self.raw_data.push(0);
            self.raw_data
                .append(&mut self.checksum.clone().to_vec().to_owned());
        } else {
            self.raw_data.push(255);
        }
    }
    // /// 在二进制数据最后添加checksum。通过generate_evf_v0_raw_data或push_checksum添加checksum二选一。
    // /// 若无checksum就用generate_evf_v0_raw_data
    // pub fn push_checksum(&mut self, checksum: &mut Vec<u8>) {
    //     *self.raw_data.last_mut().unwrap() = 0;
    //     self.raw_data.append(checksum);
    // }
    /// 存evf文件，自动加后缀，xxx.evf重复变成xxx(2).evf
    pub fn save_to_evf_file(&self, file_name: &str) {
        let file_exist =
            std::path::Path::new((file_name.to_string() + &(".evf".to_string())).as_str()).exists();
        if !file_exist {
            fs::write(
                (file_name.to_string() + &(".evf".to_string())).as_str(),
                &self.raw_data,
            )
            .unwrap();
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

/// 软件的合法时间范围。单位为秒。
pub fn valid_time_period(software: &str) -> Result<(String, String), String> {
    match software {
        // 永久
        "Arbiter" => Ok(("0".to_string(), "4102415999".to_string())),
        // 未确定
        "0.97 beta" => Ok(("0".to_string(), "0".to_string())),
        // 2023-04-24 00:00:00 前
        "Viennasweeper" => Ok(("0".to_string(), "1682265600".to_string())),
        // 2024-07-26 00:00:00 发布，有效期将有至少一年
        "元3.1.9" => Ok(("1721836800".to_string(), "1753459200".to_string())),
        // 2024-09-10 00:00:00 发布，有效期将有至少一年
        "元3.1.11" => Ok(("1725811200".to_string(), "1757433600".to_string())),
        _ => Err(String::from("Unknown software: ") + software),
    }
}

// 录像审核相关的方法
impl<T> BaseVideo<T> {
    /// 录像是否合法。排名网站的自动审核录像策略。检查是否扫完、是否有标识、是否用合法的软件。不检查校验码。
    /// 必须在分析完后调用
    /// - 返回：0合法，1不合法，3不确定
    /// - 返回3的原因有：（1）步数很少，例如使用触摸屏
    /// - （2）使用了该软件的不合法的模式，例如元扫雷中的强可猜
    pub fn is_valid(&self) -> u8 {
        let software = match String::from_utf8(self.software.clone()) {
            Ok(software) => software,
            Err(_) => {
                // 软件名称不是合法的utf-8
                return 1;
            }
        };
        if &software == "Arbiter" {
            // 对于arbiter，开源界没有鉴定的能力，只能追溯
        } else if &software == "Viennasweeper" {
        } else if &software == "元3.1.9" {
            if self.checksum.iter().all(|&e| e == self.checksum[0]) {
                // 大概率是使用了测试用的校验和
                return 1;
            }
            if !self.is_fair {
                // 被软件判定使用了辅助手段
                return 1;
            }
            if self.mode != 0 && self.mode != 5 {
                // 只允许标准、经典无猜
                return 3;
            }
        } else if &software == "元3.1.11" {
            if self.checksum.iter().all(|&e| e == self.checksum[0]) {
                // 大概率是使用了测试用的校验和
                return 1;
            }
            if !self.is_fair {
                // 被软件判定使用了辅助手段
                return 1;
            }
            if self.mode != 0 && self.mode != 5 && self.mode != 6 && self.mode != 10 {
                // 只允许标准、经典无猜、强无猜、弱可猜
                return 3;
            }
        } else {
            // 仅限如上三种软件。不可能执行到此。
            panic!("");
        }

        if self.video_action_state_recorder.len() as f64 / (self.static_params.bbbv as f64) < 10.0 {
            // 缺少鼠标移动细节，报不确定
            return 3;
        }

        if self.is_completed {
            // 最后，检查是否完成
            return 0;
        } else {
            return 1;
        }
    }
}
