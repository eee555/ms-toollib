use crate::utils::{cal_bbbv, cal_board_numbers};
use crate::videos::base_video::{BaseVideo, NewBaseVideo};
use crate::videos::byte_reader::ByteReader;
use crate::videos::types::{ErrReadVideoReason, Event, MouseEvent, VideoActionStateRecorder};
#[cfg(any(feature = "py", feature = "rs"))]
use crate::videos::NewSomeVideo;
use crate::videos::NewSomeVideo2;

/// mvf录像解析器。  
/// - 功能：解析mvf格式的录像，有详细分析录像的方法。  
/// - 以下是在python中调用的示例。  
/// ```python
/// v = ms.MvfVideo("video_name.mvf") # 第一步，读取文件的二进制内容
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
pub struct MvfVideo {
    pub file_name: String,
    pub data: BaseVideo<Vec<Vec<i32>>>,
}

#[cfg(any(feature = "py", feature = "rs"))]
impl NewSomeVideo<&str> for MvfVideo {
    fn new(file_name: &str) -> Self {
        MvfVideo {
            file_name: file_name.to_string(),
            data: BaseVideo::<Vec<Vec<i32>>>::new(file_name),
        }
    }
}

impl NewSomeVideo2<Vec<u8>, &str> for MvfVideo {
    fn new(raw_data: Vec<u8>, file_name: &str) -> Self {
        MvfVideo {
            file_name: file_name.to_string(),
            data: BaseVideo::<Vec<Vec<i32>>>::new(raw_data),
        }
    }
}

impl MvfVideo {
    fn read_board(&mut self, add: i32) -> Result<(), ErrReadVideoReason> {
        self.data.width = self.data.get_u8()?.into();
        self.data.height = self.data.get_u8()?.into();
        self.data.board = vec![vec![0; self.data.width]; self.data.height];

        self.data.mine_num = self.data.get_u16()?.into();
        for _ in 0..self.data.mine_num {
            let w = (self.data.get_u8()? as i32 + add) as usize;
            let h = (self.data.get_u8()? as i32 + add) as usize;
            // 要检查
            self.data.board[h][w] = -1;
        }
        cal_board_numbers(&mut self.data.board);
        Ok(())
    }
    fn parse_event(
        &self,
        lb: u16,
        rb: u16,
        mb: u16,
        x: u16,
        y: u16,
        prev_lb: u16,
        prev_rb: u16,
        prev_mb: u16,
        prev_x: u16,
        prev_y: u16,
    ) -> Vec<String> {
        let mut evs: Vec<String> = vec![];
        if x != prev_x || y != prev_y {
            evs.push("mv".to_string());
        }
        if lb > 0 && prev_lb == 0 {
            evs.push("lc".to_string());
        }
        if rb > 0 && prev_rb == 0 {
            evs.push("rc".to_string());
        }
        if mb > 0 && prev_mb == 0 {
            evs.push("mc".to_string());
        }
        if lb == 0 && prev_lb > 0 {
            evs.push("lr".to_string());
        }
        if rb == 0 && prev_rb > 0 {
            evs.push("rr".to_string());
        }
        if mb == 0 && prev_mb > 0 {
            evs.push("mr".to_string());
        }
        return evs;
    }
    fn apply_perm(&self, num: usize, byte: &[usize; 40], bit: &[u8; 40], e: &[u8; 5]) -> u16 {
        // 从c翻译过来，别问我
        if (e[byte[num]] & bit[num]) > 0 {
            return 1;
        } else {
            return 0;
        }
    }
    /// Clone 2006、2007的置换表，共48位。同样从rawvf的c代码翻译过来
    fn clone_perm_2007(leading: u16) -> ([usize; 48], [u8; 48]) {
        const MULT: f64 = 100000000.0;
        let num1 = (leading as f64).sqrt();
        let num2 = (leading as f64 + 1000.0).sqrt();
        let num3 = (num1 + 1000.0).sqrt();
        let num4 = (num2 + 1000.0).sqrt();
        let magic = [
            (num3 + 1000.0).cos(),
            num2.sqrt().sin(),
            num3.cos(),
            (num1.sqrt() + 1000.0).sin(),
            num4.cos(),
            num4.sin(),
        ];
        let mut s = [b'0'; 48];
        for (i, m) in magic.iter().enumerate() {
            let text = format!("{:08}", (*m * MULT).abs().round_ties_even() as u64);
            for (j, b) in text.as_bytes().iter().take(8).enumerate() {
                s[i * 8 + j] = *b;
            }
        }
        let mut byte = [0usize; 48];
        let mut bit = [0u8; 48];
        let mut cur = 0;
        for d in b'0'..=b'9' {
            for j in 0..48 {
                if s[j] == d {
                    byte[cur] = j / 8;
                    bit[cur] = 1 << (j % 8);
                    cur += 1;
                }
            }
        }
        (byte, bit)
    }
    fn apply_perm_2007(num: usize, byte: &[usize; 48], bit: &[u8; 48], e: &[u8; 6]) -> u16 {
        if (e[byte[num]] & bit[num]) > 0 {
            1
        } else {
            0
        }
    }
    /// 读取Clone 2006、2007的录像
    fn read_2007(&mut self) -> Result<(), ErrReadVideoReason> {
        let mut e = [0u8; 6];
        let cell = self.data.cell_pixel_size as i32;

        // 读时间戳
        let month = self.data.get_u8()? as u64;
        let day = self.data.get_u8()? as u64;
        let year = self.data.get_u16()? as u64;
        let hour = self.data.get_u8()? as u64;
        let minute = self.data.get_u8()? as u64;
        let second = self.data.get_u8()? as u64;
        // Clone 2007有把日期保存为00的bug，此时按当月1号计算
        let days = self.data.days_since_epoch(year, month, day.max(1));
        self.data.start_time = (days * 86400 + hour * 3600 + minute * 60 + second) * 1_000_000;

        // 下面2 bytes是难度和模式
        self.data.level = self.data.get_u8()? + 2;
        self.data.mode = match self.data.get_u8()? {
            1 => 0,
            2 => 3,
            3 => 1,
            _ => 2,
        };

        // 下面3 bytes是时间，单位毫秒
        let score_ms = self.data.get_u24()?;

        // 是否开启了问号
        self.data.use_question = self.data.get_u8()? != 0;

        // 读取局面
        self.read_board(-1)?;
        // Clone 2006、2007的录像不保存3BV，由局面计算得出
        self.data.static_params.bbbv = cal_bbbv(&self.data.board);

        // 读取玩家名
        let len = self.data.get_u8()?;
        self.data.player_identifier = self.data.get_unknown_encoding_string(len)?;

        // 前2 bytes决定文件的置换表
        let leading = self.data.get_u16()?;
        let (byte, bit) = Self::clone_perm_2007(leading);

        // 鼠标事件的数量
        let event_size = self.data.get_u24()?;
        let (w_px, h_px) = (
            self.data.width as i32 * cell,
            self.data.height as i32 * cell,
        );
        let mut prev_lb = 0u16;
        let mut prev_rb = 0u16;
        let mut prev_mb = 0u16;
        let mut prev_x = 0u16;
        let mut prev_y = 0u16;
        for idx in 0..event_size {
            for k in 0..6 {
                e[k] = self.data.get_u8()?;
            }
            let rb = Self::apply_perm_2007(0, &byte, &bit, &e);
            let mb = Self::apply_perm_2007(1, &byte, &bit, &e);
            let lb = Self::apply_perm_2007(2, &byte, &bit, &e);
            let mut raw_x = 0u16;
            let mut raw_y = 0u16;
            let mut ths = 0u32;
            for j in 0..11 {
                raw_x |= Self::apply_perm_2007(14 + j, &byte, &bit, &e) << j;
                raw_y |= Self::apply_perm_2007(3 + j, &byte, &bit, &e) << j;
            }
            for j in 0..22 {
                ths |= (Self::apply_perm_2007(25 + j, &byte, &bit, &e) as u32) << j;
            }
            // 坐标要去掉32的偏移；落在局面外的坐标统一映射到右下角，
            // 和0.97的处理方式一致（clone的录像局面外坐标不在右下角）。
            // 是否发生了移动仍然按文件里的原始坐标判断。
            let (xi, yi) = (raw_x as i32 - 32, raw_y as i32 - 32);
            let (x, y) = if xi < 0 || yi < 0 || xi >= w_px || yi >= h_px {
                (w_px as u16, h_px as u16)
            } else {
                (xi as u16, yi as u16)
            };
            let mouse_s = if idx == 0 {
                // clone不会保存计时器开始前的鼠标事件，因此第一个事件直接按下
                vec![if mb > 0 {
                    "mc".to_string()
                } else if rb > 0 {
                    "rc".to_string()
                } else if lb > 0 {
                    "lc".to_string()
                } else {
                    "mv".to_string()
                }]
            } else {
                self.parse_event(
                    lb, rb, mb, raw_x, raw_y, prev_lb, prev_rb, prev_mb, prev_x, prev_y,
                )
            };
            prev_x = raw_x;
            prev_y = raw_y;
            prev_lb = lb;
            prev_rb = rb;
            prev_mb = mb;

            for mouse in mouse_s {
                self.data
                    .video_action_state_recorder
                    .push(VideoActionStateRecorder {
                        time: ths as f64 / 1000.0,
                        event: Some(Event::Mouse(MouseEvent { mouse, x, y })),
                        ..VideoActionStateRecorder::default()
                    });
            }
        }
        // 计算rtime。录像记录的时间是从第一次左键抬起开始的
        let mut start_t = 0.0;
        for v in &self.data.video_action_state_recorder {
            if let Some(Event::Mouse(mouse_event)) = &v.event {
                if mouse_event.mouse == "lr" {
                    start_t = v.time;
                    break;
                }
            }
        }
        let mut rtime = 0.0;
        if let Some(last) = self.data.video_action_state_recorder.last() {
            rtime = last.time - start_t;
        }
        // 录像头部的成绩时间是权威的，两者理论上应该相等
        rtime = rtime.max(score_ms as f64 / 1000.0);
        self.data.set_rtime(rtime).unwrap();
        self.data.end_time =
            self.data.start_time + (self.data.get_rtime_ms().unwrap_or(0) as u64) * 1000;
        Ok(())
    }
    fn read_097(&mut self) -> Result<(), ErrReadVideoReason> {
        // 读时间戳
        const MULT: f64 = 100000000.0;
        let mut byte = [0; 40];
        let mut bit = [0u8; 40];
        let mut e = [0u8; 5];

        let month = self.data.get_u8()? as u64;
        let day = self.data.get_u8()? as u64;
        let year = self.data.get_u16()? as u64;
        let hour = self.data.get_u8()? as u64;
        let minute = self.data.get_u8()? as u64;
        let second = self.data.get_u8()? as u64;
        let days = self.data.days_since_epoch(year, month, day);
        let total_seconds = days * 86400 + hour * 3600 + minute * 60 + second;
        self.data.start_time = total_seconds * 1_000_000;

        // //Next 2 bytes are Level and Mode
        self.data.level = self.data.get_u8()? + 2;
        let mode = self.data.get_u8()?;
        if mode == 1 {
            self.data.mode = 0;
        } else if mode == 2 {
            self.data.mode = 3;
        } else if mode == 3 {
            self.data.mode = 1;
        } else {
            self.data.mode = 2;
        }

        // 下面3 bytes 是时间
        let _score_sec = self.data.get_u16()? as f64;
        let _score_ths = self.data.get_u8()? as f64 / 100.0;
        // 此时间是第一次左键按下的时间作为开始时间，到最后一次弹起，因此，此时间不准
        // self.data.set_rtime(_score_sec + _score_ths).unwrap();

        // 下面 11 bytes 只有 Clone 0.97有
        // 工具箱无法在解析录像时设置bbbv_solved、Left clicks、Double clicks、Right clicks这些值，而是在推演局面时自动计算
        let bbbv = self.data.get_u16()?;
        self.data.static_params.bbbv = bbbv.into();
        let bbbv_solved = self.data.get_u16()?;
        self.data.is_completed = bbbv == bbbv_solved;
        // 没有判断是否公正的方法，只能根据是否扫完
        self.data.is_official = self.data.is_completed;
        self.data.is_fair = self.data.is_completed;

        self.data.offset += 6; // Left clicks、Double clicks、Right clicks不读

        // Check if Questionmark option was turned on
        self.data.use_question = self.data.get_u8()? != 0;

        // Function gets Width, Height and Mines then reads board layout into memory
        self.read_board(-1)?;

        let byte_len = self.data.get_u8()?;
        self.data.player_identifier = self.data.get_unknown_encoding_string(byte_len)?;

        // First 2 bytes determine the file permutation
        let mut s = ['\0'; 40];
        let leading = self.data.get_u16()? as f64;
        let num1 = leading.sqrt();
        let num2 = (leading + 1000.0).sqrt();
        let num3 = (num1 + 1000.0).sqrt();

        let magic_code = &format!(
            "{:08}",
            ((num3 + 1000.0).cos() * MULT).abs().round() as usize
        );
        for i in 0..8 {
            s[i] = magic_code.chars().nth(i).unwrap();
        }

        let magic_code = &format!("{:08}", ((num2.sqrt()).sin() * MULT).abs().round() as usize);
        for i in 0..8 {
            s[i + 8] = magic_code.chars().nth(i).unwrap();
        }

        let magic_code = &format!("{:08}", (num3.cos() * MULT).abs().round() as usize);
        for i in 0..8 {
            s[i + 16] = magic_code.chars().nth(i).unwrap();
        }

        let magic_code = &format!(
            "{:08}",
            ((num1.sqrt() + 1000.0).sin() * MULT).abs().round() as usize
        );
        for i in 0..8 {
            s[i + 24] = magic_code.chars().nth(i).unwrap();
        }

        let magic_code = &format!(
            "{:08}",
            (((num2 + 1000.0).sqrt()).cos() * MULT).abs().round() as usize
        );
        for i in 0..8 {
            s[i + 32] = magic_code.chars().nth(i).unwrap();
        }

        let mut cur = 0;
        for i in '0'..='9' {
            for j in 0..40 {
                if s[j] == i {
                    byte[cur] = j / 8;
                    bit[cur] = 1 << (j % 8);
                    cur += 1;
                }
            }
        }

        let event_size = self.data.get_u24()?;
        let mut prev_rb;
        let mut prev_mb;
        let mut prev_lb;
        for ii in 0..5 {
            e[ii] = self.data.get_u8()?;
        }
        prev_rb = self.apply_perm(0, &byte, &bit, &e);
        prev_mb = self.apply_perm(1, &byte, &bit, &e);
        prev_lb = self.apply_perm(2, &byte, &bit, &e);
        // println!("rb: {:?}, mb: {:?}, lb: {:?}", rb, mb, lb);
        let mut x = 0u16;
        let mut y = 0u16;
        let mut prev_x = 0u16;
        let mut prev_y = 0u16;
        let mut ths = 0;
        let mut sec = 0;
        let mouse;
        for j in 0..9 {
            x |= self.apply_perm(12 + j, &byte, &bit, &e) << j;
            y |= self.apply_perm(3 + j, &byte, &bit, &e) << j;
        }
        for j in 0..7 {
            ths |= self.apply_perm(21 + j, &byte, &bit, &e) << j;
        }
        ths *= 10;
        for j in 0..10 {
            sec |= self.apply_perm(28 + j, &byte, &bit, &e) << j;
        }

        if prev_mb > 0 {
            mouse = "mc".to_string()
        } else if prev_rb > 0 {
            mouse = "rc".to_string()
        } else if prev_lb > 0 {
            mouse = "lc".to_string()
        } else {
            mouse = "mv".to_string()
        }

        // clone的录像，局面外的坐标不是在右下角，似乎是在最后出界的位置。
        // 此处主动修改为右下角。
        if x >= self.data.width as u16 * 16 || y >= self.data.height as u16 * 16 {
            x = self.data.width as u16 * 16;
            y = self.data.height as u16 * 16;
        }
        self.data
            .video_action_state_recorder
            .push(VideoActionStateRecorder {
                time: ths as f64 / 1000.0 + sec as f64,
                event: Some(Event::Mouse(MouseEvent { mouse, x, y })),
                ..VideoActionStateRecorder::default()
            });
        for _ in 0..event_size - 1 {
            for ii in 0..5 {
                e[ii] = self.data.get_u8()?;
            }
            let rb = self.apply_perm(0, &byte, &bit, &e);
            let mb = self.apply_perm(1, &byte, &bit, &e);
            let lb = self.apply_perm(2, &byte, &bit, &e);
            // println!("rb: {:?}, mb: {:?}, lb: {:?}", rb, mb, lb);
            let mut x = 0u16;
            let mut y = 0u16;
            let mut ths = 0;
            let mut sec = 0;
            let mouse_s;

            for j in 0..9 {
                x |= self.apply_perm(12 + j, &byte, &bit, &e) << j;
                y |= self.apply_perm(3 + j, &byte, &bit, &e) << j;
            }
            for j in 0..7 {
                ths |= self.apply_perm(21 + j, &byte, &bit, &e) << j;
            }
            ths *= 10;
            for j in 0..10 {
                sec |= self.apply_perm(28 + j, &byte, &bit, &e) << j;
            }

            mouse_s = self.parse_event(lb, rb, mb, x, y, prev_lb, prev_rb, prev_mb, prev_x, prev_y);
            prev_x = x;
            prev_y = y;
            prev_lb = lb;
            prev_rb = rb;
            prev_mb = mb;

            // clone的录像，局面外的坐标不是在右下角，似乎是在最后出界的位置。
            // 此处主动修改为右下角。
            if x >= self.data.width as u16 * 16 || y >= self.data.height as u16 * 16 {
                x = self.data.width as u16 * 16;
                y = self.data.height as u16 * 16;
            }

            for mouse in mouse_s {
                self.data
                    .video_action_state_recorder
                    .push(VideoActionStateRecorder {
                        time: ths as f64 / 1000.0 + sec as f64,
                        event: Some(Event::Mouse(MouseEvent { mouse, x, y })),
                        ..VideoActionStateRecorder::default()
                    });
            }
        }
        let mut start_t = 0.0;
        // 计算rtime。如前所述，录像记录的时间是从lc开始的，并不正确
        for v in &self.data.video_action_state_recorder {
            if let Some(Event::Mouse(mouse_event)) = &v.event {
                if mouse_event.mouse == "lr" {
                    start_t = v.time;
                    break;
                }
            }
        }
        let rtime = self.data.video_action_state_recorder.last().unwrap().time - start_t;
        self.data.set_rtime(rtime).unwrap();
        self.data.end_time = self.data.start_time + (self.data.get_rtime_ms().unwrap_or(0) as u64) * 1000;

        Ok(())
    }
    pub fn parse(&mut self) -> Result<(), ErrReadVideoReason> {
        self.data.can_analyse = true;
        let mut c = self.data.get_u8()?;
        let d = self.data.get_u8()?;
        let _size: usize; // 动作总数量
        if c == 0x11 && d == 0x4D {
            self.data.offset += 25;
            c = self.data.get_u8()?;
            if c as char == '5' {
                //Clone 0.97
                self.data.offset += 46;
                self.data.software = "0.97 beta".to_string();
                return self.read_097();
            } else if c as char == '6' || c as char == '7' {
                //Clone 2006、2007。版本号从53开始，正式数据从71开始
                self.data.offset = 53;
                let len = self.data.get_u8()?;
                self.data.software = self.data.get_unknown_encoding_string(len)?;
                self.data.offset = 71;
                return self.read_2007();
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}
