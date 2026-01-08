// 录像相关的类，局面在board
use crate::board::GameBoard;
use crate::cal_cell_nums;
#[cfg(any(feature = "py", feature = "rs"))]
use crate::miscellaneous::time_ms_between;
#[cfg(any(feature = "py", feature = "rs"))]
use crate::utils::cal_bbbv;
use crate::utils::{cal_isl, cal_op};
use crate::videos::analyse_methods::{
    analyse_high_risk_guess, analyse_jump_judge, analyse_mouse_trace, analyse_needless_guess,
    analyse_pluck, analyse_super_fl_local, analyse_vision_transfer,
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

use crate::{GameBoardState, MinesweeperBoard};
// use tract_onnx::prelude::Op;
use crate::algorithms::cal_probability_cells_not_mine;
use crate::mark_board;
use std::cmp::{max, min};

use crate::videos::byte_reader::ByteReader;
use crate::videos::types::{
    Event, GameDynamicParams, KeyDynamicParams, MouseEvent, StaticParams,
    VideoActionStateRecorder, VideoAnalyseParams, VideoDynamicParams,
};
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
#[derive(Clone)]
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
    /// 录像状态。
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
    pub delta_time: f64,
    /// 当前时间，仅录像播放时用。有负数。
    pub current_time: f64,
    /// 当前current_time对应的video_action_state_recorder中的索引
    pub current_event_id: usize,
    /// 仅转码录像使用，标识的可能的编码方式。
    /// 可能的值，'utf-8', 'utf-16', 'utf-16-be', 'utf-16-le', 'gbk', 'gb2312', 'big5', 'shift-jis', 'cp932', 'latin-1', 'ascii', 'iso-8859-1'等
    pub original_encoding: String,
    /// 录像用户标识
    pub player_identifier: String,
    /// 比赛标识。仅支持元扫雷和维也纳。对于不支持的版本，返回空字符串。
    pub race_identifier: String,
    /// 唯一性标识
    pub uniqueness_identifier: String,
    /// 游戏起始时间，单位为微秒的时间戳。
    /// 实际数值为游戏起始时间与 1970-01-01 00:00:00 的差值。对于元扫雷和维也纳，后者(1970-01-01)为UTC；对于阿比特，后者时区为生成录像时的系统时区。
    ///
    /// 本工具箱对文件中的数据进行了加工，实际上文件中的原始信息如下：
    ///
    /// - 在阿比特中，‘16.10.2021.22.24.23.9906’，意味 2021-10-16 22:24:23.906。工具箱将返回`1634423063906000`。
    /// 其中`9906`这四位数字，分别表示毫秒整除100、毫秒整除10（一或二位数）、毫秒对10取余。
    /// 所以`9906`就是906毫秒；再例如`083`就是83毫秒。
    ///
    /// - 在维也纳扫雷中，`1382834716`，代表以秒为单位的时间戳。工具箱将返回`1382834716000000`。
    pub start_time: u64,
    /// 游戏终止时间，单位为微秒的时间戳。参考[`start_time`]。
    ///
    /// 维也纳扫雷中没有记录，而是计算得到。
    pub end_time: u64,
    /// 国家。大写的两位国家代码，例如中国"CN"、美国"US"，未知"XX"
    pub country: String,
    /// 设备信息相关的uuid。例如在元扫雷中，长度为32。
    pub device_uuid: Vec<u8>,
    /// 原始二进制数据
    pub raw_data: Vec<u8>,
    /// 解析二进制文件数据时的指针
    pub offset: usize,
    /// 静态指标
    pub static_params: StaticParams,
    /// 最终的游戏动态指标
    pub game_dynamic_params: GameDynamicParams,
    /// 最终的录像动态指标
    pub video_dynamic_params: VideoDynamicParams,
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
    pub allow_set_rtime: bool,
    // 是否有checksum，似乎无用，拟废弃
    // has_checksum: bool,
    // 播放录像文件时用，按几倍放大来播放，涉及回报的鼠标位置
    pub video_playing_pix_size_k: f64,
    // 最后一次局面内的光标位置，用于计算path
    pub last_in_board_pos: (u16, u16),
    // 在last_in_board_pos位置，path的值，用于计算path
    pub last_in_board_pos_path: f64,
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
        if self.game_board_state != GameBoardState::Display {
            self.game_board_state = GameBoardState::Ready;
        }
        self.last_in_board_pos = (u16::MAX, u16::MAX);
        self.last_in_board_pos_path = 0.0;
    }
    /// 进行局面的推衍，计算基本的局面参数，记录所有中间过程。不包含概率计算。
    /// - 对于avf录像，必须analyse以后才能正确获取是否扫完。
    pub fn analyse(&mut self) {
        // println!("{:?}, ", self.board);
        assert!(self.can_analyse, "调用parse或扫完前，不能调用analyse方法");
        // self.minesweeper_board
        let mut b = MinesweeperBoard::<Vec<Vec<i32>>>::new(self.board.clone());
        let mut first_game_board = GameBoard::new(self.mine_num);
        first_game_board.set_game_board(&vec![vec![10; self.width]; self.height]);
        self.game_board_stream
            .push(Rc::new(RefCell::new(first_game_board)));
        for ide in 0..self.video_action_state_recorder.len() {
            // 控制svi的生命周期
            let svi = &mut self.video_action_state_recorder[ide];
            if let Some(Event::Mouse(mouse_event)) = &svi.event {
                svi.prior_game_board = Some(Rc::clone(self.game_board_stream.last().unwrap()));
                if mouse_event.mouse != "mv" {
                    let old_state = b.game_board_state;
                    // println!(
                    //     ">>>  {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                    //     mouse_event.mouse,
                    //     b.mouse_state,
                    //     b.game_board_state,
                    //     b.left,
                    //     b.right,
                    //     b.double,
                    //     b.bbbv_solved
                    // );
                    let u_level = b
                        .step(
                            &mouse_event.mouse,
                            (
                                (mouse_event.y / self.cell_pixel_size as u16) as usize,
                                (mouse_event.x / self.cell_pixel_size as u16) as usize,
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
                // let svi = &self.video_action_state_recorder[ide];
                // 在下述状态中计算path
                if b.game_board_state == GameBoardState::Playing
                    || b.game_board_state == GameBoardState::Win
                    || b.game_board_state == GameBoardState::Loss
                {
                    if self.last_in_board_pos == (u16::MAX, u16::MAX) {
                        // 第一下操作不可能是在局面外的
                        // 初始化只执行一次
                        self.last_in_board_pos = (mouse_event.y, mouse_event.x);
                        self.last_in_board_pos_path = 0.0;
                    }
                    if mouse_event.y >= self.height as u16 * self.cell_pixel_size as u16
                        && mouse_event.x >= self.width as u16 * self.cell_pixel_size as u16
                    {
                        self.video_action_state_recorder[ide].path = self.last_in_board_pos_path;
                        // 也等于self.video_action_state_recorder[ide - 1].path
                    } else {
                        // let svi = &mut self.video_action_state_recorder[ide];
                        svi.path = self.last_in_board_pos_path
                            + ((mouse_event.y as f64 - self.last_in_board_pos.0 as f64).powf(2.0)
                                + (mouse_event.x as f64 - self.last_in_board_pos.1 as f64)
                                    .powf(2.0))
                            .powf(0.5)
                                * 16.0
                                / (self.cell_pixel_size as f64);
                        self.last_in_board_pos = (mouse_event.y, mouse_event.x);
                        self.last_in_board_pos_path = svi.path;
                    }
                }
            } else if let Some(Event::GameState(_game_state_event)) = &svi.event {
                continue;
            } else {
                continue;
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
    /// v.parse()
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
    pub fn analyse_for_features(&mut self, controller: &Vec<&str>) {
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
                &"high_risk_guess" => analyse_high_risk_guess(self),
                &"jump_judge" => analyse_jump_judge(self),
                &"needless_guess" => analyse_needless_guess(self),
                &"mouse_trace" => analyse_mouse_trace(self),
                &"vision_transfer" => analyse_vision_transfer(self),
                &"pluck" => analyse_pluck(self),
                &"super_fl_local" => analyse_super_fl_local(self),
                _ => panic!("not supported analysis feature!"),
            };
        }
    }
    // 播放阶段计算pluck，必须先手动调用analyse_pluck
    pub fn get_pluck(&self) -> Result<f64, ()> {
        match self.game_board_state {
            GameBoardState::Display => Ok(self.video_action_state_recorder[self.current_event_id]
                .key_dynamic_params
                .pluck),
            _ => Err(()),
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
    // 游戏阶段（结束后）计算pluck，不必手动调用analyse_pluck
    pub fn get_pluck(&mut self) -> Result<f64, ()> {
        match self.game_board_state {
            GameBoardState::Win | GameBoardState::Loss => {
                let pluck = self.video_analyse_params.pluck;
                if pluck.is_nan() {
                    let mut pluck = 0.0;
                    let mut has_begin = false;
                    for vas in self.video_action_state_recorder.iter_mut() {
                        if let Some(Event::Mouse(e)) = &vas.event {
                            if vas.useful_level == 2 {
                                // 有效的左键
                                if !has_begin {
                                    has_begin = true;
                                    continue;
                                }
                                let x = (e.y / self.cell_pixel_size as u16) as usize;
                                let y = (e.x / self.cell_pixel_size as u16) as usize;
                                // 安全的概率
                                let p = 1.0
                                    - vas
                                        .prior_game_board
                                        .as_ref()
                                        .unwrap()
                                        .borrow_mut()
                                        .get_poss()[x][y];
                                if p <= 0.0 {
                                    return Ok(f64::INFINITY);
                                } else if p < 1.0 {
                                    pluck -= p.log10();
                                }
                            } else if vas.useful_level == 3 {
                                // 有效的双键
                                let x = (e.y / self.cell_pixel_size as u16) as usize;
                                let y = (e.x / self.cell_pixel_size as u16) as usize;
                                let mut game_board_clone = vas
                                    .prior_game_board
                                    .as_ref()
                                    .unwrap()
                                    .borrow_mut()
                                    .game_board
                                    .clone();
                                let mut chording_cells = vec![];
                                for m in max(1, x) - 1..min(self.height, x + 2) {
                                    for n in max(1, y) - 1..min(self.width, y + 2) {
                                        if game_board_clone[m][n] == 10 {
                                            chording_cells.push((m, n));
                                        }
                                    }
                                }
                                let _ = mark_board(&mut game_board_clone, true).unwrap();
                                // 安全的概率
                                let p = cal_probability_cells_not_mine(
                                    &game_board_clone,
                                    self.mine_num as f64,
                                    &chording_cells,
                                );
                                if p <= 0.0 {
                                    return Ok(f64::INFINITY);
                                } else if p > 0.0 {
                                    pluck -= p.log10();
                                }
                            } else if vas.useful_level == 4 {
                                use core::f64;

                                return Ok(f64::INFINITY);
                            }
                        }
                    }
                    return Ok(pluck);
                }
                Ok(pluck)
            }
            _ => Err(()),
        }
    }
}

impl ByteReader for BaseVideo<Vec<Vec<i32>>> {
    fn raw_data(&self) -> &[u8] {
        &self.raw_data
    }

    fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
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
    /// 实施鼠标动作
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
                self.is_completed = false;
                // 点一下左键可能直接获胜，但不可能直接失败
                self.gather_params_after_game(
                    time_ms,
                    Some(MouseEvent {
                        mouse: e.to_string(),
                        x: pos.1 as u16,
                        y: pos.0 as u16,
                    }),
                );
            }
            GameBoardState::Win => {
                self.end_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u64;
                self.is_completed = true;
                if old_state == GameBoardState::PreFlaging {
                    // 点一左键下就直接获胜的情况
                    self.start_time = self.end_time.clone();
                    self.game_start_ms = time_ms;
                }
                self.gather_params_after_game(
                    time_ms,
                    Some(MouseEvent {
                        mouse: e.to_string(),
                        x: pos.1 as u16,
                        y: pos.0 as u16,
                    }),
                );
            }
        }
        // 维护path，挺复杂的
        let mut path = 0.0;
        if self.game_board_state == GameBoardState::Playing
            || self.game_board_state == GameBoardState::Win
            || self.game_board_state == GameBoardState::Loss
        {
            // 这个值的初始值是max
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
                event: Some(Event::Mouse(MouseEvent {
                    mouse: e.to_string(),
                    x: pos.1 as u16,
                    y: pos.0 as u16,
                })),
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
                    pluck: f64::NAN,
                },
                path,
            });
        // println!("push: {:?}, {:?}, ({:?}, {:?})", time, e, pos.0, pos.1);
        Ok(0)
    }

    /// 实施游戏状态动作
    /// 游戏状态事件编码：{81: "replay", 82: "win", 83: "fail", 99: "error"}。
    #[cfg(any(feature = "py", feature = "rs"))]
    pub fn step_game_state(&mut self, e: &str) -> Result<u8, ()>
    where
        T: std::ops::Index<usize> + BoardSize + std::fmt::Debug,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        let step_instant = Instant::now();
        let time_ms = time_ms_between(step_instant, self.video_start_instant);
        match e {
            "replay" | "fail" => {
                self.end_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u64;
                self.game_board_state = GameBoardState::Loss;
                self.is_completed = false;
                self.gather_params_after_game(time_ms, None);
                Ok(0)
            }
            _ => Err(()),
        }
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
    fn gather_params_after_game(&mut self, time_ms: u32, last_mouse_event: Option<MouseEvent>)
    where
        T: std::ops::Index<usize> + BoardSize + std::fmt::Debug,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        assert!(
            self.game_board_state == GameBoardState::Loss
                || self.game_board_state == GameBoardState::Win
        );
        if self.minesweeper_board.board_changed {
            // 此处是解决可猜模式中由于局面更改，扫完后，ce、bbbv_solved等计算不对，需要重来一遍
            self.minesweeper_board.reset();
            for action_state in &self.video_action_state_recorder {
                if let Some(Event::Mouse(mouse_event)) = &action_state.event {
                    let r = mouse_event.y as usize / self.cell_pixel_size as usize;
                    let c = mouse_event.x as usize / self.cell_pixel_size as usize;
                    self.minesweeper_board
                        .step(&mouse_event.mouse, (r, c))
                        .unwrap();
                }
            }
            let mouse_e = last_mouse_event.unwrap();
            let r = mouse_e.y as usize / self.cell_pixel_size as usize;
            let c = mouse_e.x as usize / self.cell_pixel_size as usize;
            self.minesweeper_board.step(&mouse_e.mouse, (r, c)).unwrap();
        }
        // 点一下左键可能直接获胜，但不可能直接失败
        let t_ms = time_ms - self.game_start_ms;
        // 这是和录像时间成绩有关
        let t = t_ms as f64 / 1000.0;
        self.static_params.bbbv = cal_bbbv(&self.board);
        self.game_dynamic_params.rtime = t;
        self.game_dynamic_params.rtime_ms = t_ms;
        if self.game_board_state == GameBoardState::Loss {
            self.video_dynamic_params.etime =
                t / self.minesweeper_board.bbbv_solved as f64 * self.static_params.bbbv as f64;
        } else {
            self.video_dynamic_params.etime = t;
        }

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

    pub fn print_event(&self, flag_print_game_board: bool) {
        let num = 0;
        for e in &self.video_action_state_recorder {
            if num < 800 {
                if let Some(Event::Mouse(mouse_event)) = &e.event {
                    if mouse_event.mouse != "mv" {
                        println!(
                            "time = {:?}, mouse = {:?}, x = {:?}, y = {:?}, level = {:?}",
                            e.time,
                            mouse_event.mouse,
                            mouse_event.x / self.get_pix_size().unwrap() as u16,
                            mouse_event.y / self.get_pix_size().unwrap() as u16,
                            e.useful_level
                        );
                    }
                    if mouse_event.mouse != "mv" && flag_print_game_board {
                        println!(
                            "time = {:?}, mouse = {:?}, x = {:?}, y = {:?}",
                            e.time, mouse_event.mouse, mouse_event.x, mouse_event.y
                        );
                        e.next_game_board.iter().for_each(|v| println!("{:?}", v));
                        // e.prior_game_board
                        //     .poss
                        //     .iter()
                        //     .for_each(|v| println!("{:?}", v));
                    }
                }
                // if e.mouse != "mv" {
                //     println!(
                //         "time = {:?}, mouse = {:?}, x = {:?}, y = {:?}, level = {:?}",
                //         e.time, e.mouse, e.x, e.y, e.useful_level
                //     );
                // }
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

/// 软件的合法时间范围。单位为秒。没人用，暂时搁置。
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
        } else if self.software == "0.97 beta" {
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
        } else if self.software == "元3.1.11"
            || self.software == "元3.2.0"
            || self.software == "元3.2.1"
        {
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
