use crate::utils::cal_board_numbers;
use crate::videos::base_video::{BaseVideo, NewBaseVideo};
use crate::videos::byte_reader::ByteReader;
use crate::videos::types::{ErrReadVideoReason, Event, MouseEvent, VideoActionStateRecorder};
#[cfg(any(feature = "py", feature = "rs"))]
use crate::videos::NewSomeVideo;
use crate::videos::NewSomeVideo2;

/// rmv录像解析器。  
/// - 功能：解析rmv格式的录像(Vienna MineSweeper产生的)，有详细分析录像的方法。  
/// - 以下是在python中调用的示例。  
/// ```python
/// v = ms.RmvVideo("video_name.rmv") # 第一步，读取文件的二进制内容
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
    pub fn parse(&mut self) -> Result<(), ErrReadVideoReason> {
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
        let format_version = match self.data.get_u16() {
            Ok(format_version) => format_version,
            _ => return Err(ErrReadVideoReason::FileIsNotRmv),
        };

        if format_version == 0 || format_version > 2 {
            // TODO: maybe add a better reason here?
            // this is compatible with how it used to work, tho
            // Perhaps VersionBackward is a better fit?
            return Err(ErrReadVideoReason::FileIsNotRmv);
        }

        let clone_id = if format_version >= 2 {
            self.data.get_u8()?
        } else { 0 };

        let _major_version_of_clone = if format_version >= 2 {
            self.data.get_u8()?
        } else { 0 };

        // skip file_size
        self.data.offset += 4;

        let result_string_size = if format_version == 1 {
            self.data.get_u16()?
        } else { 0 };
        let version_info_size = self.data.get_u16()?;

        // skip player_info_size
        // skip board_size
        self.data.offset += 4;
        // self.data.get_unsized_int4()?;
        let preflags_size = self.data.get_u16()?;
        let properties_size = self.data.get_u16()?;
        let _extension_properties_size = if format_version >= 2 {
            self.data.get_u16()?
        } else { 0 };

        // skip vid_size
        // skip checksum_size
        self.data.offset += 6;

        if format_version == 1 {
            // ignore leading newline
            self.data.offset += 1;
            self.data.static_params.bbbv = 0;
            if result_string_size > 35 {
                // full result string
                // go to 31 before the end, as that is where bbbv will start
                // this relies on "#NF:?#TIMESTAMP:1234567890#" always having
                // the same length
                // if the bbbv has less than 3 digits, this will start 1-2 chars
                // early, but that's OK as they should never be digits
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

                // "#NF:?#TIMESTAMP:??????????"
                // no need to parse the timestamp - it is always the same as
                // timestamp_boardgen if present
                self.data.offset += 26;
            } else if result_string_size >= 3 {
                // this should always be >= 3, but let's be on the safe side
                // here we have a reduced result string that doesn't contain bbbv
                // => just ignore the whole result string!
                // we subtract -3 for leading \n and trailing #\n
                self.data.offset += result_string_size as usize - 3;
            }
            // trailing "#\n"
            self.data.offset += 2;
        }
        // skip version_info
        self.data.offset += version_info_size as usize;

        // 这里是uint16，不合理
        let num_player_info = self.data.get_u16()?;

        let mut token = vec![];

        // we need to store these in a buffer first, as how we parse them needs
        // to depend on the utf8 property
        let mut player_identifier_buffer = vec![];
        let mut uniqueness_identifier_buffer = vec![];
        let mut country_buffer = vec![];
        if num_player_info > 0 {
            let name_length = self.data.get_u8()?;
            player_identifier_buffer = self.data.get_buffer(name_length)?;
        }
        // 昵称不解析
        if num_player_info > 1 {
            let nick_length = self.data.get_u8()?;
            uniqueness_identifier_buffer = self.data.get_buffer(nick_length)?;
        }
        if num_player_info > 2 {
            let country_length = self.data.get_u8()?;
            country_buffer = self.data.get_buffer(country_length)?;
        }
        // 令牌不解析
        if num_player_info > 3 {
            let token_length = self.data.get_u8()?;
            token = self.data.get_buffer(token_length as usize)?;
        }

        // timestamp_boardgen
        let timestamp_boardgen: u32 = self.data.get_u32()?.into();

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
        let mut preflags_items = vec![];
        if preflags_size > 0 {
            let num_pre_flags = self.data.get_u16()?;
            for _ in 0..num_pre_flags {
                let c = self.data.get_u8()? as u16;
                let d = self.data.get_u8()? as u16;
                preflags_items.push((c, d));
            }
        }
        let preflags_items = preflags_items;

        let mut square_size = 16u8; // v1 default
        let mut utf8 = true; // v2 default
        // skip questionmarks property
        self.data.offset += 1;
        self.data.nf = if self.data.get_u8()? == 1 {
            true
        } else {
            false
        };
        self.data.mode = self.data.get_u8()? as u16;
        self.data.level = self.data.get_u8()? + 3;
        let mut properties_read = 4;

        if format_version >= 2 {
            let bbbv_low = self.data.get_u8()? as usize;
            let bbbv_high = self.data.get_u8()? as usize;
            self.data.static_params.bbbv = bbbv_low + (bbbv_high << 8);
            square_size = self.data.get_u8()?;
            properties_read += 3;
        } else {
            utf8 = false;
            if properties_size > 4 {
                utf8 = if self.data.get_u8()? == 1 {
                    true
                } else {
                    false
                };
                properties_read += 1;
            }
        }
        let square_size = square_size;
        let utf8 = utf8;
        self.data.cell_pixel_size = square_size;

        // ignore remaining properties
        self.data.offset += (properties_size - properties_read) as usize;

        // read extension properties. In the process, also set the clone_name,
        // if provided.
        // clone_name is a replacement for clone_id for new clones that don't
        // have an ID assigned yet.
        let mut clone_name = None;
        if format_version >= 2 {
            let num_extension_properties = self.data.get_u16()?;
            for _ii in 0..num_extension_properties {
                let key_size = self.data.get_u8()?;
                let key = self.data.get_utf8_string(key_size as usize)?;
                let value_size = self.data.get_u8()?;
                let value = self.data.get_buffer(value_size as usize)?;
                if key == "clone_name" {
                    clone_name = Some(String::from_utf8(value));
                }
            }
        }
        let clone_name = match clone_name {
            None => None,
            Some(Ok(s)) => Some(s),
            Some(Err(_)) => {
                return Err(ErrReadVideoReason::Utf8Error);
            }
        };

        if utf8 {
            // verify that text fields that we read are valid utf-8 as specified by the RMV spec
            let utf8_errfunc = |_e| ErrReadVideoReason::Utf8Error;
            self.data.player_identifier = String::from_utf8(player_identifier_buffer).map_err(utf8_errfunc)?;
            self.data.uniqueness_identifier = String::from_utf8(uniqueness_identifier_buffer).map_err(utf8_errfunc)?;
            self.data.country = String::from_utf8(country_buffer).map_err(utf8_errfunc)?;
            let _ = String::from_utf8(token.clone()).map_err(utf8_errfunc)?;
        }
        else {
            self.data.player_identifier = <BaseVideo<Vec<Vec<i32>>>>::get_unknown_cp_encoding_string_from_buf(player_identifier_buffer)?;
            self.data.uniqueness_identifier = <BaseVideo<Vec<Vec<i32>>>>::get_unknown_cp_encoding_string_from_buf(uniqueness_identifier_buffer)?;
            self.data.country = <BaseVideo<Vec<Vec<i32>>>>::get_unknown_cp_encoding_string_from_buf(country_buffer)?;
        }

        // 是不是第一个操作。录像里省略了第一个左键按下。
        let mut first_op_flag = true;
        let xoffset = if format_version == 1 {12} else {0};
        let yoffset = if format_version == 1 {56} else {0};
        let (mut x, mut y, mut time) = (0u16, 0u16, 0u32);
        for (c, d) in preflags_items {
            self.data
                .video_action_state_recorder
                .push(VideoActionStateRecorder {
                    event: Some(Event::Mouse(MouseEvent {
                        mouse: "pf".to_string(),
                        x: c * self.data.cell_pixel_size as u16,
                        y: d * self.data.cell_pixel_size as u16,
                    })),
                    ..VideoActionStateRecorder::default()
                });
        }
        loop {
            let c = self.data.get_u8()?;
            if c == 0 {
                if format_version >= 2 {
                    return Err(ErrReadVideoReason::InvalidVideoEvent);
                }
                self.data.offset += 4;
            } else if c <= 7 || (format_version >= 2 && c == 28 && !first_op_flag) {
                if c == 28 {
                    time += self.data.get_u8()? as u32;
                    let mv = self.data.get_u8()?;
                    // mv is two 4bit two's complement signed integers packed into a single byte
                    // n & 8 = leading digit, the one that has negative weight in two's complement
                    // n & 7 = remaining three digits
                    // for x, we shift into position first
                    x = x.wrapping_add(((mv >> 4) & 7u8) as u16);
                    x = x.wrapping_sub(((mv >> 4) & 8u8) as u16);
                    y = y.wrapping_add((mv & 7u8) as u16);
                    y = y.wrapping_sub((mv & 8u8) as u16);
                } else {
                    time = self.data.get_u32()? >> 8;
                    x = (self.data.get_u16()?).wrapping_sub(xoffset);
                    y = (self.data.get_u16()?).wrapping_sub(yoffset);
                }
                if c >= 1 {
                    if x >= self.data.width as u16 * self.data.cell_pixel_size as u16
                        || y >= self.data.height as u16 * self.data.cell_pixel_size as u16 {
                        x = self.data.width as u16 * self.data.cell_pixel_size as u16;
                        y = self.data.height as u16 * self.data.cell_pixel_size as u16;
                    }
                    if first_op_flag {
                        first_op_flag = false;
                        self.data
                            .video_action_state_recorder
                            .push(VideoActionStateRecorder {
                                time: time as f64 / 1000.0,
                                event: Some(Event::Mouse(MouseEvent {
                                    mouse: "lc".to_string(),
                                    x,
                                    y,
                                })),
                                ..VideoActionStateRecorder::default()
                            });
                    }
                    self.data
                        .video_action_state_recorder
                        .push(VideoActionStateRecorder {
                            time: time as f64 / 1000.0,
                            event: Some(Event::Mouse(MouseEvent {
                                mouse: match c {
                                    1 => "mv".to_string(),
                                    2 => "lc".to_string(),
                                    3 => "lr".to_string(),
                                    4 => "rc".to_string(),
                                    5 => "rr".to_string(),
                                    6 => "mc".to_string(),
                                    7 => "mr".to_string(),
                                    28 => "mv".to_string(),
                                    _ => return Err(ErrReadVideoReason::InvalidVideoEvent),
                                },
                                x,
                                y,
                            })),
                            ..VideoActionStateRecorder::default()
                        });
                }
            } else if c == 8 {
                return Err(ErrReadVideoReason::InvalidParams);
            } else if c <= 14 || (c >= 18 && c <= 27) {
                self.data.offset += 2;
            } else if c <= 17 {
                self.data.is_completed = c == 16;
                break;
            } else {
                return Err(ErrReadVideoReason::InvalidParams);
            }
        }
        self.data
            .set_rtime(self.data.video_action_state_recorder.last().unwrap().time)
            .unwrap();
        self.data.start_time = timestamp_boardgen as u64 * 1000000;
        self.data.end_time =
            self.data.start_time + (self.data.get_rtime_ms().unwrap() as u64) * 1000;
        self.data.software = if format_version == 1 {
            "Viennasweeper".to_string()
        } else {
            match clone_id {
                0 => match clone_name {
                    Some(s) => s,
                    None => {
                        return Err(ErrReadVideoReason::InvalidParams)
                    },
                },
                1 => "Viennasweeper".to_string(),
                _ => "Unknown".to_string(),
            }
        };
        self.data.is_official = self.data.is_completed;
        self.data.is_fair = self.data.is_completed;
        self.data.can_analyse = true;
        return Ok(());
    }
}
