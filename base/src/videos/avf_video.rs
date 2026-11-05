use crate::videos::base_video::{BaseVideo, NewBaseVideo};
use crate::videos::byte_reader::ByteReader;
use crate::videos::types::{ErrReadVideoReason, Event, MouseEvent, VideoActionStateRecorder};

#[cfg(any(feature = "py", feature = "rs"))]
use crate::videos::NewSomeVideo;
use crate::videos::NewSomeVideo2;
use std::cmp::{max, min};

/// avf录像解析器。  
/// - 功能：解析avf格式的录像，有详细分析录像的方法。  
/// - 以下是在python中调用的示例。  
/// ```python
/// import ms_toollib as ms
/// v = ms.AvfVideo("video_name.avf") # 第一步，读取文件的二进制内容
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
///
/// - 在python中被继承的示例。  
/// ```python
/// class MyVideo(ms.AvfVideo):
///     def __new__(cls, *args, **kargs)
///         return ms.AvfVideo.__new__(cls, *args, **kargs):
///     def __init__(self, *args, **kargs):
///         super(MyVideo, self).__init__()
///     def print_something(self):
///         self.parse()
///         self.analyse()
///         self.current_time = 999999
///         print(f"mode: {self.mode}")
///         print(f"level: {self.level}")
///         print(f"time:{self.time}")
///         print(f"bbbv: {self.bbbv}")
///         print(f"cl:{self.cl}")
///         print(f"ce: {self.ce}")
///         print(f"flag: {self.flag}")
/// my_video = MyVideo("jze.avf")
/// my_video.print_something()
/// print(my_video.bbbv_solved)
/// ```
/// - 其他实例化方法  
///
/// ```python
/// import ms_toollib as ms
/// # 使用绝对路径实例化
/// v_1 = ms.AvfVideo(r"F:\SomePath\Beg_NF_3.90_3BV=12_3BVs=3.07_Wang Jianing.avf")
///
/// # 使用二进制列表实例化
/// with open(r"F:\SomePath\Beg_NF_3.90_3BV=12_3BVs=3.07_Wang Jianing.avf", 'rb') as file:
///     video_data_list = list(file.read())
///     # 自定义另一个文件名
///     v_2 = ms.AvfVideo(r"my_file.avf", video_data_list)
///     # 也可以缺省第一个参数`file_name`，则file_name为空字符串
///     v_3 = ms.AvfVideo(raw_data=video_data_list)
/// ```
pub struct AvfVideo {
    pub file_name: String,
    pub data: BaseVideo<Vec<Vec<i32>>>,
}

#[cfg(any(feature = "py", feature = "rs"))]
impl NewSomeVideo<&str> for AvfVideo {
    fn new(file_name: &str) -> Self {
        AvfVideo {
            file_name: file_name.to_string(),
            data: BaseVideo::<Vec<Vec<i32>>>::new(file_name),
        }
    }
}

impl NewSomeVideo2<Vec<u8>, &str> for AvfVideo {
    fn new(raw_data: Vec<u8>, file_name: &str) -> Self {
        AvfVideo {
            file_name: file_name.to_string(),
            data: BaseVideo::<Vec<Vec<i32>>>::new(raw_data),
        }
    }
}

impl AvfVideo {
    pub fn parse(&mut self) -> Result<(), ErrReadVideoReason> {
        // 首位是次版本号，如0.46的录像为46、0.52为52
        let ver = match self.data.get_u8() {
            Ok(v) => v,
            Err(_) => return Err(ErrReadVideoReason::FileIsEmpty),
        };
        // 按源码，该四位全是随机数
        self.data.offset += 4;
        self.data.level = self.data.get_u8()?;
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
        // 算数字
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
        }
        // 0.47之前的版本没有这个校验段，也没有国家字段，直接去找时间戳
        if ver >= 47 {
            // 动态的全局校验过程，不管。目的是读取country。
            let mut t = 0;
            for x in 0..self.data.width {
                for y in 0..self.data.height {
                    let a2res = match self.data.board[y][x] {
                        -1 => 10,
                        e @ _ => e as usize + 1,
                    };
                    t += a2res * (x + x * y);
                }
            }
            let s = t.to_string(); // 将 t 转换为字符串
            let mut s1 = String::new(); // 初始化 s1 为空字符串

            // 遍历 s 中的每个字符
            for c in s.chars() {
                let new_char = (c as u8 - 45) as char; // 将字符的 ASCII 码值减去 45 并转换回字符
                s1.push(new_char); // 将变换后的字符添加到 s1 中
            }
            for _ in 0..s1.to_string().len() + 4 {
                self.data.get_char()?;
            }
            // 中国8，英格兰10， 法国12，美国41
            // print!("{:?}, ", self.data.get_u8()?);
            // https://www.qqxiuzi.cn/zh/region-codes.htm
            self.data.country = match self.data.get_u8()? {
                8 | 37 => "CN".to_string(),
                10 => "GB".to_string(), // 英格兰
                12 => "FR".to_string(),
                13 => "DE".to_string(),
                23 => "KR".to_string(),
                29 => "PL".to_string(),
                31 => "RU".to_string(),
                41 => "US".to_string(),
                69 => "JP".to_string(),
                _ => "XX".to_string(),
            };
        }

        let mut buffer: [char; 5] = ['\0'; 5];

        // 关问号：'\u{6}', '\u{a0}', 'È', '\u{8f}', '¡', '\u{97}', 'Ñ', '\u{7}', '\u{8}', '\u{b}', '\n', '\u{7f}', '9', '[', '3', '|', 'W', '8'
        // 关问号：'\u{6}', '\u{a0}', 'È', '\u{8f}', '¡', '\u{97}', 'Ñ', '\u{7}', '\u{8}', '\u{b}', '\n', '\u{7f}', '<', '[', '3', '|', 'W', '1', '2'
        // 开问号：'\u{6}', '\u{a0}', 'È', '\u{8f}', '¡', '\u{97}', 'Ñ', '\u{7}', '\u{8}', '\u{b}', '\n', '\u{11}', '=', '[', '3', '|', 'W', '1', '7',

        // let mut buf = String::with_capacity(300);
        // for _ in 0..300 {
        //     buf.push(self.data.get_char()?);
        // }
        // print!("{}", buf);

        loop {
            buffer[0] = buffer[1];
            buffer[1] = buffer[2];
            buffer[2] = buffer[3];
            buffer[3] = buffer[4];
            buffer[4] = self.data.get_char()?;
            if buffer[2] == '['
                && (buffer[3] == '0' || buffer[3] == '1' || buffer[3] == '2' || buffer[3] == '3')
                && buffer[4] == '|'
            {
                break;
            }
        }
        // 解析是否开启标问号
        if buffer[0] as u8 == 17 {
            self.data.use_question = true;
        } else if buffer[0] as u8 == 127 {
            self.data.use_question = false;
        } else {
            return Err(ErrReadVideoReason::InvalidParams);
        }

        if self.data.level == 6 {
            loop {
                if self.data.get_char()? == '|' {
                    break;
                }
            }
        }
        // avf中的时间戳没有时区，最大可能有12小时的偏差
        let mut start_time = String::new();
        loop {
            match self.data.get_char()? {
                '|' => break,
                other => start_time.push(other),
            }
        }
        self.data.start_time = self.data.parse_avf_start_timestamp(&start_time)?;
        // 时间戳部分正常情况举例：
        // 初级：[0|26.10.2022.23:13:27:2236|26.23:13:29:7764|B7T3.52]
        // 高级破纪录时：[2|18.10.2022.20:15:35:6606|18.20:16:24:8868|HS|B127T50.25]
        // 自定义：[3|W8H11M7|3.9.2025.17:00:08:6660|3.17:00:14:081|B8T6.42]
        // 异常情况举例：
        // 高级：[2|17.7.2012.12:08:03:3338|17.12:09:44:6697B248T102.34]

        // Wang_Jia_Ning_Beg_15.523bv24.avf这局录像证明，avf的T后面的rtime并不一定正确(下称T后值)。这局录像的此数值是3.61，而实际rtime为14.52
        // [0|24.8.2024.16:38:59:6687|24.16:39:14:2213|B24T3.61]
        let mut end_time = String::new();
        let mut buffer: [char; 2];
        loop {
            match self.data.get_char()? {
                '|' => {
                    buffer = ['\0', '|'];
                    break;
                }
                'B' => {
                    buffer = ['|', 'B'];
                    break;
                }
                other => end_time.push(other),
            }
        }
        self.data.end_time = self.data.parse_avf_end_timestamp(&start_time, &end_time)?;

        loop {
            if buffer[0] == '|' && buffer[1] == 'B' {
                break;
            }
            buffer[0] = buffer[1];
            buffer[1] = self.data.get_char()?;
        }
        // 0.46及更早的版本此处是“3BV/已解3BV”，例如“154/154”，只取前一个
        let s = self.data.get_utf8_c_string('T')?;
        let bbbv_str = s.split('/').next().unwrap_or("").trim().to_string();
        self.data.static_params.bbbv = match bbbv_str.parse() {
            Ok(v) => v,
            Err(_) => return Err(ErrReadVideoReason::InvalidParams),
        };
        let mut s = self.data.get_utf8_c_string(']')?;
        s = str::replace(&s, ",", "."); // 有些录像小数点是逗号
        let _time_from_header = match s.parse::<f64>() {
            Ok(v) => v - 1.0,
            Err(_) => return Err(ErrReadVideoReason::InvalidParams),
        };
        let mut buffer = [0u8; 8];
        if ver < 47 {
            // 老版本事件区前的填充字节数不固定（常见5个或6个），
            // 用“整段事件流的鼠标编码是否合法”来选出正确的对齐位置
            let start = self.data.offset;
            let mut best_skip = 0usize;
            let mut best_count = 0usize;
            let mut best_ended = false;
            {
                let raw = self.data.raw_data();
                for skip in 0..=24usize {
                    let mut pos = start + skip;
                    let mut count = 0usize;
                    let mut ended = false;
                    while pos + 8 <= raw.len() {
                        let b = &raw[pos..pos + 8];
                        if b[6] == 0 && b[2] == 0 {
                            // 终止事件
                            ended = true;
                            break;
                        }
                        if !matches!(b[0], 1 | 3 | 5 | 9 | 11 | 17 | 21 | 33 | 65 | 145 | 193) {
                            break;
                        }
                        count += 1;
                        pos += 8;
                    }
                    if count > best_count || (count == best_count && ended && !best_ended) {
                        best_count = count;
                        best_ended = ended;
                        best_skip = skip;
                    }
                }
            }
            self.data.offset = start + best_skip;
            for i in 0..8 {
                buffer[i] = self.data.get_u8()?;
            }
        } else {
            while buffer[2] != 1 || buffer[1] > 1 {
                buffer[0] = buffer[1];
                buffer[1] = buffer[2];
                buffer[2] = self.data.get_u8()?;
            }
            for i in 3..8 {
                buffer[i] = self.data.get_u8()?;
            }
        }
        loop {
            self.data
                .video_action_state_recorder
                .push(VideoActionStateRecorder {
                    time: ((buffer[6] as u16) << 8 | buffer[2] as u16) as f64 - 1.0
                        + (buffer[4] as f64) / 100.0,
                    event: Some(Event::Mouse(MouseEvent {
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
                            11 => panic!(),
                            // left_click_with_shift没见过，不清楚用途
                            // 11 => "sc".to_string(),
                            21 => "lr".to_string(),
                            _ => return Err(ErrReadVideoReason::InvalidVideoEvent),
                        },
                        x: (buffer[1] as u16) << 8 | buffer[3] as u16,
                        y: (buffer[5] as u16) << 8 | buffer[7] as u16,
                    })),
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
        // 一共计算了两种rtime，T后值和事件值。理论上两者应该相等，实际上，T后值有极小可能不准，Wang_Jia_Ning_Beg_15.523bv24.avf证明了了这一点。事件值计算复杂，因为不能简单拿最后一次事件的时间作为rtime，avf录像有可能不以lc开头，这一点只能凭借印象，暂时无法举例说明。
        if let Some(last) = self.data.video_action_state_recorder.last() {
            if _time_from_header + 1e-6 < last.time {
                let last_time = last.time;
                let mut lr_time = 0.0;
                for record in self.data.video_action_state_recorder.iter_mut().rev() {
                    if let Some(Event::Mouse(mouse_event)) = &record.event {
                        if mouse_event.mouse == "lr" {
                            lr_time = record.time;
                        }
                    }
                }
                self.data.set_rtime(last_time - lr_time).unwrap();
            } else {
                self.data.set_rtime(_time_from_header).unwrap();
            }
        }
        if ver >= 47 {
            // 标识符
            while self.data.get_char()? != 'S' {}
            while self.data.get_char()? != 'k' {}
            while self.data.get_char()? != 'i' {}
            while self.data.get_char()? != 'n' {}
            while self.data.get_char()? != ':' {}
            while self.data.get_char()? != '\r' {}
            self.data.player_identifier = self.data.get_unknown_encoding_c_string('\r')?;
        } else {
            // 0.47之前的录像尾部没有Realtime、Skin字段，结构为“cs=”+校验和+“\r”+玩家名+“\r”
            let mut buffer = [0u8; 3];
            while !(buffer[0] == b'c' && buffer[1] == b's' && buffer[2] == b'=') {
                buffer[0] = buffer[1];
                buffer[1] = buffer[2];
                buffer[2] = self.data.get_u8()?;
            }
            // 跳过“cs=”后的16字节校验和和换行，再读玩家名
            for _ in 0..17 {
                self.data.get_u8()?;
            }
            self.data.player_identifier = self.data.get_unknown_encoding_c_string('\r')?;
        }
        // 软件名+版本号。这里枚举Arbiter所有合法版本号（与录像包中Program串的取值一一对应）
        let version = match self.program_version().as_str() {
            "0.46.2" => "0.46.2",
            "0.48.5" => "0.48.5",
            "0.49.3" => "0.49.3",
            "0.49.4" => "0.49.4",
            "0.49.6" => "0.49.6",
            "0.50.1" => "0.50.1",
            "0.52" => "0.52",
            "0.52.2" => "0.52.2",
            "0.52.3" => "0.52.3",
            // 版本未登记：直接报错，防止假冒录像
            _ => return Err(ErrReadVideoReason::InvalidParams),
        };
        self.data.software = format!("Arbiter {}", version);
        self.data.can_analyse = true;
        Ok(())
    }

    /// 按固定位置读取尾部Program串中的版本号。格式严格为“Minesweeper Arbiter X.Y[.Z]. Copyright”，
    /// 位置或格式不符直接panic，不做任何回退（安全模型要求）。
    fn program_version(&self) -> String {
        let raw = self.data.raw_data();
        let tail = &raw[self.data.offset..];
        let tail = tail
            .strip_prefix(&b"Minesweeper Arbiter "[..])
            .expect("Program串不是以“Minesweeper Arbiter ”开头");
        let mut end = 0;
        while end < tail.len() && (tail[end].is_ascii_digit() || tail[end] == b'.') {
            end += 1;
        }
        let token = std::str::from_utf8(&tail[..end]).expect("版本号不是有效的UTF-8");
        assert!(
            token.ends_with('.') && tail[end..].starts_with(b" Copyright"),
            "版本号格式错误: {token}"
        );
        token.trim_end_matches('.').to_string()
    }
}
