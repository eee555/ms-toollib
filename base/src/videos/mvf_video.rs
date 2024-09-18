use crate::utils::cal_board_numbers;
use crate::videos::base_video::{BaseVideo, ErrReadVideoReason, VideoActionStateRecorder};
use crate::videos::{NewSomeVideo, NewSomeVideo2};
use crate::videos::base_video::NewBaseVideo;

/// mvf录像解析器。  
/// - 功能：解析mvf格式的录像，有详细分析录像的方法。  
/// - 以下是在python中调用的示例。  
/// ```python
/// v = ms.MvfVideo("video_name.mvf") # 第一步，读取文件的二进制内容
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
    // #[cfg(any(feature = "py", feature = "rs"))]
    // pub fn new(file_name: &str) -> MvfVideo {
    //     MvfVideo {
    //         file_name: file_name.to_string(),
    //         data: BaseVideo::<Vec<Vec<i32>>>::new(file_name),
    //     }
    // }
    // #[cfg(feature = "js")]
    // pub fn new(video_data: Vec<u8>, file_name: &str) -> MvfVideo {
    //     MvfVideo {
    //         file_name: file_name.to_string(),
    //         data: BaseVideo::<Vec<Vec<i32>>>::new(video_data),
    //     }
    // }
    fn read_board(&mut self, add: i32) -> Result<(), ErrReadVideoReason> {
        //     unsigned char c;
        // int board_sz,i,pos;
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
    fn read_097(&mut self) -> Result<(), ErrReadVideoReason> {
        // 读时间戳
        const MULT: f64 = 100000000.0;
        let mut byte = [0; 40];
        let mut bit = [0u8; 40];
        let mut e = [0u8; 5];

        let _month = self.data.get_u8()?;
        let _day = self.data.get_u8()?;
        let _year = self.data.get_u16()?;
        let _hour = self.data.get_u8()?;
        let _minute = self.data.get_u8()?;
        let _second = self.data.get_u8()?;

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
        let score_sec = self.data.get_u16()? as f64;
        let score_ths = self.data.get_u8()? as f64 / 100.0;
        self.data.set_rtime(score_sec + score_ths).unwrap();

        // 下面 11 bytes 只有 Clone 0.97有
        self.data.static_params.bbbv = self.data.get_u16()?.into();
        // bbbv_solved、Left clicks、Double clicks、Right clicks不读
        self.data.offset += 8;

        // Check if Questionmark option was turned on
        self.data.offset += 1;

        // Function gets Width, Height and Mines then reads board layout into memory
        self.read_board(-1)?;

        let byte_len = self.data.get_u8()?;
        for _ in 0..byte_len {
            let t = self.data.get_u8()?;
            self.data.player_identifier.push(t);
        }

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
        // println!("s: {:?}, byte: {:?}, bit: {:?}", s, byte, bit);

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
        self.data
            .video_action_state_recorder
            .push(VideoActionStateRecorder {
                time: ths as f64 / 1000.0 + sec as f64,
                mouse,
                x,
                y,
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

            for mouse in mouse_s {
                self.data
                    .video_action_state_recorder
                    .push(VideoActionStateRecorder {
                        time: ths as f64 / 1000.0 + sec as f64,
                        mouse,
                        x,
                        y,
                        ..VideoActionStateRecorder::default()
                    });
            }
        }

        // return 1;
        Ok(())
    }
    pub fn parse_video(&mut self) -> Result<(), ErrReadVideoReason> {
        self.data.can_analyse = true;
        // self.data.is_completed; // 该格式解析前不能确定是否扫完
        self.data.is_official = true;
        self.data.is_fair = true;
        let mut c = self.data.get_u8()?;
        let d = self.data.get_u8()?;
        let _size: usize; // 动作总数量
        if c == 0x11 && d == 0x4D {
            self.data.offset += 25;
            c = self.data.get_u8()?;
            if c as char == '5' {
                //Clone 0.97
                self.data.offset += 46;
                self.data.software = "0.97 beta".as_bytes().to_vec();
                return self.read_097();
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}
