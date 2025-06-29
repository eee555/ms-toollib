// 录像相关的类，局面在board
use crate::board::GameBoard;
use crate::cal_cell_nums;
use crate::miscellaneous::s_to_ms;
#[cfg(any(feature = "py", feature = "rs"))]
use crate::miscellaneous::time_ms_between;
#[cfg(any(feature = "py", feature = "rs"))]
use crate::utils::cal_bbbv;
use crate::utils::{cal_isl, cal_op};
use crate::videos::analyse_methods::{
    analyse_high_risk_guess, analyse_jump_judge, analyse_mouse_trace, analyse_needless_guess,
    analyse_super_fl_local, analyse_survive_poss, analyse_vision_transfer,
};
use core::panic;
use std::cell::RefCell;
#[cfg(any(feature = "py", feature = "rs"))]
use std::fs;
use std::rc::Rc;
#[cfg(any(feature = "py", feature = "rs"))]
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::safe_board::BoardSize;
#[cfg(any(feature = "py", feature = "rs"))]
use crate::safe_board::SafeBoard;

use crate::{GameBoardState, MinesweeperBoard, MouseState};
use encoding_rs::{GB18030, WINDOWS_1252};
// use tract_onnx::prelude::Op;
use std::cmp::min;

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
    Utf8Error,
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
#[derive(Clone)]
pub struct VideoActionStateRecorder {
    /// 相对时间，从0开始，大于rtime
    pub time: f64,
    /// 操作类型，这几种："mv", "lc", "lr", "rc", "rr", "mc", "mr", "pf"
    pub mouse: String,
    /// 距离左端有几像素。
    pub x: u16,
    /// 距离上端有几像素。
    pub y: u16,
    /// 0代表完全没用；
    /// 1代表能仅推进局面但不改变对局面的后验判断，例如标雷和取消标雷；
    /// 2代表改变对局面的后验判断的操作，例如左键点开一个或一片格子，不包括双击；
    /// 3代表有效、至少打开了一个格子的双击；
    /// 4代表踩雷并失败；
    /// 和ce没有关系，仅用于控制计算
    pub useful_level: u8,
    /// 操作前的局面（先验局面）的计数引用。
    pub prior_game_board: Option<Rc<RefCell<GameBoard>>>,
    /// 操作后的局面（后验的局面）的计数引用。
    pub next_game_board: Option<Rc<RefCell<GameBoard>>>,
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
            prior_game_board: None,
            next_game_board: None,
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
#[derive(Clone)]
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
    pub pluck: Option<f64>,
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
            pluck: None,
        }
    }
}

/// 游戏动态类指标，侧重保存最终结果
/// 游戏阶段就可以展示
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
    pub path: f64,
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
            path: 0.0,
        }
    }
}

/// 录像动态类指标，侧重保存最终结果
/// 游戏阶段不能展示，录像播放时可以展示
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
    // 未完成
    pub op_solved: usize,
    // 未完成
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

/// 需要分析才能计算出的指标，通常计算代价很大。最终结果
/// 游戏阶段不能展示，录像播放时可以展示
pub struct VideoAnalyseParams {
    pub pluck: Option<f64>,
}

impl Default for VideoAnalyseParams {
    fn default() -> Self {
        VideoAnalyseParams { pluck: None }
    }
}

/// 扫雷游戏状态机
/// 功能：整局游戏的全部信息。自动推导局面、计算数据、计时、保存文件等功能。
/// 以下是在python中调用的示例。
/// ``` python
/// import ms_toollib as ms
/// import time
/// pixsize = 16
/// board = [[ 0, 0, 0, 0, 0, 0, 0, 0],
///          [ 0, 1, 2, 3, 3, 3, 3, 2],
///          [ 0, 1,-1,-1,-1,-1,-1,-1],
///          [ 1, 2, 2, 3, 3, 3, 3, 2],
///          [-1, 2, 0, 0, 0, 0, 0, 0],
///          [-1, 2, 0, 0, 0, 0, 0, 0],
///          [ 1, 1, 0, 0, 0, 1, 2, 2],
///          [ 0, 0, 0, 0, 0, 1,-1,-1],]
/// game = ms.BaseVideo(board, pixsize)
/// # 左键按下
/// game.step("lc", (8, 8))
/// # 左键抬起
/// game.step("lr", (8, 8))
/// assert game.game_board_state == 2
/// # Ready => 1
/// # Playing => 2
/// # Win => 3
/// # Loss => 4
/// # PreFlaging => 5
/// # Display => 6
/// assert game.mouse_state == 1
/// # UpUp => 1
/// # UpDown => 2
/// # UpDownNotFlag => 3
/// # DownUp => 4
/// # Chording => 5
/// # ChordingNotFlag => 6
/// # DownUpAfterChording => 7
/// # Undefined => 8
/// print(game.game_board)
/// # 右键按下
/// game.step("rc", (4 * pixsize + 3, 0))
/// # 人类操作的延时
/// time.sleep(0.01)
/// # 右键抬起
/// game.step("rr", (4 * pixsize + 3, 0))
/// game.step("rc", (5 * pixsize + 3, 0))
/// game.step("rr", (5 * pixsize + 3, 0))
/// game.step("lc", (4 * pixsize + 3, pixsize + 1))
/// game.step("lr", (4 * pixsize + 3, pixsize + 1))
/// game.step("lc", (4 * pixsize + 3, pixsize + 1))
/// # 第二个键按下，但分不清具体哪个键
/// game.step("cc", (4 * pixsize + 3, pixsize + 1))
/// game.step("rr", (4 * pixsize + 3, pixsize + 1))
/// game.step("lr", (4 * pixsize + 3, pixsize + 1))
/// # 检查已经获胜
/// assert game.game_board_state == 3
/// print("本局用时：", game.rtime)
/// assert game.left == 2
/// assert game.lce == 2
/// game.use_question = False # 禁用问号是共识
/// game.use_cursor_pos_lim = False
/// game.use_auto_replay = False
/// game.is_fair = False
/// game.is_official = False
/// game.software = "Demo"
/// game.mode = 0
/// game.player_identifier = "赵大锤"
/// game.race_identifier = "G1234"
/// game.uniqueness_identifier = "contact us with QQ 2234208506"
/// game.country = "CN"
/// game.device_uuid = "2ds2ge6rg5165g1r32ererrrtgrefd6we54"
/// game.generate_evf_v3_raw_data()
/// # 补上校验值
/// checksum = b"0" * 32
/// game.checksum = checksum
/// # 保存为test.evf
/// game.save_to_evf_file("test")
/// ```
pub struct BaseVideo<T> {
    /// 软件名，包括："Viennasweeper"、"0.97 beta"、"Arbiter"、"元3.1.9"、"元3.1.11"、"元3.2.0"等
    pub software: String,
    /// 转码用的软件名，包括："元3.1.11"、"元3.2.0"等
    pub translate_software: String,
    /// 宽度（格数），等同于column
    pub width: usize,
    /// 高度（格数），等同于row
    pub height: usize,
    /// 雷数
    pub mine_num: usize,
    /// 是否扫完: 软件证明这局是完成的，即没有踩雷、时间等所有数值没有溢出。其余条件一概不保证。  
    /// 初始是false，踩雷的话，分析完还是false
    pub is_completed: bool,
    /// 是否正式: 软件证明这局是正式的，一定扫完，包括没有用软件筛选3BV、没有看概率、是标准模式、时间等所有数值没有溢出。  
    /// 不一定包括是否满足排名网站对于3BV的额外限制。例如，。
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
    /// false是此录像非转码录像；true为此录像为转码录像
    pub translated: bool,
    /// 游戏模式。0->标准、1->upk；2->cheat；3->Density（来自Viennasweeper、clone软件）、4->win7、5->经典无猜、6->强无猜、7->弱无猜、8->准无猜、9->强可猜、10->弱可猜
    /// 递归模式暂时未能实现
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
    game_board_stream: Vec<Rc<RefCell<GameBoard>>>,
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
    /// 当前current_time对应的video_action_state_recorder中的索引
    pub current_event_id: usize,
    /// 仅转码录像使用，标识的可能的编码方式。
    /// 可能的值，'utf-8', 'utf-16', 'utf-16-be', 'utf-16-le', 'gbk', 'gb2312', 'big5', 'shift-jis', 'cp932', 'latin-1', 'ascii', 'iso-8859-1'等
    pub original_encoding: String,
    /// 录像用户标识
    pub player_identifier: String,
    /// 比赛标识
    pub race_identifier: String,
    /// 唯一性标识
    pub uniqueness_identifier: String,
    /// 游戏起始时间，单位为微秒的时间戳。
    /// 本工具箱对文件中的数据进行了加工，实际上文件中的原始信息如下：
    /// 在阿比特中，‘16.10.2021.22.24.23.9906’，意味2021年10月16日，下午10点24分23秒906。
    /// 其中"9906"这四位数字，表示毫秒整除100、毫秒整除10（一或二位数）、毫秒对10取余。
    /// 所以"9906"就是906毫秒；再例如"083"就是83毫秒。
    /// 在维也纳扫雷中，‘1382834716’，代表以秒为单位的时间戳
    pub start_time: u64,
    /// 游戏终止时间，单位为微秒的时间戳。维也纳扫雷中没有记录，而是计算得到。
    pub end_time: u64,
    /// 国家。大写的两位国家代码，例如中国"CN"、美国"US"，未知"XX"
    pub country: String,
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
    /// 最终的录像需要分析才能计算的指标，计算代价最大
    pub video_analyse_params: VideoAnalyseParams,
    // /// 最终的路径长度
    // pub path: usize,
    // /// 开始扫前，已经标上的雷。如果操作流中包含标这些雷的过程，
    // pub pre_flags: Vec<(usize, usize)>,
    ///校验码
    pub checksum: Vec<u8>,
    pub can_analyse: bool,
    // 游戏前标的雷数
    // new_before_game方法里用到，真正开始的时间
    // net_start_time: f64,
    // 允许设置最终成绩，解析录像文件时用
    allow_set_rtime: bool,
    // 是否有checksum，似乎无用，拟废弃
    // has_checksum: bool,
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
            software: String::new(),
            translate_software: String::new(),
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
            translated: true,
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
            original_encoding: String::new(),
            player_identifier: String::new(),
            race_identifier: String::new(),
            uniqueness_identifier: String::new(),
            start_time: 0,
            end_time: 0,
            country: String::new(),
            device_uuid: vec![],
            raw_data: vec![],
            offset: 0,
            static_params: StaticParams::default(),
            game_dynamic_params: GameDynamicParams::default(),
            video_dynamic_params: VideoDynamicParams::default(),
            video_analyse_params: VideoAnalyseParams::default(),
            checksum: vec![],
            can_analyse: false,
            // net_start_time: 0.0,
            // has_checksum: false,
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
            software: String::new(),
            translate_software: String::new(),
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
            translated: false,
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
            original_encoding: String::new(),
            player_identifier: String::new(),
            race_identifier: String::new(),
            uniqueness_identifier: String::new(),
            start_time: 0,
            end_time: 0,
            country: String::new(),
            device_uuid: vec![],
            raw_data: vec![],
            offset: 0,
            static_params: StaticParams::default(),
            game_dynamic_params: GameDynamicParams::default(),
            video_dynamic_params: VideoDynamicParams::default(),
            video_analyse_params: VideoAnalyseParams::default(),
            checksum: vec![],
            can_analyse: false,
            // net_start_time: 0.0,
            // has_checksum: false,
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
        self.game_board_stream
            .push(Rc::new(RefCell::new(first_game_board)));
        for ide in 0..self.video_action_state_recorder.len() {
            // 控制svi的生命周期
            let svi = &mut self.video_action_state_recorder[ide];
            svi.prior_game_board = Some(Rc::clone(self.game_board_stream.last().unwrap()));
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
                    self.game_board_stream.push(Rc::new(RefCell::new(g_b)));
                    if old_state != GameBoardState::Playing {
                        self.delta_time = svi.time;
                    }
                    // println!("{:?}, {:?}", self.game_board_stream.len(), svi.mouse);
                }
            }
            svi.next_game_board = Some(Rc::clone(self.game_board_stream.last().unwrap()));
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
        let rtime = self.game_dynamic_params.rtime;
        let bbbv = self.static_params.bbbv as f64;
        self.is_completed = b.game_board_state == GameBoardState::Win;
        
        // evf以外的录像没有判断是否公正的方法，只能根据是否扫完
        // avf是唯一在parse解析阶段不能判定是否扫完的录像
        self.is_official = self.is_completed;
        self.is_fair = self.is_completed;

        self.nf = b.rce == 0;
        self.game_dynamic_params.left = b.left;
        self.game_dynamic_params.left_s = b.left as f64 / rtime;
        self.game_dynamic_params.right = b.right;
        self.game_dynamic_params.right_s = b.right as f64 / rtime;
        // println!("---{:?}", b.bbbv_solved);
        self.video_dynamic_params.bbbv_solved = b.bbbv_solved;
        self.video_dynamic_params.lce = b.lce;
        self.video_dynamic_params.rce = b.rce;
        self.video_dynamic_params.dce = b.dce;
        self.video_dynamic_params.ce = b.lce + b.rce + b.dce;
        self.video_dynamic_params.ce_s = (b.lce + b.rce + b.dce) as f64 / rtime;
        self.game_dynamic_params.double = b.double;
        self.game_dynamic_params.cl = b.left + b.right + b.double;
        self.game_dynamic_params.cl_s = self.game_dynamic_params.cl as f64 / rtime;
        self.game_dynamic_params.flag = b.flag;
        self.video_dynamic_params.bbbv_s = bbbv / rtime;
        self.video_dynamic_params.rqp = rtime * rtime / bbbv;
        if self.height == 8 && self.width == 8 && self.mine_num == 10 {
            self.video_dynamic_params.stnb =
                47.22 / (rtime.powf(1.7) / bbbv) * (b.bbbv_solved as f64 / bbbv).powf(0.5);
        } else if self.height == 16 && self.width == 16 && self.mine_num == 40 {
            self.video_dynamic_params.stnb =
                153.73 / (rtime.powf(1.7) / bbbv) * (b.bbbv_solved as f64 / bbbv).powf(0.5);
        } else if self.height == 16 && self.width == 30 && self.mine_num == 99 {
            self.video_dynamic_params.stnb =
                435.001 / (rtime.powf(1.7) / bbbv) * (b.bbbv_solved as f64 / bbbv).powf(0.5);
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

impl BaseVideo<Vec<Vec<i32>>> {
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
    pub fn get_i16(&mut self) -> Result<i16, ErrReadVideoReason> {
        let a = self.get_u8()?;
        let b = self.get_u8()?;
        Ok((a as i16) << 8 | (b as i16))
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
    pub fn get_u64(&mut self) -> Result<u64, ErrReadVideoReason> {
        let a = self.get_u32()?;
        let b = self.get_u32()?;
        Ok((a as u64) << 32 | (b as u64))
    }
    pub fn get_char(&mut self) -> Result<char, ErrReadVideoReason> {
        let a = self.get_u8()?;
        Ok(a as char)
    }
    pub fn get_buffer<U>(&mut self, length: U) -> Result<Vec<u8>, ErrReadVideoReason>
    where
        U: Into<usize>,
    {
        let length = length.into();
        self.offset += length;
        self.raw_data
            .get((self.offset - length)..self.offset)
            .map(|vv| vv.to_vec())
            .ok_or(ErrReadVideoReason::FileIsTooShort)
    }
    pub fn get_c_buffer(&mut self, end: char) -> Result<Vec<u8>, ErrReadVideoReason> {
        let mut s = vec![];
        loop {
            let the_byte = self.get_char()?;
            if the_byte == end {
                break;
            }
            s.push(the_byte as u8);
        }
        Ok(s)
    }
    pub fn get_utf8_string<U>(&mut self, length: U) -> Result<String, ErrReadVideoReason>
    where
        U: Into<usize>,
    {
        let length = length.into();
        String::from_utf8(self.get_buffer(length)?).map_err(|_e| ErrReadVideoReason::Utf8Error)
    }
    /// 读取以end结尾的合法utf-8字符串
    pub fn get_utf8_c_string(&mut self, end: char) -> Result<String, ErrReadVideoReason> {
        String::from_utf8(self.get_c_buffer(end)?).map_err(|_e| ErrReadVideoReason::Utf8Error)
    }
    pub fn get_unknown_encoding_string<U>(
        &mut self,
        length: U,
    ) -> Result<String, ErrReadVideoReason>
    where
        U: Into<usize>,
    {
        let code = self.get_buffer(length)?;
        if let Ok(s) = String::from_utf8(code.clone()) {
            return Ok(s);
        }
        let (cow, _, had_errors) = GB18030.decode(&code);
        if !had_errors {
            return Ok(cow.into_owned());
        };
        let (cow, _, had_errors) = WINDOWS_1252.decode(&code);
        if !had_errors {
            return Ok(cow.into_owned());
        };
        Ok(String::from_utf8_lossy(&code).to_string())
    }
    /// 读取以end结尾的未知编码字符串，假如所有编码都失败，返回utf-8乱码
    pub fn get_unknown_encoding_c_string(
        &mut self,
        end: char,
    ) -> Result<String, ErrReadVideoReason> {
        let code = self.get_c_buffer(end)?;
        if let Ok(s) = String::from_utf8(code.clone()) {
            return Ok(s);
        }
        let (cow, _, had_errors) = GB18030.decode(&code);
        if !had_errors {
            return Ok(cow.into_owned());
        };
        let (cow, _, had_errors) = WINDOWS_1252.decode(&code);
        if !had_errors {
            return Ok(cow.into_owned());
        };
        Ok(String::from_utf8_lossy(&code).to_string())
    }
    // 是否闰年，计算阿比特时间戳
    fn is_leap_year(&self, year: u64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
    // 一个月有几天，计算阿比特时间戳
    fn days_in_month(&self, year: u64, month: u64) -> u32 {
        let days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        if month == 2 && self.is_leap_year(year) {
            29
        } else {
            days[(month - 1) as usize]
        }
    }

    fn days_since_epoch(&self, year: u64, month: u64, day: u64) -> u64 {
        let mut total_days = 0;
        for y in 1970..year {
            total_days += if self.is_leap_year(y) { 366 } else { 365 };
        }
        for m in 1..month {
            total_days += self.days_in_month(year, m) as u64;
        }
        total_days + day as u64 - 1
    }

    /// 解析avf里的开始时间戳，返回时间戳，微秒。“6606”只取后三位“606”，三位数取后两位
    /// "18.10.2022.20:15:35:6606" -> 1666124135606000
    pub fn parse_avf_start_timestamp(
        &mut self,
        start_timestamp: &str,
    ) -> Result<u64, ErrReadVideoReason> {
        let mut timestamp_parts = start_timestamp.split('.');
        let day = timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let month = timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let year = timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        timestamp_parts = timestamp_parts.next().unwrap().split(':');
        let hour = timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let minute = timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let second = timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let sub_second = timestamp_parts.next().unwrap()[1..]
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;

        let days = self.days_since_epoch(year, month, day);
        let total_seconds = days * 24 * 60 * 60 + hour * 60 * 60 + minute * 60 + second;
        let microseconds = total_seconds * 1_000_000 + sub_second * 1_000;

        Ok(microseconds)
    }

    // 解析avf里的结束时间戳，返回时间戳，微秒
    // "18.10.2022.20:15:35:6606", "18.20:16:24:8868" -> 1666124184868000
    pub fn parse_avf_end_timestamp(
        &mut self,
        start_timestamp: &str,
        end_timestamp: &str,
    ) -> Result<u64, ErrReadVideoReason> {
        let mut start_timestamp_parts = start_timestamp.split('.');
        let mut end_timestamp_parts = end_timestamp.split('.');
        let start_day = start_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let end_day = end_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let mut month = start_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let mut year = start_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        if start_day > end_day {
            // 跨月
            month += 1;
            if month >= 13 {
                month = 1;
                year += 1;
            }
        }
        end_timestamp_parts = end_timestamp_parts.next().unwrap().split(':');
        let hour = end_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let minute = end_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let second = end_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let sub_second = end_timestamp_parts.next().unwrap()[1..]
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;

        let days = self.days_since_epoch(year, month, end_day);
        let total_seconds = days * 24 * 60 * 60 + hour * 60 * 60 + minute * 60 + second;
        let microseconds = total_seconds * 1_000_000 + sub_second * 1_000;

        Ok(microseconds)
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
                        .as_micros() as u64;
                }
            }
            // 不可能
            GameBoardState::Display => {}
            GameBoardState::Loss => {
                self.end_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u64;
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
                    .as_micros() as u64;
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
            self.game_board_stream.push(Rc::new(RefCell::new(g_b)));
            path = 0.0;
        }
        // self.current_time = time;
        let prior_game_board;
        let next_game_board;
        if a >= 1 {
            prior_game_board = Some(Rc::clone(self.game_board_stream.last().unwrap()));
            let mut g_b = GameBoard::new(self.mine_num);
            g_b.set_game_board(&self.minesweeper_board.game_board);
            self.game_board_stream.push(Rc::new(RefCell::new(g_b)));
            next_game_board = Some(Rc::clone(self.game_board_stream.last().unwrap()));
        } else {
            next_game_board = Some(Rc::clone(self.game_board_stream.last().unwrap()));
            prior_game_board = Some(Rc::clone(self.game_board_stream.last().unwrap()));
        }
        self.video_action_state_recorder
            .push(VideoActionStateRecorder {
                time,
                mouse: e.to_string(),
                x: pos.1 as u16,
                y: pos.0 as u16,
                next_game_board,
                prior_game_board,
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
                    pluck: None,
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
        let num = 0;
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
            print!("{:b}, ", v);
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
            return self.video_action_state_recorder[self.current_event_id]
                .next_game_board
                .as_ref()
                .unwrap()
                .borrow()
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
                return self.video_action_state_recorder[self.current_event_id]
                    .next_game_board
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .get_poss()
                    .clone();
            }
        }
    }
    // 录像解析时，设置游戏时间，时间成绩。
    // 同时设置秒和毫秒的时间，并且只能写入一次
    pub fn set_rtime<U>(&mut self, time: U) -> Result<u8, ()>
    where
        U: Into<f64>,
    {
        if !self.allow_set_rtime {
            return Err(());
        }
        let time = time.into();
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
        // end_time的计算方法是特殊的，直接返回rtime，而不是用减法
        // 因为此处减法会带来浮点数误差
        // Ok(self.video_action_state_recorder.last().unwrap().time - self.delta_time)
        Ok(self.game_dynamic_params.rtime)
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
    pub fn set_software(&mut self, software: String) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Ready
        {
            return Err(());
        };
        self.software = software;
        Ok(0)
    }

    pub fn set_player_identifier(&mut self, player_identifier: String) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.player_identifier = player_identifier;
        Ok(0)
    }
    pub fn set_race_identifier(&mut self, race_identifier: String) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.race_identifier = race_identifier;
        Ok(0)
    }
    pub fn set_uniqueness_identifier(&mut self, uniqueness_identifier: String) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.uniqueness_identifier = uniqueness_identifier;
        Ok(0)
    }
    /// 拟弃用，会自动记录
    // pub fn set_start_time(&mut self, start_time: Vec<u8>) -> Result<u8, ()> {
    //     if self.game_board_state != GameBoardState::Loss
    //         && self.game_board_state != GameBoardState::Win
    //     {
    //         return Err(());
    //     };
    //     self.start_time = start_time;
    //     Ok(0)
    // }
    /// 拟弃用，会自动记录
    // pub fn set_end_time(&mut self, end_time: Vec<u8>) -> Result<u8, ()> {
    //     if self.game_board_state != GameBoardState::Loss
    //         && self.game_board_state != GameBoardState::Win
    //     {
    //         return Err(());
    //     };
    //     self.end_time = end_time;
    //     Ok(0)
    // }
    pub fn set_country(&mut self, country: String) -> Result<u8, ()> {
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
    /// 在生成二进制数据后，在raw_data里添加checksum
    /// 按照evf0-3的标准添加，即删除末尾的/255，添加/0、32位checksum
    pub fn set_checksum_evf_v3(&mut self, checksum: Vec<u8>) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        if self.checksum.is_empty() {
            *self.raw_data.last_mut().unwrap() = 0;
            self.raw_data
                .append(&mut checksum.clone().to_vec().to_owned());
            self.checksum = checksum;
            // self.has_checksum = true;
            return Ok(0);
        } else {
            let ptr = self.raw_data.len() - 32;
            for i in 0..32 {
                self.raw_data[ptr + i] = checksum[i];
            }
            return Ok(0);
        }
    }
    /// 在生成二进制数据后，在raw_data里添加checksum
    /// 按照evf4的标准添加，即添加u16的长度、若干位checksum
    pub fn set_checksum_evf_v4(&mut self, checksum: Vec<u8>) -> Result<u8, ()> {
        // avf、evfv3、evfv4的典型高级录像体积对比，单位kB
        // 压缩前：64.2，63.9，47.9
        // 压缩后(zip)：25.4，24.6，6.84
        // 压缩后(gzip)：25.2，24.7，6.6
        // 压缩后(xz-6)：10.9，11.1，4.98
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.raw_data
            .truncate(self.raw_data.len() - self.checksum.len() - 2);
        self.raw_data
            .extend_from_slice(&(checksum.len() as u16).to_be_bytes());
        self.raw_data
            .append(&mut checksum.clone().to_vec().to_owned());
        return Ok(0);
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
            #[allow(unreachable_patterns)]
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
            #[allow(unreachable_patterns)]
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
            #[allow(unreachable_patterns)]
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
            #[allow(unreachable_patterns)]
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
            #[allow(unreachable_patterns)]
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
            Ok(c * bbbv_solved as f64 / self.current_time.powf(1.7)
                * (bbbv_solved as f64 / self.static_params.bbbv as f64).powf(0.5))
        } else {
            Ok(
                c * bbbv_solved as f64 / self.game_dynamic_params.rtime.powf(1.7)
                    * (bbbv_solved as f64 / self.static_params.bbbv as f64).powf(0.5),
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
    // 未实现
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
    // 未实现
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
    // 必须用survive_poss方法分析以后才能获取
    pub fn get_pluck(&self) -> Result<f64, ()> {
        if self.game_board_state != GameBoardState::Display {
            return Err(());
        };
        Ok(self.video_action_state_recorder[self.current_event_id]
            .key_dynamic_params
            .pluck
            .unwrap())
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
    pub fn get_checksum(&self) -> Result<Vec<u8>, ()> {
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
        self.raw_data
            .append(&mut self.software.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.player_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.race_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.uniqueness_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.start_time.to_string().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.end_time.to_string().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.country.to_string().into_bytes());
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
        if !self.checksum.is_empty() {
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
        self.raw_data
            .append(&mut self.software.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.player_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.race_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.uniqueness_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.start_time.to_string().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.end_time.to_string().into_bytes());
        self.raw_data.push(0);
        self.raw_data.append(&mut self.country.clone().into_bytes());
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
        if !self.checksum.is_empty() {
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
        self.raw_data
            .append(&mut self.software.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.player_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.race_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.uniqueness_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.start_time.to_string().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.end_time.to_string().into_bytes());
        self.raw_data.push(0);
        self.raw_data.append(&mut self.country.clone().into_bytes());
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
        if !self.checksum.is_empty() {
            self.raw_data.push(0);
            self.raw_data
                .append(&mut self.checksum.clone().to_vec().to_owned());
        } else {
            self.raw_data.push(255);
        }
    }

    /// 按evf v4标准，编码出原始二进制数据
    /// v4开始，判断nf的标准发生了变化！
    pub fn generate_evf_v4_raw_data(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        assert!(self.height <= 255);
        assert!(self.width <= 255);
        assert!(self.height * self.cell_pixel_size as usize <= 32767);
        assert!(self.width * self.cell_pixel_size as usize <= 32767);
        assert!(self.mine_num <= 65535);
        self.raw_data = vec![4, 0, 0];
        if self.is_completed {
            self.raw_data[1] |= 0b1000_0000;
        }
        if self.is_official {
            self.raw_data[1] |= 0b0100_0000;
        }
        if self.is_fair {
            self.raw_data[1] |= 0b0010_0000;
        }
        if self.get_rce().unwrap() == 0 {
            self.raw_data[1] |= 0b0001_0000;
        }
        if self.translated {
            self.raw_data[1] |= 0b0000_1000;
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
        self.raw_data
            .extend_from_slice(&self.game_dynamic_params.rtime_ms.to_be_bytes());
        if self.country.len() < 2 {
            self.raw_data.extend("XX".as_bytes());
        } else {
            let first_char = self.country.chars().nth(0).unwrap();
            let second_char = self.country.chars().nth(1).unwrap();
            if first_char.is_ascii_alphabetic() && second_char.is_ascii_alphabetic() {
                self.raw_data.push(first_char.to_ascii_uppercase() as u8);
                self.raw_data.push(second_char.to_ascii_uppercase() as u8);
            } else {
                self.raw_data.extend("XX".as_bytes());
            }
        }
        self.raw_data
            .extend_from_slice(&self.start_time.to_be_bytes());
        self.raw_data
            .extend_from_slice(&self.end_time.to_be_bytes());
        self.raw_data
            .append(&mut self.software.clone().into_bytes());
        self.raw_data.push(0);
        if self.translated {
            self.raw_data
                .append(&mut self.translate_software.clone().into_bytes());
            self.raw_data.push(0);
            self.raw_data
                .append(&mut self.original_encoding.clone().into_bytes());
            self.raw_data.push(0);
        }
        self.raw_data
            .append(&mut self.player_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.race_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.uniqueness_identifier.clone().into_bytes());
        self.raw_data.push(0);
        let device_uuid_length = self.device_uuid.len() as u16;
        self.raw_data
            .extend_from_slice(&device_uuid_length.to_be_bytes());
        self.raw_data
            .append(&mut self.device_uuid.clone().to_owned());
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
        // 自定义指标的数量
        self.raw_data.push(0);
        self.raw_data.push(0);
        let event_0 = &self.video_action_state_recorder[0];
        match event_0.mouse.as_str() {
            "mv" => self.raw_data.push(1),
            "lc" => self.raw_data.push(2),
            "lr" => self.raw_data.push(3),
            "rc" => self.raw_data.push(4),
            "rr" => self.raw_data.push(5),
            "mc" => self.raw_data.push(6),
            "mr" => self.raw_data.push(7),
            "pf" => self.raw_data.push(8),
            "cc" => self.raw_data.push(9),
            "l" => self.raw_data.push(10),
            "r" => self.raw_data.push(11),
            "m" => self.raw_data.push(12),
            // 不可能出现，出现再说
            _ => {}
        }
        let t_ms = s_to_ms(event_0.time) as u8;
        self.raw_data.push((t_ms).try_into().unwrap());
        self.raw_data.push((event_0.x >> 8).try_into().unwrap());
        self.raw_data.push((event_0.x % 256).try_into().unwrap());
        self.raw_data.push((event_0.y >> 8).try_into().unwrap());
        self.raw_data.push((event_0.y % 256).try_into().unwrap());

        for event_id in 1..self.video_action_state_recorder.len() {
            let event = &self.video_action_state_recorder[event_id];
            // println!("{:?}: '{:?}', ({:?}, {:?})", event.time, event.mouse.as_str(), event.x, event.y);
            let last_event = &self.video_action_state_recorder[event_id - 1];
            let mut delta_t = s_to_ms(event.time) - s_to_ms(last_event.time);
            while delta_t > 255 {
                self.raw_data.push(255);
                let pause_time = min(65535 as u32, delta_t) as u16;
                self.raw_data.extend_from_slice(&pause_time.to_be_bytes());
                delta_t -= pause_time as u32;
            }
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
                "l" => self.raw_data.push(10),
                "r" => self.raw_data.push(11),
                "m" => self.raw_data.push(12),
                // 不可能出现，出现再说
                _ => {
                    continue;
                }
            }
            self.raw_data.push(delta_t as u8);
            let delta_x = event.x as i16 - last_event.x as i16;
            let delta_y = event.y as i16 - last_event.y as i16;
            self.raw_data.extend_from_slice(&delta_x.to_be_bytes());
            self.raw_data.extend_from_slice(&delta_y.to_be_bytes());
        }
        self.raw_data.push(0);
        self.raw_data
            .extend_from_slice(&(self.checksum.len() as u16).to_be_bytes());
        self.raw_data
            .append(&mut self.checksum.clone().to_vec().to_owned());
    }
    // /// 在二进制数据最后添加checksum。通过generate_evf_v0_raw_data或push_checksum添加checksum二选一。
    // /// 若无checksum就用generate_evf_v0_raw_data
    // pub fn push_checksum(&mut self, checksum: &mut Vec<u8>) {
    //     *self.raw_data.last_mut().unwrap() = 0;
    //     self.raw_data.append(checksum);
    // }
    /// 存evf文件，自动加后缀，xxx.evf重复变成xxx(2).evf
    pub fn save_to_evf_file(&self, file_name: &str) -> String {
        let file_exist =
            std::path::Path::new((file_name.to_string() + &(".evf".to_string())).as_str()).exists();
        if !file_exist {
            fs::write(
                (file_name.to_string() + &(".evf".to_string())).as_str(),
                &self.raw_data,
            )
            .unwrap();
            return (file_name.to_string() + &(".evf".to_string()))
                .as_str()
                .to_string();
        } else {
            let mut id = 2;
            let mut format_name;
            loop {
                format_name = file_name.to_string() + &(format!("({}).evf", id).to_string());
                let new_file_name = format_name.as_str();
                let file_exist = std::path::Path::new(new_file_name).exists();
                if !file_exist {
                    fs::write(new_file_name, &self.raw_data).unwrap();
                    return new_file_name.to_string();
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
        // 2024-11-21 00:00:00 发布，有效期将有至少一年
        "元3.2.0" => Ok(("1732118400".to_string(), "1763654400".to_string())),
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
        if self.software == "Arbiter" {
            // 对于arbiter，开源界没有鉴定的能力，只能追溯
        } else if self.software == "Viennasweeper" {
        } else if self.software == "元3.1.9" {
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
        } else if self.software == "元3.1.11" || self.software == "元3.2.0" {
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
