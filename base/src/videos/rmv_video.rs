use crate::utils::cal_board_numbers;
use crate::videos::base_video::{BaseVideo, ErrReadVideoReason, VideoActionStateRecorder};
use crate::videos::{NewSomeVideo, NewSomeVideo2};
use crate::videos::base_video::NewBaseVideo;

/// rmv录像解析器。  
/// - 功能：解析rmv格式的录像(Vienna MineSweeper产生的)，有详细分析录像的方法。  
/// - 以下是在python中调用的示例。  
/// ```python
/// v = ms.RmvVideo("video_name.rmv") # 第一步，读取文件的二进制内容
/// v.parse_video() # 第二步，解析文件的二进制内容
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
pub struct RmvVideo {
    pub file_name: String,
    pub data: BaseVideo<Vec<Vec<i32>>>,
}

#[cfg(any(feature = "py", feature = "rs"))]
impl NewSomeVideo<&str> for RmvVideo {
    fn new(file_name: &str) -> Self {
        RmvVideo {
            file_name: file_name.to_string(),
            data: BaseVideo::<Vec<Vec<i32>>>::new(file_name),
        }
    }
}

impl NewSomeVideo2<Vec<u8>, &str> for RmvVideo {
    fn new(raw_data: Vec<u8>, file_name: &str) -> Self {
        RmvVideo {
            file_name: file_name.to_string(),
            data: BaseVideo::<Vec<Vec<i32>>>::new(raw_data),
        }
    }
}

impl RmvVideo {
    // #[cfg(any(feature = "py", feature = "rs"))]
    // pub fn new(file_name: &str) -> RmvVideo {
    //     RmvVideo {
    //         file_name: file_name.to_string(),
    //         data: BaseVideo::<Vec<Vec<i32>>>::new(file_name),
    //     }
    // }
    // #[cfg(feature = "js")]
    // pub fn new(video_data: Vec<u8>, file_name: &str) -> RmvVideo {
    //     RmvVideo {
    //         file_name: file_name.to_string(),
    //         data: BaseVideo::<Vec<Vec<i32>>>::new(video_data),
    //     }
    // }
    pub fn parse_video(&mut self) -> Result<(), ErrReadVideoReason> {
        // self.data.is_completed; // 该格式解析前不能确定是否扫完
        self.data.is_official = true;
        self.data.is_fair = true;
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
            let mut timestamp = vec![];
            for _ in 0..10 {
                timestamp.push(self.data.get_u8()?);
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

        let mut player = vec![];
        let mut country = vec![];
        if num_player_info > 0 {
            let name_length = self.data.get_u8()?;
            for _ in 0..name_length {
                player.push(self.data.get_u8()?);
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
                country.push(self.data.get_u8()?);
            }
        }
        // 令牌不解析
        if num_player_info > 3 {
            let token_length = self.data.get_u8()?;
            for _ in 0..token_length {
                self.data.get_char()?;
            }
        }
        self.data.player_identifier = player;
        self.data.country = country;

        self.data.offset += 4;

        self.data.width = self.data.get_u8()?.into();
        self.data.height = self.data.get_u8()?.into();
        self.data.mine_num = self.data.get_u16()?.into();

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
                self.data
                    .video_action_state_recorder
                    .push(VideoActionStateRecorder {
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

        // 是不是第一个操作。录像里省略了第一个左键按下。
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
                        self.data
                            .video_action_state_recorder
                            .push(VideoActionStateRecorder {
                                time: time as f64 / 1000.0,
                                mouse: "lc".to_string(),
                                x,
                                y,
                                ..VideoActionStateRecorder::default()
                            });
                    }
                    self.data
                        .video_action_state_recorder
                        .push(VideoActionStateRecorder {
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
                            x,
                            y,
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
        self.data
            .set_rtime(self.data.video_action_state_recorder.last().unwrap().time)
            .unwrap();
        self.data.software = "Viennasweeper".as_bytes().to_vec();
        self.data.can_analyse = true;
        return Ok(());
    }
}
