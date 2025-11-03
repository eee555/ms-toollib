use crate::{MouseState, GameBoard};
use std::cell::RefCell;
use std::rc::Rc;

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

#[derive(Clone)]
pub enum Event {
    Mouse(MouseEvent),
    GameState(GameStateEvent),
    Board(BoardEvent),
    Index(IndexEvent),
}

/// evf标准中的鼠标事件
#[derive(Clone)]
pub struct MouseEvent {
    /// 操作类型，这几种："mv", "lc", "lr", "rc", "rr", "mc", "mr", "pf", "cc", "l", "r", "m"
    pub mouse: String,
    /// 距离左端有几像素。
    pub x: u16,
    /// 距离上端有几像素。
    pub y: u16,
}

/// evf标准中的游戏状态事件
#[derive(Clone)]
pub struct GameStateEvent {
    /// 操作类型，这几种：{81: "replay", 82: "win", 83: "fail", 99: "error"}
    pub game_state: String,
}

/// evf标准中的局面事件
#[derive(Clone)]
pub struct BoardEvent {
    /// 操作类型，这几种：{100: "cell_0", 101: "cell_1", 102: "cell_2", 103: "cell_3", 104: "cell_4",
    /// 105: "cell_5", 106: "cell_6", 107: "cell_7", 108: "cell_8", 110: "up", 111: "flag",
    /// 114: "cross mine", 115: "blast", 116: "mine", 118: "pressed", 120: "questionmark",
    /// 121: "pressed questionmark"}
    pub board: String,
    /// 从上往下，从0开始数，第几行
    pub row_id: u8,
    /// 从左往右，从0开始数，第几列
    pub column_id: u8,
}

#[derive(Clone)]
pub enum IndexValue {
    Number(f64),
    String(String),
}

/// evf标准中的指标事件
#[derive(Clone)]
pub struct IndexEvent {
    pub key: String,
    pub value: IndexValue,
}

/// 录像里的局面活动（点击或移动）、指标状态(该活动完成后的)、先验后验局面索引
#[derive(Clone)]
pub struct VideoActionStateRecorder {
    /// 相对时间，从0开始，大于rtime
    pub time: f64,
    pub event: Option<Event>,
    /// 操作类型，这几种："mv", "lc", "lr", "rc", "rr", "mc", "mr", "pf"
    // pub mouse: String,
    // /// 距离左端有几像素。
    // pub x: u16,
    // /// 距离上端有几像素。
    // pub y: u16,
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
            event: None,
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

#[derive(Clone)]
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
    pub pluck: f64,
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
            pluck: f64::NAN,
        }
    }
}

/// 游戏动态类指标，侧重保存最终结果
/// 游戏阶段就可以展示
#[derive(Clone)]
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
#[derive(Clone)]
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
#[derive(Clone)]
pub struct VideoAnalyseParams {
    pub pluck: f64,
}

impl Default for VideoAnalyseParams {
    fn default() -> Self {
        VideoAnalyseParams { pluck: f64::NAN }
    }
}
