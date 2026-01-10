use crate::miscellaneous::s_to_ms;
use crate::videos::{BaseVideo, Event};

use crate::safe_board::BoardSize;
use std::cmp::min;
use std::path::Path;
use std::thread;

#[cfg(any(feature = "py", feature = "rs"))]
use std::fs;
// BaseVideo按照evf标准，生成evf录像文件的方法

// 和文件操作相关的一些方法
#[cfg(any(feature = "py", feature = "rs"))]
impl<T> BaseVideo<T> {
    /// 按evf v0.0-0.1标准，编码出原始二进制数据
    pub fn generate_evf_v0_raw_data(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        self.raw_data = vec![0, 0];
        if self.is_completed {
            self.raw_data[1] |= 0b1000_0000;
        }
        if self.is_official {
            self.raw_data[1] |= 0b0100_0000;
        }
        if self.is_fair {
            self.raw_data[1] |= 0b0010_0000;
        }
        self.raw_data.push(self.height as u8);
        self.raw_data.push(self.width as u8);
        self.raw_data.push((self.mine_num >> 8).try_into().unwrap());
        self.raw_data
            .push((self.mine_num % 256).try_into().unwrap());
        self.raw_data.push(self.cell_pixel_size);
        self.raw_data.push((self.mode >> 8).try_into().unwrap());
        self.raw_data.push((self.mode % 256).try_into().unwrap());
        self.raw_data
            .push((self.static_params.bbbv >> 8).try_into().unwrap());
        self.raw_data
            .push((self.static_params.bbbv % 256).try_into().unwrap());
        // println!("fff: {:?}", self.game_dynamic_params.rtime_ms);
        self.raw_data.push(
            (self.game_dynamic_params.rtime_ms >> 16)
                .try_into()
                .unwrap(),
        );
        self.raw_data.push(
            ((self.game_dynamic_params.rtime_ms >> 8) % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data.push(
            (self.game_dynamic_params.rtime_ms % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data
            .append(&mut self.software.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.player_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.race_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.uniqueness_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.start_time.to_string().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.end_time.to_string().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.country.to_string().into_bytes());
        self.raw_data.push(0);

        let mut byte = 0;
        let mut ptr = 0;
        for i in 0..self.height {
            for j in 0..self.width {
                byte <<= 1;
                if self.board[i][j] == -1 {
                    byte |= 1;
                }
                ptr += 1;
                if ptr == 8 {
                    self.raw_data.push(byte);
                    ptr = 0;
                    byte = 0;
                }
            }
        }
        if ptr > 0 {
            byte <<= 8 - ptr;
            self.raw_data.push(byte);
        }

        for vas in &self.video_action_state_recorder {
            if let Some(Event::Mouse(mouse_event)) = &vas.event {
                // println!("{:?}: '{:?}', ({:?}, {:?})", event.time, event.mouse.as_str(), event.x, event.y);
                match mouse_event.mouse.as_str() {
                    "mv" => self.raw_data.push(1),
                    "lc" => self.raw_data.push(2),
                    "lr" => self.raw_data.push(3),
                    "rc" => self.raw_data.push(4),
                    "rr" => self.raw_data.push(5),
                    "mc" => self.raw_data.push(6),
                    "mr" => self.raw_data.push(7),
                    "pf" => self.raw_data.push(8),
                    "cc" => self.raw_data.push(9),
                    // 不可能出现，出现再说
                    _ => self.raw_data.push(99),
                }
                let t_ms = s_to_ms(vas.time);
                self.raw_data.push((t_ms >> 16).try_into().unwrap());
                self.raw_data.push(((t_ms >> 8) % 256).try_into().unwrap());
                self.raw_data.push((t_ms % 256).try_into().unwrap());
                self.raw_data.push((mouse_event.x >> 8).try_into().unwrap());
                self.raw_data
                    .push((mouse_event.x % 256).try_into().unwrap());
                self.raw_data.push((mouse_event.y >> 8).try_into().unwrap());
                self.raw_data
                    .push((mouse_event.y % 256).try_into().unwrap());
            }
        }
        if !self.checksum.is_empty() {
            self.raw_data.push(0);
            self.raw_data
                .append(&mut self.checksum.clone().to_vec().to_owned());
        } else {
            self.raw_data.push(255);
        }
    }
    /// 按evf v0.2标准，编码出原始二进制数据
    pub fn generate_evf_v2_raw_data(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        self.raw_data = vec![0, 0];
        if self.is_completed {
            self.raw_data[1] |= 0b1000_0000;
        }
        if self.is_official {
            self.raw_data[1] |= 0b0100_0000;
        }
        if self.is_fair {
            self.raw_data[1] |= 0b0010_0000;
        }
        self.raw_data.push(self.height as u8);
        self.raw_data.push(self.width as u8);
        self.raw_data.push((self.mine_num >> 8).try_into().unwrap());
        self.raw_data
            .push((self.mine_num % 256).try_into().unwrap());
        self.raw_data.push(self.cell_pixel_size);
        self.raw_data.push((self.mode >> 8).try_into().unwrap());
        self.raw_data.push((self.mode % 256).try_into().unwrap());
        self.raw_data
            .push((self.static_params.bbbv >> 8).try_into().unwrap());
        self.raw_data
            .push((self.static_params.bbbv % 256).try_into().unwrap());
        // println!("fff: {:?}", self.game_dynamic_params.rtime_ms);
        self.raw_data.push(
            (self.game_dynamic_params.rtime_ms >> 16)
                .try_into()
                .unwrap(),
        );
        self.raw_data.push(
            ((self.game_dynamic_params.rtime_ms >> 8) % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data.push(
            (self.game_dynamic_params.rtime_ms % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data
            .append(&mut self.software.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.player_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.race_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.uniqueness_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.start_time.to_string().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.end_time.to_string().into_bytes());
        self.raw_data.push(0);
        self.raw_data.append(&mut self.country.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.device_uuid.clone().to_owned());
        self.raw_data.push(0);

        let mut byte = 0;
        let mut ptr = 0;
        for i in 0..self.height {
            for j in 0..self.width {
                byte <<= 1;
                if self.board[i][j] == -1 {
                    byte |= 1;
                }
                ptr += 1;
                if ptr == 8 {
                    self.raw_data.push(byte);
                    ptr = 0;
                    byte = 0;
                }
            }
        }
        if ptr > 0 {
            byte <<= 8 - ptr;
            self.raw_data.push(byte);
        }

        for vas in &self.video_action_state_recorder {
            if let Some(Event::Mouse(mouse_event)) = &vas.event {
                // println!("{:?}: '{:?}', ({:?}, {:?})", event.time, event.mouse.as_str(), event.x, event.y);
                match mouse_event.mouse.as_str() {
                    "mv" => self.raw_data.push(1),
                    "lc" => self.raw_data.push(2),
                    "lr" => self.raw_data.push(3),
                    "rc" => self.raw_data.push(4),
                    "rr" => self.raw_data.push(5),
                    "mc" => self.raw_data.push(6),
                    "mr" => self.raw_data.push(7),
                    "pf" => self.raw_data.push(8),
                    "cc" => self.raw_data.push(9),
                    // 不可能出现，出现再说
                    _ => self.raw_data.push(99),
                }
                let t_ms = s_to_ms(vas.time);
                self.raw_data.push((t_ms >> 16).try_into().unwrap());
                self.raw_data.push(((t_ms >> 8) % 256).try_into().unwrap());
                self.raw_data.push((t_ms % 256).try_into().unwrap());
                self.raw_data.push((mouse_event.x >> 8).try_into().unwrap());
                self.raw_data
                    .push((mouse_event.x % 256).try_into().unwrap());
                self.raw_data.push((mouse_event.y >> 8).try_into().unwrap());
                self.raw_data
                    .push((mouse_event.y % 256).try_into().unwrap());
            }
        }
        if !self.checksum.is_empty() {
            self.raw_data.push(0);
            self.raw_data
                .append(&mut self.checksum.clone().to_vec().to_owned());
        } else {
            self.raw_data.push(255);
        }
    }
    /// 按evf v0.3标准，编码出原始二进制数据
    pub fn generate_evf_v3_raw_data(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        self.raw_data = vec![3, 0, 0];
        if self.is_completed {
            self.raw_data[1] |= 0b1000_0000;
        }
        if self.is_official {
            self.raw_data[1] |= 0b0100_0000;
        }
        if self.is_fair {
            self.raw_data[1] |= 0b0010_0000;
        }
        if self.get_right() == 0 {
            self.raw_data[1] |= 0b0001_0000;
        }
        if self.use_question {
            self.raw_data[2] |= 0b1000_0000;
        }
        if self.use_cursor_pos_lim {
            self.raw_data[2] |= 0b0100_0000;
        }
        if self.use_auto_replay {
            self.raw_data[2] |= 0b0010_0000;
        }
        self.raw_data.push(self.height as u8);
        self.raw_data.push(self.width as u8);
        self.raw_data.push((self.mine_num >> 8).try_into().unwrap());
        self.raw_data
            .push((self.mine_num % 256).try_into().unwrap());
        self.raw_data.push(self.cell_pixel_size);
        self.raw_data.push((self.mode >> 8).try_into().unwrap());
        self.raw_data.push((self.mode % 256).try_into().unwrap());
        self.raw_data
            .push((self.static_params.bbbv >> 8).try_into().unwrap());
        self.raw_data
            .push((self.static_params.bbbv % 256).try_into().unwrap());
        // println!("fff: {:?}", self.game_dynamic_params.rtime_ms);
        self.raw_data.push(
            (self.game_dynamic_params.rtime_ms >> 16)
                .try_into()
                .unwrap(),
        );
        self.raw_data.push(
            ((self.game_dynamic_params.rtime_ms >> 8) % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data.push(
            (self.game_dynamic_params.rtime_ms % 256)
                .try_into()
                .unwrap(),
        );
        self.raw_data
            .append(&mut self.software.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.player_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.race_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.uniqueness_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.start_time.to_string().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.end_time.to_string().into_bytes());
        self.raw_data.push(0);
        self.raw_data.append(&mut self.country.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.device_uuid.clone().to_owned());
        self.raw_data.push(0);

        let mut byte = 0;
        let mut ptr = 0;
        for i in 0..self.height {
            for j in 0..self.width {
                byte <<= 1;
                if self.board[i][j] == -1 {
                    byte |= 1;
                }
                ptr += 1;
                if ptr == 8 {
                    self.raw_data.push(byte);
                    ptr = 0;
                    byte = 0;
                }
            }
        }
        if ptr > 0 {
            byte <<= 8 - ptr;
            self.raw_data.push(byte);
        }

        for vas in &self.video_action_state_recorder {
            if let Some(Event::Mouse(mouse_event)) = &vas.event {
                // println!("{:?}: '{:?}', ({:?}, {:?})", event.time, event.mouse.as_str(), event.x, event.y);
                match mouse_event.mouse.as_str() {
                    "mv" => self.raw_data.push(1),
                    "lc" => self.raw_data.push(2),
                    "lr" => self.raw_data.push(3),
                    "rc" => self.raw_data.push(4),
                    "rr" => self.raw_data.push(5),
                    "mc" => self.raw_data.push(6),
                    "mr" => self.raw_data.push(7),
                    "pf" => self.raw_data.push(8),
                    "cc" => self.raw_data.push(9),
                    // 不可能出现，出现再说
                    _ => self.raw_data.push(99),
                }
                let t_ms = s_to_ms(vas.time);
                self.raw_data.push((t_ms >> 16).try_into().unwrap());
                self.raw_data.push(((t_ms >> 8) % 256).try_into().unwrap());
                self.raw_data.push((t_ms % 256).try_into().unwrap());
                self.raw_data.push((mouse_event.x >> 8).try_into().unwrap());
                self.raw_data
                    .push((mouse_event.x % 256).try_into().unwrap());
                self.raw_data.push((mouse_event.y >> 8).try_into().unwrap());
                self.raw_data
                    .push((mouse_event.y % 256).try_into().unwrap());
            }
        }
        if !self.checksum.is_empty() {
            self.raw_data.push(0);
            self.raw_data
                .append(&mut self.checksum.clone().to_vec().to_owned());
        } else {
            self.raw_data.push(255);
        }
    }

    /// 按evf v4标准，编码出原始二进制数据
    /// v4开始，判断nf的标准发生了变化！
    pub fn generate_evf_v4_raw_data(&mut self)
    where
        T: std::ops::Index<usize> + BoardSize,
        T::Output: std::ops::Index<usize, Output = i32>,
    {
        assert!(self.height <= 255);
        assert!(self.width <= 255);
        assert!(self.height * self.cell_pixel_size as usize <= 32767);
        assert!(self.width * self.cell_pixel_size as usize <= 32767);
        assert!(self.mine_num <= 65535);
        self.raw_data = vec![4, 0, 0];
        if self.is_completed {
            self.raw_data[1] |= 0b1000_0000;
        }
        if self.is_official {
            self.raw_data[1] |= 0b0100_0000;
        }
        if self.is_fair {
            self.raw_data[1] |= 0b0010_0000;
        }
        if self.get_rce().unwrap() == 0 {
            self.raw_data[1] |= 0b0001_0000;
        }
        if self.translated {
            self.raw_data[1] |= 0b0000_1000;
        }
        if self.use_question {
            self.raw_data[2] |= 0b1000_0000;
        }
        if self.use_cursor_pos_lim {
            self.raw_data[2] |= 0b0100_0000;
        }
        if self.use_auto_replay {
            self.raw_data[2] |= 0b0010_0000;
        }
        self.raw_data.push(self.height as u8);
        self.raw_data.push(self.width as u8);
        self.raw_data.push((self.mine_num >> 8).try_into().unwrap());
        self.raw_data
            .push((self.mine_num % 256).try_into().unwrap());
        self.raw_data.push(self.cell_pixel_size);
        self.raw_data.push((self.mode >> 8).try_into().unwrap());
        self.raw_data.push((self.mode % 256).try_into().unwrap());
        self.raw_data
            .push((self.static_params.bbbv >> 8).try_into().unwrap());
        self.raw_data
            .push((self.static_params.bbbv % 256).try_into().unwrap());
        self.raw_data
            .extend_from_slice(&self.game_dynamic_params.rtime_ms.to_be_bytes());
        if self.country.len() != 2 {
            self.raw_data.extend("XX".as_bytes());
        } else {
            let first_char = self.country.chars().nth(0).unwrap();
            let second_char = self.country.chars().nth(1).unwrap();
            if first_char.is_ascii_alphabetic() && second_char.is_ascii_alphabetic() {
                self.raw_data.push(first_char.to_ascii_uppercase() as u8);
                self.raw_data.push(second_char.to_ascii_uppercase() as u8);
            } else {
                self.raw_data.extend("XX".as_bytes());
            }
        }
        self.raw_data
            .extend_from_slice(&self.start_time.to_be_bytes());
        self.raw_data
            .extend_from_slice(&self.end_time.to_be_bytes());
        self.raw_data
            .append(&mut self.software.clone().into_bytes());
        self.raw_data.push(0);
        if self.translated {
            self.raw_data
                .append(&mut self.translate_software.clone().into_bytes());
            self.raw_data.push(0);
            self.raw_data
                .append(&mut self.original_encoding.clone().into_bytes());
            self.raw_data.push(0);
        }
        self.raw_data
            .append(&mut self.player_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.race_identifier.clone().into_bytes());
        self.raw_data.push(0);
        self.raw_data
            .append(&mut self.uniqueness_identifier.clone().into_bytes());
        self.raw_data.push(0);
        let device_uuid_length = self.device_uuid.len() as u16;
        self.raw_data
            .extend_from_slice(&device_uuid_length.to_be_bytes());
        self.raw_data
            .append(&mut self.device_uuid.clone().to_owned());
        let mut byte = 0;
        let mut ptr = 0;
        for i in 0..self.height {
            for j in 0..self.width {
                byte <<= 1;
                if self.board[i][j] == -1 {
                    byte |= 1;
                }
                ptr += 1;
                if ptr == 8 {
                    self.raw_data.push(byte);
                    ptr = 0;
                    byte = 0;
                }
            }
        }
        if ptr > 0 {
            byte <<= 8 - ptr;
            self.raw_data.push(byte);
        }
        // 自定义指标的数量
        self.raw_data.push(0);
        self.raw_data.push(0);
        let vas_0 = &self.video_action_state_recorder[0];
        // 计算鼠标坐标差值使用
        let mut last_mouse_event;
        let mut last_mouse_event_time;
        if let Some(Event::Mouse(event_0)) = &vas_0.event {
            last_mouse_event = event_0;
            last_mouse_event_time = vas_0.time;
            match event_0.mouse.as_str() {
                "mv" => self.raw_data.push(1),
                "lc" => self.raw_data.push(2),
                "rc" => self.raw_data.push(4),
                "pf" => self.raw_data.push(8),
                // 不可能出现，出现再说
                _ => panic!(""),
            }
            let t_ms = s_to_ms(vas_0.time) as u8;
            self.raw_data.push((t_ms).try_into().unwrap());
            self.raw_data.push((event_0.x >> 8).try_into().unwrap());
            self.raw_data.push((event_0.x % 256).try_into().unwrap());
            self.raw_data.push((event_0.y >> 8).try_into().unwrap());
            self.raw_data.push((event_0.y % 256).try_into().unwrap());
        } else {
            panic!("");
        }

        for event_id in 1..self.video_action_state_recorder.len() {
            let vas = &self.video_action_state_recorder[event_id];
            if let Some(Event::Mouse(mouse_event)) = &vas.event {
                // println!("{:?}: '{:?}', ({:?}, {:?})", event.time, event.mouse.as_str(), event.x, event.y);
                let mut delta_t = s_to_ms(vas.time) - s_to_ms(last_mouse_event_time);
                while delta_t > 255 {
                    self.raw_data.push(255);
                    let pause_time = min(65535 as u32, delta_t) as u16;
                    self.raw_data.extend_from_slice(&pause_time.to_be_bytes());
                    delta_t -= pause_time as u32;
                }
                match mouse_event.mouse.as_str() {
                    "mv" => self.raw_data.push(1),
                    "lc" => self.raw_data.push(2),
                    "lr" => self.raw_data.push(3),
                    "rc" => self.raw_data.push(4),
                    "rr" => self.raw_data.push(5),
                    "mc" => self.raw_data.push(6),
                    "mr" => self.raw_data.push(7),
                    "pf" => self.raw_data.push(8),
                    "cc" => self.raw_data.push(9),
                    "l" => self.raw_data.push(10),
                    "r" => self.raw_data.push(11),
                    "m" => self.raw_data.push(12),
                    // 不可能出现，出现再说
                    _ => {
                        continue;
                    }
                }
                self.raw_data.push(delta_t as u8);
                let delta_x = mouse_event.x as i16 - last_mouse_event.x as i16;
                let delta_y = mouse_event.y as i16 - last_mouse_event.y as i16;
                self.raw_data.extend_from_slice(&delta_x.to_be_bytes());
                self.raw_data.extend_from_slice(&delta_y.to_be_bytes());
                last_mouse_event = mouse_event;
                last_mouse_event_time = vas.time;
            }
        }
        self.raw_data.push(0);
        self.raw_data
            .extend_from_slice(&(self.checksum.len() as u16).to_be_bytes());
        self.raw_data
            .append(&mut self.checksum.clone().to_vec().to_owned());
    }
    // /// 在二进制数据最后添加checksum。通过generate_evf_v0_raw_data或push_checksum添加checksum二选一。
    // /// 若无checksum就用generate_evf_v0_raw_data
    // pub fn push_checksum(&mut self, checksum: &mut Vec<u8>) {
    //     *self.raw_data.last_mut().unwrap() = 0;
    //     self.raw_data.append(checksum);
    // }
    /// 存evf文件，自动加后缀，xxx.evf重复变成xxx(2).evf。后台线程执行，即使对象销毁也不影响。
    pub fn save_to_evf_file(&self, file_name: &str) -> String {
        if self.raw_data.is_empty() {
            panic!(
                "Raw data is empty. Please generate raw data by `generate_evf_v4_raw_data` first."
            );
        }

        let base = file_name.to_string();
        let data = self.raw_data.clone();

        // 先计算文件名（同步，极快）
        let final_name = {
            let first = format!("{}.evf", base);
            if !Path::new(&first).exists() {
                first
            } else {
                let mut id = 2;
                loop {
                    let name = format!("{}({}).evf", base, id);
                    if !Path::new(&name).exists() {
                        break name;
                    }
                    id += 1;
                }
            }
        };

        // 后台线程写入文件
        let write_name = final_name.clone();
        thread::spawn(move || {
            if let Err(e) = fs::write(&write_name, data) {
                eprintln!("Failed to write evf file: {}", e);
            }
        });

        // 立刻返回，不阻塞
        final_name
    }
}
