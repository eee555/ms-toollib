use crate::utils::cal_board_numbers;
use crate::videos::base_video::{BaseVideo, ErrReadVideoReason, VideoActionStateRecorder};
use crate::videos::{NewSomeVideo, NewSomeVideo2};
use crate::videos::base_video::NewBaseVideo;

/// evf录像解析器。  
/// - 功能：解析evf格式的录像(唯一的计算机易读、开源的录像格式)，有详细分析录像的方法。  
/// - 以下是在python中调用的示例。  
/// ```python
/// v = ms.EvfVideo("video_name.evf") # 第一步，读取文件的二进制内容
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
pub struct EvfVideo {
    pub file_name: String,
    pub data: BaseVideo<Vec<Vec<i32>>>,
}

#[cfg(any(feature = "py", feature = "rs"))]
impl NewSomeVideo<&str> for EvfVideo {
    fn new(file_name: &str) -> Self {
        EvfVideo {
            file_name: file_name.to_string(),
            data: BaseVideo::<Vec<Vec<i32>>>::new(file_name),
        }
    }
}

impl NewSomeVideo2<Vec<u8>, &str> for EvfVideo {
    fn new(raw_data: Vec<u8>, file_name: &str) -> Self {
        EvfVideo {
            file_name: file_name.to_string(),
            data: BaseVideo::<Vec<Vec<i32>>>::new(raw_data),
        }
    }
}

impl EvfVideo {
    // #[cfg(any(feature = "py", feature = "rs"))]
    // pub fn new_with_file(file_name: &str) -> EvfVideo {
    //     EvfVideo {
    //         file_name: file_name.to_string(),
    //         data: BaseVideo::<Vec<Vec<i32>>>::new(file_name),
    //     }
    // }
    // pub fn new(video_data: Vec<u8>, file_name: &str) -> EvfVideo {
    //     EvfVideo {
    //         file_name: file_name.to_string(),
    //         data: BaseVideo::<Vec<Vec<i32>>>::new(video_data),
    //     }
    // }
    pub fn parse_video(&mut self) -> Result<(), ErrReadVideoReason> {
        let version = self.data.get_u8()?;
        match version {
            0 | 1 => self.parse_v1(),
            2 => self.parse_v2(),
            3 => self.parse_v3(),
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

        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.software.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.player_identifier.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.race_identifier.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.uniqueness_identifier.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.start_time.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.end_time.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.country.push(the_byte as u8);
        }

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
                    mouse: mouse.to_string(),
                    x,
                    y,
                    ..VideoActionStateRecorder::default()
                });
        }
        let mut csum = [0; 32];
        if have_checksum {
            for i in 0..32 {
                csum[i] = self.data.get_u8()?;
                // csum.push(self.data.get_char()?)
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

        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.software.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.player_identifier.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.race_identifier.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.uniqueness_identifier.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.start_time.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.end_time.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.country.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.device_uuid.push(the_byte as u8);
        }

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
                    mouse: mouse.to_string(),
                    x,
                    y,
                    ..VideoActionStateRecorder::default()
                });
        }
        let mut csum = [0; 32];
        if have_checksum {
            for i in 0..32 {
                csum[i] = self.data.get_u8()?;
                // csum.push(self.data.get_char()?)
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

        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.software.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.player_identifier.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.race_identifier.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.uniqueness_identifier.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.start_time.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.end_time.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.country.push(the_byte as u8);
        }
        loop {
            let the_byte = self.data.get_char()?;
            if the_byte == '\0' {
                break;
            }
            self.data.device_uuid.push(the_byte as u8);
        }

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
                    mouse: mouse.to_string(),
                    x,
                    y,
                    ..VideoActionStateRecorder::default()
                });
        }
        let mut csum = [0; 32];
        if have_checksum {
            for i in 0..32 {
                csum[i] = self.data.get_u8()?;
                // csum.push(self.data.get_char()?)
            }
        }
        self.data.checksum = csum;
        self.data.can_analyse = true;
        return Ok(());
    }
}
