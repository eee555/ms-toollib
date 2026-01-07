use crate::GameStateEvent;
use crate::utils::cal_board_numbers;
use crate::videos::base_video::{BaseVideo, NewBaseVideo};
use crate::videos::byte_reader::ByteReader;
use crate::videos::types::{ErrReadVideoReason, Event, MouseEvent, VideoActionStateRecorder};
#[cfg(any(feature = "py", feature = "rs"))]
use crate::videos::NewSomeVideo;
use crate::videos::NewSomeVideo2;

/// evf录像解析器。  
/// - 功能：解析evf格式的录像(唯一的计算机易读、开源的录像格式)，有详细分析录像的方法。  
/// - 以下是在python中调用的示例。  
/// ```python
/// v = ms.EvfVideo("video_name.evf") # 第一步，读取文件的二进制内容
/// v.parse() # 第二步，解析文件的二进制内容
/// v.analyse() # 第三步，根据解析到的内容，推衍整个局面
/// video.current_time = 999.999 # set time to the end of the video
/// print(video.left)
/// print(video.right)
/// print(video.double)
/// print(video.left_s)
/// print(video.right_s)
/// print(video.double_s)
/// print(video.level)
/// print(video.cl)
/// print(video.cl_s)
/// print(video.ce)
/// print(video.ce_s)
/// print(video.bbbv)
/// print(video.bbbv_solved)
/// print(video.bbbv_s)
/// print(video.flag)
/// print(video.path)
/// print(video.time)  # the time shown on the counter currently
/// print(video.rtime) # game time, shown on leaderboard
/// print(video.etime) # the estimated time shown on the counter currently
/// print(video.video_start_time)
/// print(video.video_end_time)
/// print(video.mode)
/// print(video.software)
/// print(video.stnb)
/// print(video.corr)
/// print(video.thrp)
/// print(video.ioe)
/// print(video.is_official)
/// print(video.is_fair)
/// print("对象上的所有属性和方法：" + dir(v))
/// v.analyse_for_features(["high_risk_guess"]) # 用哪些分析方法。分析结果会记录到events.comments里
/// for i in range(v.events_len):
///     print(v.events_time(i), v.events_x(i), v.events_y(i), v.events_mouse(i))
/// for i in range(v.events_len):
///     if v.events_useful_level(i) >= 2:
///         print(v.events_posteriori_game_board(i).poss)
/// ```
#[derive(Clone)]
pub struct EvfVideo {
    pub file_name: String,
    pub version: u8,
    pub data: BaseVideo<Vec<Vec<i32>>>,
}

#[cfg(any(feature = "py", feature = "rs"))]
impl NewSomeVideo<&str> for EvfVideo {
    /// 从文件名创建EvfVideo实例
    fn new(file_name: &str) -> Self {
        let data = BaseVideo::<Vec<Vec<i32>>>::new(file_name);
        EvfVideo {
            file_name: file_name.to_string(),
            version: data.get_raw_data().unwrap()[0],
            data,
        }
    }
}

impl NewSomeVideo2<Vec<u8>, &str> for EvfVideo {
    /// 从二进制数据和虚拟的文件名创建EvfVideo实例
    fn new(raw_data: Vec<u8>, file_name: &str) -> Self {
        EvfVideo {
            file_name: file_name.to_string(),
            version: raw_data[0],
            data: BaseVideo::<Vec<Vec<i32>>>::new(raw_data),
        }
    }
}

impl EvfVideo {
    pub fn parse(&mut self) -> Result<(), ErrReadVideoReason> {
        if self.data.can_analyse {
            return Ok(());
        }
        let version = self.data.get_u8()?;
        match version {
            0 | 1 => self.parse_v1(),
            2 => self.parse_v2(),
            3 => self.parse_v3(),
            4 => self.parse_v4(),
            _ => Err(ErrReadVideoReason::VersionBackward),
        }
    }

    /// 0.0-0.1版本
    fn parse_v1(&mut self) -> Result<(), ErrReadVideoReason> {
        let the_byte = self.data.get_u8()?;
        self.data.is_completed = the_byte & 0b1000_0000 != 0;
        self.data.is_official = the_byte & 0b0100_0000 != 0;
        self.data.is_fair = the_byte & 0b0010_0000 != 0;
        self.data.nf = the_byte & 0b0001_0000 != 0;
        self.data.height = self.data.get_u8()? as usize;
        self.data.width = self.data.get_u8()? as usize;
        self.data.mine_num = self.data.get_u16()? as usize;
        if self.data.height == 8 && self.data.width == 8 && self.data.mine_num == 10 {
            self.data.level = 3;
        } else if self.data.height == 16 && self.data.width == 16 && self.data.mine_num == 40 {
            self.data.level = 4;
        } else if self.data.height == 16 && self.data.width == 30 && self.data.mine_num == 99 {
            self.data.level = 5;
        } else {
            self.data.level = 6;
        }

        self.data.cell_pixel_size = self.data.get_u8()?;
        self.data.mode = self.data.get_u16()?;
        self.data.static_params.bbbv = self.data.get_u16()? as usize;
        let t = self.data.get_u24()?;
        self.data.set_rtime(t as f64 / 1000.0).unwrap();
        self.data.software = self.data.get_utf8_c_string('\0')?;
        self.data.translated = !self.data.software.starts_with("元");
        self.data.player_identifier = self.data.get_unknown_encoding_c_string('\0')?;
        self.data.race_identifier = self.data.get_unknown_encoding_c_string('\0')?;
        self.data.uniqueness_identifier = self.data.get_unknown_encoding_c_string('\0')?;
        let start_time = self.data.get_utf8_c_string('\0')?;
        let end_time = self.data.get_utf8_c_string('\0')?;
        match self.data.software.as_str() {
            "Arbiter" => {
                self.data.start_time = self.data.parse_avf_start_timestamp(&start_time)?;
                self.data.end_time = self.data.parse_avf_end_timestamp(&start_time, &end_time)?;
            }
            "Viennasweeper" => {
                self.data.start_time = start_time
                    .parse::<u64>()
                    .map_err(|_| ErrReadVideoReason::InvalidParams)?
                    * 1000000;
                self.data.end_time = self.data.start_time + (t as u64) * 1000;
            }
            software @ _ if software.starts_with("元") => {
                self.data.start_time = start_time
                    .parse::<u64>()
                    .map_err(|_| ErrReadVideoReason::InvalidParams)?;
                self.data.end_time = end_time
                    .parse::<u64>()
                    .map_err(|_| ErrReadVideoReason::InvalidParams)?;
            }
            _ => {}
        }
        self.data.country = self.data.get_unknown_encoding_c_string('\0')?;
        self.data.board = vec![vec![0; self.data.width]; self.data.height];

        let mut byte = 0;
        let mut ptr = 0;
        for i in 0..self.data.height {
            for j in 0..self.data.width {
                ptr -= 1;
                if ptr < 0 {
                    byte = self.data.get_u8()?;
                    ptr = 7;
                }
                if byte & (1 << ptr) != 0 {
                    self.data.board[i][j] = -1
                }
            }
        }
        cal_board_numbers(&mut self.data.board);
        let have_checksum;
        loop {
            let byte = self.data.get_u8()?;
            let mouse;
            match byte {
                0 => {
                    have_checksum = true;
                    break;
                }
                1 => mouse = "mv",
                2 => mouse = "lc",
                3 => mouse = "lr",
                4 => mouse = "rc",
                5 => mouse = "rr",
                6 => mouse = "mc",
                7 => mouse = "mr",
                8 => mouse = "pf",
                9 => mouse = "cc",
                255 => {
                    have_checksum = false;
                    break;
                }
                _ => mouse = "ub", // 不可能
            }
            let time = self.data.get_u24()? as f64 / 1000.0;
            let x = self.data.get_u16()?;
            let y = self.data.get_u16()?;
            self.data
                .video_action_state_recorder
                .push(VideoActionStateRecorder {
                    time,
                    event: Some(Event::Mouse(MouseEvent {
                        mouse: mouse.to_string(),
                        x,
                        y,
                    })),
                    ..VideoActionStateRecorder::default()
                });
        }
        let mut csum = vec![];
        if have_checksum {
            for _ in 0..32 {
                csum.push(self.data.get_u8()?);
            }
        }
        self.data.checksum = csum;
        self.data.can_analyse = true;
        return Ok(());
    }
    /// 0.2版本
    fn parse_v2(&mut self) -> Result<(), ErrReadVideoReason> {
        let the_byte = self.data.get_u8()?;
        self.data.is_completed = the_byte & 0b1000_0000 != 0;
        self.data.is_official = the_byte & 0b0100_0000 != 0;
        self.data.is_fair = the_byte & 0b0010_0000 != 0;
        self.data.nf = the_byte & 0b0001_0000 != 0;
        self.data.height = self.data.get_u8()? as usize;
        self.data.width = self.data.get_u8()? as usize;
        self.data.mine_num = self.data.get_u16()? as usize;
        if self.data.height == 8 && self.data.width == 8 && self.data.mine_num == 10 {
            self.data.level = 3;
        } else if self.data.height == 16 && self.data.width == 16 && self.data.mine_num == 40 {
            self.data.level = 4;
        } else if self.data.height == 16 && self.data.width == 30 && self.data.mine_num == 99 {
            self.data.level = 5;
        } else {
            self.data.level = 6;
        }

        self.data.cell_pixel_size = self.data.get_u8()?;
        self.data.mode = self.data.get_u16()?;
        self.data.static_params.bbbv = self.data.get_u16()? as usize;
        let t = self.data.get_u24()?;
        self.data.set_rtime(t as f64 / 1000.0).unwrap();

        self.data.software = self.data.get_utf8_c_string('\0')?;
        self.data.translated = !self.data.software.starts_with("元");
        self.data.player_identifier = self.data.get_unknown_encoding_c_string('\0')?;
        self.data.race_identifier = self.data.get_unknown_encoding_c_string('\0')?;
        self.data.uniqueness_identifier = self.data.get_unknown_encoding_c_string('\0')?;
        let start_time = self.data.get_utf8_c_string('\0')?;
        let end_time = self.data.get_utf8_c_string('\0')?;
        match self.data.software.as_str() {
            "Arbiter" => {
                self.data.start_time = self.data.parse_avf_start_timestamp(&start_time)?;
                self.data.end_time = self.data.parse_avf_end_timestamp(&start_time, &end_time)?;
            }
            "Viennasweeper" => {
                self.data.start_time = start_time
                    .parse::<u64>()
                    .map_err(|_| ErrReadVideoReason::InvalidParams)?
                    * 1000000;
                self.data.end_time = self.data.start_time + (t as u64) * 1000;
            }
            software @ _ if software.starts_with("元") => {
                self.data.start_time = start_time
                    .parse::<u64>()
                    .map_err(|_| ErrReadVideoReason::InvalidParams)?;
                self.data.end_time = end_time
                    .parse::<u64>()
                    .map_err(|_| ErrReadVideoReason::InvalidParams)?;
            }
            _ => {}
        }
        self.data.country = self.data.get_unknown_encoding_c_string('\0')?;
        self.data.device_uuid = self.data.get_c_buffer('\0')?;
        self.data.board = vec![vec![0; self.data.width]; self.data.height];

        let mut byte = 0;
        let mut ptr = 0;
        for i in 0..self.data.height {
            for j in 0..self.data.width {
                ptr -= 1;
                if ptr < 0 {
                    byte = self.data.get_u8()?;
                    ptr = 7;
                }
                if byte & (1 << ptr) != 0 {
                    self.data.board[i][j] = -1
                }
            }
        }
        cal_board_numbers(&mut self.data.board);
        let have_checksum;
        loop {
            let byte = self.data.get_u8()?;
            let mouse;
            match byte {
                0 => {
                    have_checksum = true;
                    break;
                }
                1 => mouse = "mv",
                2 => mouse = "lc",
                3 => mouse = "lr",
                4 => mouse = "rc",
                5 => mouse = "rr",
                6 => mouse = "mc",
                7 => mouse = "mr",
                8 => mouse = "pf",
                9 => mouse = "cc",
                255 => {
                    have_checksum = false;
                    break;
                }
                _ => mouse = "ub", // 不可能
            }
            let time = self.data.get_u24()? as f64 / 1000.0;
            let x = self.data.get_u16()?;
            let y = self.data.get_u16()?;
            self.data
                .video_action_state_recorder
                .push(VideoActionStateRecorder {
                    time,
                    event: Some(Event::Mouse(MouseEvent {
                        mouse: mouse.to_string(),
                        x,
                        y,
                    })),
                    ..VideoActionStateRecorder::default()
                });
        }
        let mut csum = vec![];
        if have_checksum {
            for _ in 0..32 {
                csum.push(self.data.get_u8()?);
            }
        }
        self.data.checksum = csum;
        self.data.can_analyse = true;
        return Ok(());
    }
    /// 0.3版本
    fn parse_v3(&mut self) -> Result<(), ErrReadVideoReason> {
        let the_byte = self.data.get_u8()?;
        self.data.is_completed = the_byte & 0b1000_0000 != 0;
        self.data.is_official = the_byte & 0b0100_0000 != 0;
        self.data.is_fair = the_byte & 0b0010_0000 != 0;
        self.data.nf = the_byte & 0b0001_0000 != 0;
        let the_byte = self.data.get_u8()?;
        self.data.use_question = the_byte & 0b1000_0000 != 0;
        self.data.use_cursor_pos_lim = the_byte & 0b0100_0000 != 0;
        self.data.use_auto_replay = the_byte & 0b0010_0000 != 0;
        self.data.height = self.data.get_u8()? as usize;
        self.data.width = self.data.get_u8()? as usize;
        self.data.mine_num = self.data.get_u16()? as usize;
        if self.data.height == 8 && self.data.width == 8 && self.data.mine_num == 10 {
            self.data.level = 3;
        } else if self.data.height == 16 && self.data.width == 16 && self.data.mine_num == 40 {
            self.data.level = 4;
        } else if self.data.height == 16 && self.data.width == 30 && self.data.mine_num == 99 {
            self.data.level = 5;
        } else {
            self.data.level = 6;
        }

        self.data.cell_pixel_size = self.data.get_u8()?;
        self.data.mode = self.data.get_u16()?;
        self.data.static_params.bbbv = self.data.get_u16()? as usize;
        let t = self.data.get_u24()?;
        self.data.set_rtime(t as f64 / 1000.0).unwrap();

        self.data.software = self.data.get_utf8_c_string('\0')?;
        self.data.translated = !self.data.software.starts_with("元");
        self.data.player_identifier = self.data.get_unknown_encoding_c_string('\0')?;
        self.data.race_identifier = self.data.get_unknown_encoding_c_string('\0')?;
        self.data.uniqueness_identifier = self.data.get_unknown_encoding_c_string('\0')?;
        let start_time = self.data.get_utf8_c_string('\0')?;
        let end_time = self.data.get_utf8_c_string('\0')?;
        match self.data.software.as_str() {
            "Arbiter" => {
                self.data.start_time = self.data.parse_avf_start_timestamp(&start_time)?;
                self.data.end_time = self.data.parse_avf_end_timestamp(&start_time, &end_time)?;
            }
            "Viennasweeper" => {
                self.data.start_time = start_time
                    .parse::<u64>()
                    .map_err(|_| ErrReadVideoReason::InvalidParams)?
                    * 1000000;
                self.data.end_time = self.data.start_time + (t as u64) * 1000;
            }
            software @ _ if software.starts_with("元") => {
                self.data.start_time = start_time
                    .parse::<u64>()
                    .map_err(|_| ErrReadVideoReason::InvalidParams)?;
                self.data.end_time = end_time
                    .parse::<u64>()
                    .map_err(|_| ErrReadVideoReason::InvalidParams)?;
            }
            _ => {}
        }
        self.data.country = self.data.get_unknown_encoding_c_string('\0')?;
        self.data.device_uuid = self.data.get_c_buffer('\0')?;
        self.data.board = vec![vec![0; self.data.width]; self.data.height];

        let mut byte = 0;
        let mut ptr = 0;
        for i in 0..self.data.height {
            for j in 0..self.data.width {
                ptr -= 1;
                if ptr < 0 {
                    byte = self.data.get_u8()?;
                    ptr = 7;
                }
                if byte & (1 << ptr) != 0 {
                    self.data.board[i][j] = -1
                }
            }
        }
        cal_board_numbers(&mut self.data.board);
        let have_checksum;
        loop {
            let byte = self.data.get_u8()?;
            let mouse;
            match byte {
                0 => {
                    have_checksum = true;
                    break;
                }
                1 => mouse = "mv",
                2 => mouse = "lc",
                3 => mouse = "lr",
                4 => mouse = "rc",
                5 => mouse = "rr",
                6 => mouse = "mc",
                7 => mouse = "mr",
                8 => mouse = "pf",
                9 => mouse = "cc",
                10 => mouse = "l",
                11 => mouse = "r",
                12 => mouse = "m",
                255 => {
                    have_checksum = false;
                    break;
                }
                _ => mouse = "ub", // 不可能
            }
            let time = self.data.get_u24()? as f64 / 1000.0;
            let x = self.data.get_u16()?;
            let y = self.data.get_u16()?;
            self.data
                .video_action_state_recorder
                .push(VideoActionStateRecorder {
                    time,
                    event: Some(Event::Mouse(MouseEvent {
                        mouse: mouse.to_string(),
                        x,
                        y,
                    })),
                    ..VideoActionStateRecorder::default()
                });
        }
        let mut csum = vec![];
        if have_checksum {
            for _ in 0..32 {
                csum.push(self.data.get_u8()?);
            }
        }
        self.data.checksum = csum;
        self.data.can_analyse = true;
        return Ok(());
    }
    /// 0.4版本
    fn parse_v4(&mut self) -> Result<(), ErrReadVideoReason> {
        let the_byte = self.data.get_u8()?;
        self.data.is_completed = the_byte & 0b1000_0000 != 0;
        self.data.is_official = the_byte & 0b0100_0000 != 0;
        self.data.is_fair = the_byte & 0b0010_0000 != 0;
        self.data.nf = the_byte & 0b0001_0000 != 0;
        self.data.translated = the_byte & 0b0000_1000 != 0;
        let the_byte = self.data.get_u8()?;
        self.data.use_question = the_byte & 0b1000_0000 != 0;
        self.data.use_cursor_pos_lim = the_byte & 0b0100_0000 != 0;
        self.data.use_auto_replay = the_byte & 0b0010_0000 != 0;
        self.data.height = self.data.get_u8()? as usize;
        self.data.width = self.data.get_u8()? as usize;
        self.data.mine_num = self.data.get_u16()? as usize;
        if self.data.height == 8 && self.data.width == 8 && self.data.mine_num == 10 {
            self.data.level = 3;
        } else if self.data.height == 16 && self.data.width == 16 && self.data.mine_num == 40 {
            self.data.level = 4;
        } else if self.data.height == 16 && self.data.width == 30 && self.data.mine_num == 99 {
            self.data.level = 5;
        } else {
            self.data.level = 6;
        }

        self.data.cell_pixel_size = self.data.get_u8()?;
        self.data.mode = self.data.get_u16()?;
        self.data.static_params.bbbv = self.data.get_u16()? as usize;
        let t = self.data.get_u32()?;
        self.data.set_rtime(t as f64 / 1000.0).unwrap();
        self.data.country = self.data.get_utf8_string(2usize)?;
        self.data.start_time = self.data.get_u64()?;
        self.data.end_time = self.data.get_u64()?;
        self.data.software = self.data.get_utf8_c_string('\0')?;
        if self.data.translated {
            self.data.translate_software = self.data.get_utf8_c_string('\0')?;
            self.data.original_encoding = self.data.get_utf8_c_string('\0')?;
        }
        self.data.player_identifier = self.data.get_utf8_c_string('\0')?;
        self.data.race_identifier = self.data.get_utf8_c_string('\0')?;
        self.data.uniqueness_identifier = self.data.get_utf8_c_string('\0')?;
        let device_uuid_length = self.data.get_u16()?;
        self.data.device_uuid = self.data.get_buffer(device_uuid_length)?;
        self.data.board = vec![vec![0; self.data.width]; self.data.height];
        let mut byte = 0;
        let mut ptr = 0;
        for i in 0..self.data.height {
            for j in 0..self.data.width {
                ptr -= 1;
                if ptr < 0 {
                    byte = self.data.get_u8()?;
                    ptr = 7;
                }
                if byte & (1 << ptr) != 0 {
                    self.data.board[i][j] = -1
                }
            }
        }
        cal_board_numbers(&mut self.data.board);
        // 自定义指标暂时不解析，没人用到
        let custom_index_num = self.data.get_u16()?;
        for _ in 0..custom_index_num {
            self.data.get_utf8_c_string('\0')?;
        }

        // 解析事件循环，暂时只包含鼠标事件、停顿事件
        let byte = self.data.get_u8()?;
        let mouse;
        match byte {
            1 => mouse = "mv",
            2 => mouse = "lc",
            4 => mouse = "rc",
            8 => mouse = "pf",
            _ => panic!(), // impossible
        }
        let time_ms = self.data.get_u8()? as u32;
        let time = time_ms as f64 / 1000.0;
        let x = self.data.get_u16()?;
        let y = self.data.get_u16()?;
        let event_0 = MouseEvent {
            mouse: mouse.to_string(),
            x,
            y,
        };
        // 增量计算鼠标坐标用，只有鼠标事件才有坐标
        let mut last_mouse_event = event_0.clone();
        // 增量时间戳用，所有事件都有时间戳
        let mut last_event_time_ms = time_ms;
        self.data
            .video_action_state_recorder
            .push(VideoActionStateRecorder {
                time,
                event: Some(Event::Mouse(event_0)),
                ..VideoActionStateRecorder::default()
            });
        // 累计的暂停时间
        let mut pause_time_ms = 0;
        // for i in 0..200{
        //     print!("{:?}, ", self.data.get_u8()?);
        // }
        loop {
            let byte = self.data.get_u8()?;
            match byte {
                0 => {
                    break;
                }
                // 开始解析鼠标事件
                byte_mouse @ 1..=80 => {
                    let mouse;
                    match byte_mouse {
                        1 => mouse = "mv",
                        2 => mouse = "lc",
                        3 => mouse = "lr",
                        4 => mouse = "rc",
                        5 => mouse = "rr",
                        6 => mouse = "mc",
                        7 => mouse = "mr",
                        8 => mouse = "pf",
                        9 => mouse = "cc",
                        10 => mouse = "l",
                        11 => mouse = "r",
                        12 => mouse = "m",
                        _ => panic!(),
                    }
                    let time_u8: u8 = self.data.get_u8()?;
                    let x = self.data.get_i16()?;
                    let y = self.data.get_i16()?;
                    // let last_event = self.data.video_action_state_recorder.last().unwrap();
                    let event_i = MouseEvent {
                        mouse: mouse.to_string(),
                        x: (last_mouse_event.x as i16 + x) as u16,
                        y: (last_mouse_event.y as i16 + y) as u16,
                    };
                    let time_i_ms = time_u8 as u32 + pause_time_ms + last_event_time_ms;
                    let time_i = time_i_ms as f64 / 1000.0;
                    last_mouse_event = event_i.clone();
                    last_event_time_ms = time_i_ms;
                    self.data
                        .video_action_state_recorder
                        .push(VideoActionStateRecorder {
                            time: time_i,
                            event: Some(Event::Mouse(event_i)),
                            ..VideoActionStateRecorder::default()
                        });
                    pause_time_ms = 0;
                }
                // 开始解析游戏状态事件
                byte_game_state @ 81..=99 => {
                    let game_state;
                    match byte_game_state {
                        81 => game_state = "replay",
                        82 => game_state = "win",
                        83 => game_state = "fail",
                        99 => game_state = "error",
                        _ => panic!(),
                    }
                    let time_u8: u8 = self.data.get_u8()?;
                    let event_i = GameStateEvent {
                        game_state: game_state.to_string(),
                    };
                    let time_i_ms = time_u8 as u32 + pause_time_ms + last_event_time_ms;
                    let time_i = time_i_ms as f64 / 1000.0;
                    last_event_time_ms = time_i_ms;
                    self.data
                        .video_action_state_recorder
                        .push(VideoActionStateRecorder {
                            time: time_i,
                            event: Some(Event::GameState(event_i)),
                            ..VideoActionStateRecorder::default()
                        });
                    pause_time_ms = 0;
                }
                _b @ 100..=199 => {}
                _b @ 200..=254 => {}
                // 开始解析停顿事件
                255 => {
                    let pause_time = self.data.get_u16()?;
                    pause_time_ms += pause_time as u32;
                    continue;
                }
            }
        }

        let checksum_length = self.data.get_u16()?;
        self.data.checksum = self.data.get_buffer(checksum_length)?;
        self.data.can_analyse = true;
        return Ok(());
    }
}
