use crate::MouseState;
use crate::miscellaneous::s_to_ms;
use crate::utils::{cal_board_numbers};
use std::cmp::{max, min};
use crate::videos::base_video::{BaseVideo, ErrReadVideoReason, VideoActionStateRecorder};


/// avf录像解析器。  
/// - 功能：解析avf格式的录像，有详细分析录像的方法。  
/// - 以下是在python中调用的示例。  
/// ```python
/// v = ms.AvfVideo("video_name.avf") # 第一步，读取文件的二进制内容
/// v.parse_video() # 第二步，解析文件的二进制内容
/// v.analyse() # 第三步，根据解析到的内容，推衍整个局面
/// print(v.bbbv)
/// print(v.clicks)
/// print(v.clicks_s)
/// print("对象上的所有属性和方法：" + dir(v))
/// v.analyse_for_features(["high_risk_guess"]) # 用哪些分析方法。分析结果会记录到events.comments里
/// for i in range(v.events_len):
///     print(v.events_time(i), v.events_x(i), v.events_y(i), v.events_mouse(i))
/// for i in range(v.events_len):
///     if v.events_useful_level(i) >= 2:
///         print(v.events_posteriori_game_board(i).poss)
/// ```
pub struct AvfVideo {
    pub file_name: String,
    pub data: BaseVideo,
}

impl AvfVideo {
    #[cfg(any(feature = "py", feature = "rs"))]
    pub fn new(file_name: &str) -> AvfVideo {
        AvfVideo {
            file_name: file_name.to_string(),
            data: BaseVideo::new_with_file(file_name),
        }
    }
    #[cfg(feature = "js")]
    pub fn new(video_data: Vec<u8>) -> AvfVideo {
        AvfVideo {
            file_name: file_name.to_string(),
            data: BaseVideo::new(video_data),
        }
    }
    pub fn parse_video(&mut self) -> Result<(), ErrReadVideoReason> {
        match self.data.get_u8() {
            Ok(_) => {}
            Err(_) => return Err(ErrReadVideoReason::FileIsEmpty),
        };
        self.data.offset += 4;
        self.data.level = self.data.get_u8()?;
        // println!("{:?}", self.data.level);
        match self.data.level {
            3 => {
                self.data.width = 8;
                self.data.height = 8;
                self.data.mine_num = 10;
            }
            4 => {
                self.data.width = 16;
                self.data.height = 16;
                self.data.mine_num = 40;
            }
            5 => {
                self.data.width = 30;
                self.data.height = 16;
                self.data.mine_num = 99;
            }
            6 => {
                self.data.width = self.data.get_u8()? as usize + 1;
                self.data.height = self.data.get_u8()? as usize + 1;
                self.data.mine_num = self.data.get_u16()? as usize;
            }
            _ => return Err(ErrReadVideoReason::InvalidLevel),
        }
        self.data.board = vec![vec![0; self.data.width]; self.data.height];
        for _ in 0..self.data.mine_num {
            let c = self.data.get_u8()? as usize;
            let d = self.data.get_u8()? as usize;
            self.data.board[c - 1][d - 1] = -1;
        }

        for x in 0..self.data.height {
            for y in 0..self.data.width {
                if self.data.board[x][y] == -1 {
                    for j in max(1, x) - 1..min(self.data.height, x + 2) {
                        for k in max(1, y) - 1..min(self.data.width, y + 2) {
                            if self.data.board[j][k] >= 0 {
                                self.data.board[j][k] += 1;
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
            buffer[2] = self.data.get_char()?;
            if buffer[0] == '['
                && (buffer[1] == '0' || buffer[1] == '1' || buffer[1] == '2' || buffer[1] == '3')
                && buffer[2] == '|'
            {
                break;
            }
        }
        loop {
            let v = self.data.get_char()?;
            match v {
                '|' => break,
                _ => self.data.start_time.push(v),
            }
        }
        // println!("666");
        // loop {
        //     let v = self.get_char()?;
        //     print!("{:?}", v as char);
        // }
        loop {
            let v = self.data.get_char()?;
            match v {
                '|' => break,
                _ => self.data.end_time.push(v),
            }
        }
        let v = self.data.get_char()?;
        let mut buffer: [char; 2];
        match v {
            '|' => buffer = ['\0', '|'],
            'B' => buffer = ['|', 'B'],
            _ => buffer = ['\0', '\0'],
        }
        // 此处以下10行的写法有危险
        loop {
            if buffer[0] == '|' && buffer[1] == 'B' {
                break;
            }
            buffer[0] = buffer[1];
            buffer[1] = self.data.get_char()?;
        }
        let mut s: String = "".to_string();
        loop {
            let v = self.data.get_char()?;
            match v {
                'T' => break,
                _ => s.push(v),
            }
        }
        self.data.static_params.bbbv = match s.parse() {
            Ok(v) => v,
            Err(_) => return Err(ErrReadVideoReason::InvalidParams),
        };
        let mut s: String = "".to_string();
        loop {
            let v = self.data.get_char()?;
            match v {
                ']' => break,
                _ => s.push(v),
            }
        }
        s = str::replace(&s, ",", "."); // 有些录像小数点是逗号
                                        // println!("{:?}", s);
        self.data.game_dynamic_params.rtime = match s.parse::<f64>() {
            Ok(v) => v - 1.0,
            Err(_) => return Err(ErrReadVideoReason::InvalidParams),
        };
        self.data.game_dynamic_params.rtime_ms = s_to_ms(self.data.game_dynamic_params.rtime);
        let mut buffer = [0u8; 8];
        while buffer[2] != 1 || buffer[1] > 1 {
            buffer[0] = buffer[1];
            buffer[1] = buffer[2];
            buffer[2] = self.data.get_u8()?;
        }
        for i in 3..8 {
            buffer[i] = self.data.get_u8()?;
        }
        loop {
            // if buffer[0] != 1 {
            // println!("{:?}, {:?}", ((buffer[6] as u16) << 8 | buffer[2] as u16) as f64 - 1.0
            // + (buffer[4] as f64) / 100.0, buffer[0]);}
            self.data.video_action_state_recorder.push(VideoActionStateRecorder {
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
                ..VideoActionStateRecorder::default()
            });
            for i in 0..8 {
                // ???????
                buffer[i] = self.data.get_u8()?;
            }
            if buffer[2] == 0 && buffer[6] == 0 {
                break;
            }
        }
        // 标识符
        while self.data.get_char()? != 'S' {}
        while self.data.get_char()? != 'k' {}
        while self.data.get_char()? != 'i' {}
        while self.data.get_char()? != 'n' {}
        while self.data.get_char()? != ':' {}
        while self.data.get_char()? != '\r' {}
        loop {
            let v = self.data.get_char()?;
            match v {
                '\r' => break,
                _ => self.data.player_designator.push(v),
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
}

/// rmv录像解析器。  
/// - 功能：解析rmv格式的录像(Vienna MineSweeper产生的)，有详细分析录像的方法。  
/// - 以下是在python中调用的示例。  
/// ```python
/// v = ms.RmvVideo("video_name.rmv") # 第一步，读取文件的二进制内容
/// v.parse_video() # 第二步，解析文件的二进制内容
/// v.analyse() # 第三步，根据解析到的内容，推衍整个局面
/// print(v.bbbv)
/// print(v.clicks)
/// print(v.clicks_s)
/// print("对象上的所有属性和方法：" + dir(v))
/// v.analyse_for_features(["high_risk_guess"]) # 用哪些分析方法。分析结果会记录到events.comments里
/// for i in range(v.events_len):
///     print(v.events_time(i), v.events_x(i), v.events_y(i), v.events_mouse(i))
/// for i in range(v.events_len):
///     if v.events_useful_level(i) >= 2:
///         print(v.events_posteriori_game_board(i).poss)
/// ```
pub struct RmvVideo {
    pub file_name: String,
    pub data: BaseVideo,
}

impl RmvVideo {
    #[cfg(any(feature = "py", feature = "rs"))]
    pub fn new(file_name: &str) -> RmvVideo {
        RmvVideo {
            file_name: file_name.to_string(),
            data: BaseVideo::new_with_file(file_name),
        }
    }
    #[cfg(feature = "js")]
    pub fn new(video_data: Vec<u8>) -> RmvVideo {
        RmvVideo {
            file_name: file_name.to_string(),
            data: BaseVideo::new(video_data),
        }
    }
    pub fn parse_video(&mut self) -> Result<(), ErrReadVideoReason> {
        match self.data.get_char() {
            Ok('*') => {}
            Ok(_) => return Err(ErrReadVideoReason::FileIsNotRmv),
            Err(_) => return Err(ErrReadVideoReason::FileIsEmpty),
        };
        match self.data.get_char() {
            Ok('r') => {}
            _ => return Err(ErrReadVideoReason::FileIsNotRmv),
        };
        match self.data.get_char() {
            Ok('m') => {}
            _ => return Err(ErrReadVideoReason::FileIsNotRmv),
        };
        match self.data.get_char() {
            Ok('v') => {}
            _ => return Err(ErrReadVideoReason::FileIsNotRmv),
        };
        match self.data.get_u16() {
            Ok(1u16) => {}
            _ => return Err(ErrReadVideoReason::FileIsNotRmv),
        };
        self.data.offset += 4;
        let result_string_size = self.data.get_u16()?;
        let version_info_size = self.data.get_u16()?;
        self.data.offset += 4;
        // self.data.get_unsized_int4()?;
        let preflags_size = self.data.get_u16()?; // Gets bytes 18-19
        let properties_size = self.data.get_u16()?; // Gets bytes 20-21
        self.data.offset += 7;

        if result_string_size > 35 {
            self.data.offset += (result_string_size - 32) as usize;
            // 这种录像格式，3BV最多只支持3位数，宽和高支持最大256，雷数最多65536
            // 注意，3BV如果解析得到0，说明局面没有完成（我认为这种设计并不合理）
            let mut bbbv: String = "".to_string();

            for _ in 0..3 {
                let v = self.data.get_char()?;
                if v as u8 >= 48 && v as u8 <= 57 {
                    bbbv.push(v);
                }
            }
            self.data.static_params.bbbv = match bbbv.parse() {
                Ok(v) => v,
                Err(_) => return Err(ErrReadVideoReason::InvalidParams),
            };
            self.data.offset += 16;

            // 2286-11-21以后，会遇到时间戳溢出
            let mut timestamp: String = "".to_string();
            for _ in 0..10 {
                timestamp.push(self.data.get_char()?);
            }
            self.data.start_time = timestamp;

            // 2 beta和更早的版本里没有3bv和时间戳
        } else {
            self.data.static_params.bbbv = 0;
            for _ in 0..result_string_size - 3 {
                self.data.get_u8()?;
            }
        }
        self.data.offset += version_info_size as usize + 2;

        // 这里是uint16，不合理
        let num_player_info = self.data.get_u16()?;

        let mut player: String = "".to_string();
        let mut country: String = "".to_string();
        if num_player_info > 0 {
            let name_length = self.data.get_u8()?;
            for _ in 0..name_length {
                player.push(self.data.get_char()?);
            }
        }
        // 昵称不解析
        if num_player_info > 1 {
            let nick_length = self.data.get_u8()?;
            for _ in 0..nick_length {
                self.data.get_char()?;
            }
        }
        if num_player_info > 2 {
            let country_length = self.data.get_u8()?;
            for _ in 0..country_length {
                country.push(self.data.get_char()?);
            }
        }
        // 令牌不解析
        if num_player_info > 3 {
            let token_length = self.data.get_u8()?;
            for _ in 0..token_length {
                self.data.get_char()?;
            }
        }
        self.data.player_designator = player;
        self.data.country = country;

        self.data.offset += 4;

        // Get board size and Mine details
        self.data.width = self.data.get_u8()?.into(); // Next byte is w so 8, 9 or 1E
        self.data.height = self.data.get_u8()?.into(); // Next byte is h so 8, 9 or 10
        self.data.mine_num = self.data.get_u16()?.into(); // Next two bytes are number of mines
                                                          // println!("{:?}", self.data.width);

        // Fetch board layout and put in memory
        self.data.board = vec![vec![0; self.data.width]; self.data.height];

        // Every 2 bytes is x,y with 0,0 being the top left corner
        for _ in 0..self.data.mine_num {
            let c = self.data.get_u8()? as usize;
            let d = self.data.get_u8()? as usize;
            if c >= self.data.width || d >= self.data.height {
                return Err(ErrReadVideoReason::InvalidMinePosition);
            }
            self.data.board[d][c] = -1;
        }
        cal_board_numbers(&mut self.data.board);
        // 开始前已经标上的雷
        if preflags_size > 0 {
            let num_pre_flags = self.data.get_u16()?;
            for _ in 0..num_pre_flags {
                let c = self.data.get_u8()? as u16;
                let d = self.data.get_u8()? as u16;
                // self.data.pre_flags.push((d, c));
                self.data.video_action_state_recorder.push(VideoActionStateRecorder {
                    mouse: "pf".to_string(),
                    x: d * 16,
                    y: c * 16,
                    ..VideoActionStateRecorder::default()
                });
            }
        }

        self.data.offset += 1;
        self.data.nf = if self.data.get_u8()? == 1 {
            true
        } else {
            false
        };
        self.data.mode = self.data.get_u8()? as u16;
        self.data.level = self.data.get_u8()? + 3;

        self.data.offset += (properties_size - 4) as usize;

        let mut first_op_flag = true;
        loop {
            let c = self.data.get_u8()?;
            if c == 0 {
                self.data.offset += 4;
            } else if c <= 7 {
                let time = self.data.get_u32()? >> 8;
                let mut x = (self.data.get_u16()?).wrapping_sub(12);
                let mut y = (self.data.get_u16()?).wrapping_sub(56);
                if c >= 1 {
                    if x >= self.data.width as u16 * 16 || y >= self.data.height as u16 * 16 {
                        x = self.data.width as u16 * 16;
                        y = self.data.height as u16 * 16;
                    }
                    if first_op_flag {
                        first_op_flag = false;
                        self.data.video_action_state_recorder.push(VideoActionStateRecorder {
                            time: time as f64 / 1000.0,
                            mouse: "lc".to_string(),
                            x,
                            y,
                            ..VideoActionStateRecorder::default()
                        });
                    }
                    self.data.video_action_state_recorder.push(VideoActionStateRecorder {
                        time: time as f64 / 1000.0,
                        mouse: match c {
                            1 => "mv".to_string(),
                            2 => "lc".to_string(),
                            3 => "lr".to_string(),
                            4 => "rc".to_string(),
                            5 => "rr".to_string(),
                            6 => "mc".to_string(),
                            7 => "mr".to_string(),
                            _ => return Err(ErrReadVideoReason::InvalidVideoEvent),
                        },
                        x: x,
                        y: y,
                        ..VideoActionStateRecorder::default()
                    });
                }
            } else if c == 8 {
                return Err(ErrReadVideoReason::InvalidParams);
            } else if c <= 14 || (c >= 18 && c <= 27) {
                self.data.offset += 2;
            } else if c <= 17 {
                break;
            } else {
                return Err(ErrReadVideoReason::InvalidParams);
            }
        }
        self.data.game_dynamic_params.rtime = self.data.video_action_state_recorder.last().unwrap().time;
        self.data.game_dynamic_params.rtime_ms = s_to_ms(self.data.game_dynamic_params.rtime);
        self.data.can_analyse = true;
        return Ok(());
    }
}
