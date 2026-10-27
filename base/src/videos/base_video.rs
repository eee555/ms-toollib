// 录像相关的类，局面在board
use crate::board::GameBoard;
use crate::cal_cell_nums;
use crate::miscellaneous::time_ms_between;
use crate::utils::cal_bbbv;
use crate::utils::{cal_isl, cal_op};
use crate::videos::analyse_methods::{
    analyse_high_risk_guess, analyse_jump_judge, analyse_mouse_trace, analyse_needless_guess,
    analyse_pluck, analyse_super_fl_local, analyse_vision_transfer,
};
use crate::{cal_hzini, cal_zini};
use core::panic;
use std::cell::RefCell;
#[cfg(any(feature = "py", feature = "rs"))]
use std::fs;
use std::rc::Rc;
use web_time::{Instant, SystemTime, UNIX_EPOCH};

use crate::safe_board::{BoardSize, EmptyBoard};
#[cfg(any(feature = "py", feature = "rs"))]
use crate::safe_board::SafeBoard;

use crate::{GameBoardState, MinesweeperBoard};
// use tract_onnx::prelude::Op;
use crate::algorithms::cal_probability_cells_not_mine;
use crate::mark_board;
use std::cmp::{max, min};

use crate::videos::byte_reader::ByteReader;
use crate::videos::types::{
    Event, GameDynamicParams, KeyDynamicParams, MouseEvent, StaticParams, VideoActionStateRecorder,
    VideoAnalyseParams, VideoDynamicParams,
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
/// game.unique_identifier = "contact us with QQ 2234208506"
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
    pub video_start_instant: Instant,
    /// 第一次有效的左键抬起的时间，由计时器控制，仅游戏时用, new_before_game方法里用到，真正开始的时间
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
    pub unique_identifier: String,
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
            video_start_instant: Instant::now(),
            game_start_ms: 0,
            delta_time: 0.0,
            current_time: 0.0,
            current_event_id: 0,
            original_encoding: String::new(),
            player_identifier: String::new(),
            race_identifier: String::new(),
            unique_identifier: String::new(),
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
            unique_identifier: String::new(),
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
                    //     ">>>  {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                    //     svi.time,
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
                36.0 / (rtime.powf(1.7) / bbbv) * (b.bbbv_solved as f64 / bbbv).powf(0.5);
        } else if self.height == 16 && self.width == 16 && self.mine_num == 40 {
            self.video_dynamic_params.stnb =
                162.0 / (rtime.powf(1.7) / bbbv) * (b.bbbv_solved as f64 / bbbv).powf(0.5);
        } else if self.height == 16 && self.width == 30 && self.mine_num == 99 {
            self.video_dynamic_params.stnb =
                435.0 / (rtime.powf(1.7) / bbbv) * (b.bbbv_solved as f64 / bbbv).powf(0.5);
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

        //         if self.height > 36 || self.width > 36 {
        //     return
        // }
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
}

impl ByteReader for BaseVideo<Vec<Vec<i32>>> {
    fn raw_data(&self) -> &[u8] {
        &self.raw_data
    }

    fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
    }
}

// 单参数的初始化方法。可以通过文件名（除js端）、二进制数据
pub trait NewBaseVideo<T> {
    fn new(arg: T) -> Self;
}

// 双参数的初始化方法。可以通过局面+像素边长
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
impl NewBaseVideo2<Vec<Vec<i32>>, u8> for BaseVideo<Vec<Vec<i32>>> {
    fn new(board: Vec<Vec<i32>>, cell_pixel_size: u8) -> BaseVideo<Vec<Vec<i32>>> {
        let mine_num = board.iter().fold(0, |y, row| {
            y + row
                .iter()
                .fold(0, |yy, x| if *x == -1 { yy + 1 } else { yy })
        });
        let board_clone = board.clone();
        BaseVideo {
            width: board[0].len(),
            height: board.len(),
            mine_num,
            cell_pixel_size,
            board,
            minesweeper_board: MinesweeperBoard::<Vec<Vec<i32>>>::new(board_clone),
            game_board_state: GameBoardState::Ready,
            static_params: StaticParams::default(),
            ..BaseVideo::<Vec<Vec<i32>>>::default()
        }
    }
}

/// 游戏前实例化，游戏中不断调用step方法来维护。
#[cfg(any(feature = "py", feature = "rs"))]
impl NewBaseVideo2<Vec<Vec<i32>>, u8> for BaseVideo<SafeBoard> {
    fn new(board: Vec<Vec<i32>>, cell_pixel_size: u8) -> BaseVideo<SafeBoard> {
        // 安全性考虑，此时禁止计算3BV
        // let bbbv = cal_bbbv(&board);
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
                // bbbv,
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
    /// 重置游戏状态等，不重置标识等很多局都不会变的数据。点脸等，重开。
    pub fn reset(&mut self, row: usize, column: usize, pix_size: u8)
    where
        T: std::ops::Index<usize> + BoardSize + EmptyBoard,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        self.game_board_stream.clear();
        self.minesweeper_board = MinesweeperBoard::<T>::new_from_board(row, column);
        self.width = column;
        self.height = row;
        self.cell_pixel_size = pix_size;
        self.video_action_state_recorder.clear();
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
    /// 两种情况调用：游戏开始前、可猜模式（尺寸相等，雷数不检查）
    pub fn set_board(&mut self, board: Vec<Vec<i32>>) -> Result<u8, ()>
    where
        T: std::ops::Index<usize> + BoardSize + From<Vec<Vec<i32>>> + Clone,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        assert!(!board.is_empty());
        match self.game_board_state {
            GameBoardState::Ready | GameBoardState::PreFlaging => {
                if self.width != board[0].len() || self.height != board.len() {
                    return Err(());
                }
            }
            GameBoardState::Playing => {
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
        self.board = T::from(board.clone());
        if self.game_board_state == GameBoardState::Playing {
            self.minesweeper_board.replace_board(self.board.clone());
        } else {
            self.minesweeper_board.board = self.board.clone();
        }
        Ok(0)
    }
    // 播放阶段计算pluck。Display阶段返回缓存值，Win/Loss阶段实时计算。
    pub fn get_pluck(&mut self) -> Result<f64, ()> {
        match self.game_board_state {
            GameBoardState::Display => Ok(self.video_action_state_recorder[self.current_event_id]
                .key_dynamic_params
                .pluck),
            GameBoardState::Win | GameBoardState::Loss => {
                let pluck = self.video_analyse_params.pluck;
                if pluck.is_nan() {
                    let mut pluck = 0.0;
                    let mut has_begin = false;
                    for vas in self.video_action_state_recorder.iter_mut() {
                        if let Some(Event::Mouse(e)) = &vas.event {
                            if vas.useful_level == 2 {
                                if !has_begin {
                                    has_begin = true;
                                    continue;
                                }
                                let x = (e.y / self.cell_pixel_size as u16) as usize;
                                let y = (e.x / self.cell_pixel_size as u16) as usize;
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
    /// 实施鼠标动作
    /// - pos的单位是像素，(距离上方，距离左侧)
    /// - 如果操作发生在界外，要求转换成pos=(row*pixsize, column*pixsize)
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
                // 同步局面状态机，否则后续鼠标事件仍会被当成有效操作处理
                self.minesweeper_board.game_board_state = GameBoardState::Loss;
                self.is_completed = false;
                self.gather_params_after_game(time_ms, None);
                Ok(0)
            }
            _ => Err(()),
        }
    }
    /// 获胜后标上所有的雷，没获胜则返回
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
            // 可猜模式中，假如局面发生过修改，且以点脸等reset方式结束游戏，就没有最后一步鼠标操作
            if let Some(mouse_e) = last_mouse_event {
                let r = mouse_e.y as usize / self.cell_pixel_size as usize;
                let c = mouse_e.x as usize / self.cell_pixel_size as usize;
                self.minesweeper_board.step(&mouse_e.mouse, (r, c)).unwrap();
            }
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
    // 计算除fps、bbbv以外的静态指标。因为bbbv是计算动态指标的基础
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
        self.static_params.zini = cal_zini(&self.board);
        self.static_params.hzini = cal_hzini(&self.board);
    }

    pub fn print_event(&self, flag_print_game_board: bool) {
        let num = 0;
        for e in &self.video_action_state_recorder {
            if num < 800 {
                if let Some(Event::Mouse(mouse_event)) = &e.event {
                    if mouse_event.mouse != "mv" {
                        println!(
                            "》time = {:?}, mouse = {:?}, x = {:?}, y = {:?}, level = {:?}",
                            e.time,
                            mouse_event.mouse,
                            mouse_event.x / self.get_pix_size().unwrap() as u16,
                            mouse_event.y / self.get_pix_size().unwrap() as u16,
                            e.useful_level
                        );
                    }
                    if mouse_event.mouse != "mv" && flag_print_game_board {
                        println!(
                            "《time = {:?}, mouse = {:?}, x = {:?}, y = {:?}",
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
        // 2023-04-24 00:00:00 前，发现影响公平性的漏洞失效
        "Viennasweeper" => Ok(("0".to_string(), "1682265600".to_string())),
        // 2024-07-26 00:00:00 发布，2026-03-03 00:00:00发现影响公平性的漏洞失效
        "元3.1.9" => Ok(("1721836800".to_string(), "1772467200".to_string())),
        // 2024-09-10 00:00:00 发布，2026-03-03 00:00:00发现影响公平性的漏洞失效
        "元3.1.11" => Ok(("1725811200".to_string(), "1772467200".to_string())),
        // 2024-11-21 00:00:00 发布，2026-03-03 00:00:00发现影响公平性的漏洞失效
        "元3.2.0" => Ok(("1732118400".to_string(), "1772467200".to_string())),
        // 2025-11-22 00:00:00 发布，2026-03-03 00:00:00发现影响公平性的漏洞失效
        "元3.2.1" => Ok(("1763740800".to_string(), "1772467200".to_string())),
        // 2026-03-03 00:00:00 发布
        "元3.2.2" => Ok(("1772467200".to_string(), "4102415999".to_string())),
        // 2026-06-25 00:00:00 发布
        "元3.3.1" => Ok(("1782316800".to_string(), "4102415999".to_string())),
        // 2026-07-07 00:00:00 发布
        "元3.3.2" => Ok(("1783353600".to_string(), "4102415999".to_string())),
        // 2026-07-10 00:00:00 发布
        "元3.3.3" => Ok(("1783612800".to_string(), "4102415999".to_string())),
        // 2026-08-11 00:00:00 发布
        "元3.3.4" => Ok(("1786377600".to_string(), "4102415999".to_string())),
        _ => Err(String::from("Unknown software: ") + software),
    }
}

/// 计算一段鼠标移动的方向熵和转向模式熵。
///
/// 输入是一段连续的 `mv` 坐标。少于32个非零位移或总路径长度少于32像素时返回 `None`，
/// 因为短轨迹本来就可能只朝一个方向，不能据此判断为程序化轨迹。
/// `Some(true)` 表示方向熵或转向模式熵低于复核阈值；`Some(false)` 表示两者都未低于阈值。
fn mouse_trace_segment_has_low_entropy(points: &[(u16, u16)]) -> Option<bool> {
    const MIN_SEGMENT_STEPS: usize = 32;
    const MIN_SEGMENT_DISTANCE_PX: f64 = 32.0;
    const DIRECTION_BINS: usize = 16;
    const TURN_SIGN_DEADBAND_RAD: f64 = 0.05235987755982989;
    const LOW_DIRECTION_ENTROPY: f64 = 0.15;
    const LOW_TURN_TRANSITION_ENTROPY: f64 = 0.20;

    let mut direction_counts = [0usize; DIRECTION_BINS];
    let mut turn_transition_counts = [0usize; 4];
    let mut headings = Vec::with_capacity(points.len().saturating_sub(1));
    let mut path_length = 0.0;

    for pair in points.windows(2) {
        let delta_x = pair[1].0 as f64 - pair[0].0 as f64;
        let delta_y = pair[1].1 as f64 - pair[0].1 as f64;
        if delta_x == 0.0 && delta_y == 0.0 {
            continue;
        }

        path_length += delta_x.hypot(delta_y);
        let heading = delta_y.atan2(delta_x);
        let direction_bin = (((heading + std::f64::consts::PI)
            / (2.0 * std::f64::consts::PI)
            * DIRECTION_BINS as f64) as usize)
            .min(DIRECTION_BINS - 1);
        direction_counts[direction_bin] += 1;
        headings.push(heading);
    }

    // 短路径的方向类别少是正常现象；不足32个有效步或32像素时，不根据熵推断轨迹真假。
    if headings.len() < MIN_SEGMENT_STEPS || path_length < MIN_SEGMENT_DISTANCE_PX {
        return None;
    }

    // 忽略小于3度的转向，降低像素取整导致的微小角度抖动。
    // 统计连续有效转向的四种变化：负转向接负/正转向、正转向接负/正转向。
    // 直线几乎没有有效转向；匀顺的样条曲线通常长期保持同一转向方向。
    let mut previous_turn_sign = 0i8;
    for pair in headings.windows(2) {
        let turn = (pair[1] - pair[0] + std::f64::consts::PI)
            .rem_euclid(2.0 * std::f64::consts::PI)
            - std::f64::consts::PI;
        let turn_sign = if turn > TURN_SIGN_DEADBAND_RAD {
            1
        } else if turn < -TURN_SIGN_DEADBAND_RAD {
            -1
        } else {
            0
        };
        if turn_sign != 0 {
            if previous_turn_sign != 0 {
                let transition = match (previous_turn_sign, turn_sign) {
                    (-1, -1) => 0,
                    (-1, 1) => 1,
                    (1, -1) => 2,
                    (1, 1) => 3,
                    _ => unreachable!(),
                };
                turn_transition_counts[transition] += 1;
            }
            previous_turn_sign = turn_sign;
        }
    }

    // Shannon 熵先量化各类别的出现比例 p，再计算 -Σ(p * ln(p)) / ln(类别数)。
    // 归一化结果在0到1之间：接近0表示几乎都落在少数类别，接近1表示分布较均匀。
    let normalized_entropy = |counts: &[usize]| {
        let total: usize = counts.iter().sum();
        if total == 0 {
            return 0.0;
        }
        let entropy = counts
            .iter()
            .filter(|&&count| count > 0)
            .map(|&count| {
                let probability = count as f64 / total as f64;
                -probability * probability.ln()
            })
            .sum::<f64>();
        entropy / (counts.len() as f64).ln()
    };

    // 特征1：方向熵。把每个非零位移的角度分入16个方向格（每格22.5度），统计方向分布。
    // 低于0.15表示位移高度集中于少数方向，直线或接近直线的轨迹容易触发。
    let direction_entropy = normalized_entropy(&direction_counts);
    // 特征2：转向模式熵。相邻方向变化超过3度才算有效转向，再统计左/右转向的4种连续组合。
    // 低于0.20表示转向组合集中，长期只向一侧平滑弯曲的轨迹容易触发。
    let turn_transition_entropy = normalized_entropy(&turn_transition_counts);
    // 0.15/0.20是用于筛查的经验阈值，不是生理学或真实用户的硬边界；低熵仅作为可疑段证据。
    Some(
        direction_entropy < LOW_DIRECTION_ENTROPY
            || turn_transition_entropy < LOW_TURN_TRANSITION_ENTROPY,
    )
}

// 录像审核相关的方法
impl<T> BaseVideo<T> {
    /// 按排名网站的自动审核规则检查录像。必须先调用 `analyse()`，再调用本方法。
    ///
    /// 检查顺序：
    /// 1. 检查录像软件是否支持，并按软件版本检查校验和、是否公平及允许的游戏模式。
    /// 2. 检查录像事件是否足以提供操作细节，以及录像是否完成。
    /// 3. 最后调用 [`user_mouse_trace_valid`](Self::user_mouse_trace_valid)，检查重复移动、
    ///    按键按住时长、极短按次数和轨迹熵等鼠标特征。
    ///
    /// 本方法不检查校验码，也不能仅凭鼠标轨迹证明录像一定由真人操作。也不检查录像的时间戳是否在软件的合法时间范围内。
    ///
    /// 返回值：
    /// - `0`：通过。通过当前自动检查，未发现已实现规则标记的异常；不代表绝对证明真实。
    /// - `1`：不通过。发现明确不符合规则的情况，例如录像未完成、软件/公平性校验失败，或鼠标特征达到不通过阈值。
    /// - `3`：可疑。无法确认或需要人工复核，例如操作细节不足、模式不在允许范围，或鼠标特征达到可疑阈值。
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
            || self.software == "元3.2.2"
            || self.software == "元3.3.1"
            || self.software == "元3.3.2"
            || self.software == "元3.3.3"
            || self.software == "元3.3.4"
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
            // 其他版本都不合法
            return 1;
        }

        if self.video_action_state_recorder.len() as f64 / (self.static_params.bbbv as f64) < 10.0 {
            // 缺少鼠标移动细节，报不确定
            return 3;
        }

        if !self.is_completed {
            return 1;
        }

        self.user_mouse_trace_valid()
    }

    /// 检查录像中的鼠标轨迹是否有明显程序化特征。
    ///
    /// ## 提取的特征与判定标准
    /// 1. **重复移动步长**：相邻移动事件的 `(dx, dy, 毫秒间隔)` 四舍五入到整数毫秒后作为
    ///    一个特征签名。若同一签名连续出现至少20步，且占所有非零移动步数至少75%，返回 `1`。
    ///    这是针对高度重复的合成移动的强信号；普通平滑移动不满足“完全相同”的要求。
    /// 2. **按键按住时长**：配对鼠标按下与抬起事件，计算每次 `抬起时间 - 按下时间`。
    ///    时长为0的记录不参与统计，因为录像格式可能把两事件量化到同一时间；负值或非有限时间返回 `3`。
    ///    可测时长至少5次才计算中位数：中位数低于30ms返回 `1`；30ms（含）到40ms（不含）返回 `3`；
    ///    40ms（含）以上不因该特征单独触发。300段公开录像中，逐录像中位数的P10为66.5ms，
    ///    总体中位数为80ms，因此30/40ms设在样本低分位以下作为保守筛查线，而非声称真人不会低于此值。
    ///    使用中位数而非最短按压，是为了降低偶发短按对判定的影响。
    /// 3. **极短按次数**：可测时长小于9.5ms计为一次（10ms阈值留0.5ms误差余量）。
    ///    最多3次忽略，4到10次返回 `3` 供复核，超过10次返回 `1`。
    ///    单次或少量极短按不作为作弊结论。
    /// 4. **轨迹方向熵与转向模式熵**：每段连续移动分别计算方向分布（16个方向格）和连续左右转向组合
    ///    （4种组合）的归一化 Shannon 熵。段至少32个非零步且路径长度至少32像素才参与统计；
    ///    熵低于方向0.15或转向0.20的段算低熵段。至少有3个可评估段，且其中至少75%为低熵段时返回 `3`。
    ///
    /// 熵阈值（0.15/0.20）及重复移动阈值（20步、75%）是启发式筛查值，未用样本训练或验证为分类边界；
    /// 按键时长阈值参考上述300段样本的分布。它们都不是所有真人操作的硬边界，也不能证明硬件或用户身份。
    /// 返回 `0` 仅表示这些特征未触发，不等于证明录像一定由真人操作。
    ///
    /// ## 返回值
    /// - `0`：特征未触发异常规则。
    /// - `1`：高度重复移动，或有足够按键样本且按住时长中位数低于30ms。
    /// - `3`：信息不足或出现需要人工复核的信号（中位数低于40ms、极短按过多、多个轨迹段低熵等）。
    ///
    /// 鼠标回报率和物理惯性无法从这些游戏录像可靠推断；本方法不对这两项作判断。
    pub fn user_mouse_trace_valid(&self) -> u8 {
        // 少于这些样本时不计算轨迹熵；这种情况返回“不确定”，不当作通过或作弊证据。
        const MIN_MOVE_EVENTS: usize = 20;
        const MIN_MOVING_STEPS: usize = 8;

        // 完全相同的(dx, dy, 毫秒间隔)连续出现20次，且占全部移动步数至少75%，
        // 视为明显机械重复并返回1。这里只抓很强的重复特征，不把普通平滑轨迹判假。
        const ROBOTIC_RUN_LENGTH: usize = 20;

        // 低于10ms的按压用于统计极短点击；最多3次忽略，4到10次要求复核，超过10次判不通过。
        // 另看可测按压时长的中位数，至少需要5次按压才计算，且排除被时间戳量化为0ms的时长。
        // 样本按压中位数主要落在60-100ms：低于40ms只作可疑复核，低于30ms判不通过；
        // 这是适配新手和高手样本的经验阈值，不单凭单次短按判定。
        const SHORT_CLICK_THRESHOLD_SECONDS: f64 = 0.01;
        const CLICK_TIME_ROUNDING_GUARD_SECONDS: f64 = 0.0005;
        const MAX_IGNORED_SHORT_CLICKS: usize = 3;
        const MAX_REVIEW_SHORT_CLICKS: usize = 10;
        const MIN_CLICK_HOLDS_FOR_MEDIAN: usize = 5;
        const SUSPECT_MEDIAN_HOLD_THRESHOLD_SECONDS: f64 = 0.04;
        const FAIL_MEDIAN_HOLD_THRESHOLD_SECONDS: f64 = 0.03;

        let mut move_events = 0;
        let mut moving_steps = 0;
        let mut longest_identical_run = 0;
        let mut identical_run = 0;
        let mut previous_signature: Option<(i64, i64, i64)> = None;
        let mut previous_move: Option<(u16, u16, f64)> = None;
        let mut button_down_times = [None; 3];
        let mut resolvable_short_clicks = 0;
        let mut measurable_click_holds = Vec::new();
        let mut current_segment = Vec::new();
        let mut low_entropy_segments = 0;
        let mut eligible_segments = 0;

        // 把连续的“mv”事件当作一段移动；按钮/其他鼠标事件会结束当前段。
        // 只统计达到长度和距离下限的段，避免短促的格间移动天然低熵而被计入。
        let mut record_segment = |points: &[(u16, u16)]| {
            if let Some(has_low_entropy) = mouse_trace_segment_has_low_entropy(points) {
                eligible_segments += 1;
                if has_low_entropy {
                    low_entropy_segments += 1;
                }
            }
        };

        for record in &self.video_action_state_recorder {
            let Some(Event::Mouse(mouse_event)) = &record.event else {
                continue;
            };
            let button_index = match mouse_event.mouse.as_str() {
                "lc" => Some((0, true)),
                "lr" => Some((0, false)),
                "rc" => Some((1, true)),
                "rr" => Some((1, false)),
                "mc" => Some((2, true)),
                "mr" => Some((2, false)),
                _ => None,
            };
            if let Some((index, is_down)) = button_index {
                record_segment(&current_segment);
                current_segment.clear();
                identical_run = 0;
                previous_signature = None;
                previous_move = None;
                if !record.time.is_finite() {
                    return 3;
                }
                if is_down {
                    button_down_times[index] = Some(record.time);
                } else if let Some(down_time) = button_down_times[index].take() {
                    let hold_time = record.time - down_time;
                    if !hold_time.is_finite() || hold_time < 0.0 {
                        return 3;
                    }
                    if hold_time > 0.0 {
                        measurable_click_holds.push(hold_time);
                        if hold_time
                            < SHORT_CLICK_THRESHOLD_SECONDS - CLICK_TIME_ROUNDING_GUARD_SECONDS
                        {
                            resolvable_short_clicks += 1;
                        }
                    }
                }
                continue;
            }
            if mouse_event.mouse != "mv" {
                record_segment(&current_segment);
                current_segment.clear();
                identical_run = 0;
                previous_signature = None;
                previous_move = None;
                continue;
            }
            move_events += 1;
            current_segment.push((mouse_event.x, mouse_event.y));
            if !record.time.is_finite() {
                return 3;
            }

            if let Some((last_x, last_y, last_time)) = previous_move {
                let delta_x = mouse_event.x as i64 - last_x as i64;
                let delta_y = mouse_event.y as i64 - last_y as i64;
                if delta_x != 0 || delta_y != 0 {
                    let delta_time = record.time - last_time;
                    if !delta_time.is_finite() || delta_time < 0.0 {
                        return 3;
                    }
                    moving_steps += 1;
                    let signature = (delta_x, delta_y, (delta_time * 1000.0).round() as i64);
                    if previous_signature == Some(signature) {
                        identical_run += 1;
                    } else {
                        identical_run = 1;
                        previous_signature = Some(signature);
                    }
                    longest_identical_run = max(longest_identical_run, identical_run);
                } else {
                    identical_run = 0;
                    previous_signature = None;
                }
            }
            previous_move = Some((mouse_event.x, mouse_event.y, record.time));
        }

        record_segment(&current_segment);

        if move_events < MIN_MOVE_EVENTS || moving_steps < MIN_MOVING_STEPS {
            return 3;
        }

        if resolvable_short_clicks > MAX_REVIEW_SHORT_CLICKS {
            return 1;
        }
        if resolvable_short_clicks > MAX_IGNORED_SHORT_CLICKS {
            return 3;
        }

        if measurable_click_holds.len() >= MIN_CLICK_HOLDS_FOR_MEDIAN {
            measurable_click_holds.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let middle = measurable_click_holds.len() / 2;
            let median_hold_time = if measurable_click_holds.len() % 2 == 0 {
                (measurable_click_holds[middle - 1] + measurable_click_holds[middle]) / 2.0
            } else {
                measurable_click_holds[middle]
            };
            // Ignore sub-nanosecond floating-point subtraction noise at exact boundaries.
            if FAIL_MEDIAN_HOLD_THRESHOLD_SECONDS - median_hold_time > 1e-9 {
                return 1;
            }
            if SUSPECT_MEDIAN_HOLD_THRESHOLD_SECONDS - median_hold_time > 1e-9 {
                return 3;
            }
        }

        // 至少有3段可评估轨迹，并且其中75%以上方向/转向熵低，才标记为需要复核。
        // 需要多个独立段是为了避免把单次直线移动或一次平滑转弯误当成自动化证据。
        if eligible_segments >= 3 && low_entropy_segments * 4 >= eligible_segments * 3 {
            return 3;
        }

        if longest_identical_run >= ROBOTIC_RUN_LENGTH
            && longest_identical_run * 4 >= moving_steps * 3
        {
            return 1;
        }

        0
    }
}
