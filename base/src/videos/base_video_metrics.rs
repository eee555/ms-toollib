use crate::miscellaneous::s_to_ms;
use crate::videos::{BaseVideo, Event};
use crate::{GameBoardState, MouseState};
#[cfg(any(feature = "py", feature = "rs"))]
use std::time::Instant;
// BaseVideo指标计算和获取的方法

impl<T> BaseVideo<T> {
    // 再实现一些get、set方法
    pub fn set_pix_size(&mut self, pix_size: u8) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Ready
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Loss
        {
            return Err(());
        }
        self.cell_pixel_size = pix_size;
        Ok(0)
    }
    /// 获取当前录像时刻的后验的游戏局面
    pub fn get_game_board(&self) -> Vec<Vec<i32>> {
        if self.game_board_state == GameBoardState::Display {
            return self.video_action_state_recorder[self.current_event_id]
                .next_game_board
                .as_ref()
                .unwrap()
                .borrow()
                .game_board
                .clone();
        } else {
            return self.minesweeper_board.game_board.clone();
        }
    }
    /// 获取当前录像时刻的局面概率
    pub fn get_game_board_poss(&mut self) -> Vec<Vec<f64>> {
        let mut id = self.current_event_id;
        loop {
            if self.video_action_state_recorder[id].useful_level < 2 {
                if id >= 1 {
                    id -= 1;
                }
                if id == 0 {
                    let p = self.mine_num as f64 / (self.height * self.width) as f64;
                    return vec![vec![p; self.height]; self.width];
                }
            } else {
                // println!("{:?}, {:?}",self.current_event_id, self.video_action_state_recorder.len());
                return self.video_action_state_recorder[self.current_event_id]
                    .next_game_board
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .get_poss()
                    .clone();
            }
        }
    }
    // 录像解析时，设置游戏时间，时间成绩。
    // 同时设置秒和毫秒的时间，并且只能写入一次
    pub fn set_rtime<U>(&mut self, time: U) -> Result<u8, ()>
    where
        U: Into<f64>,
    {
        if !self.allow_set_rtime {
            return Err(());
        }
        let time = time.into();
        self.game_dynamic_params.rtime = time;
        self.game_dynamic_params.rtime_ms = s_to_ms(time);
        self.allow_set_rtime = false;
        Ok(0)
    }
    /// 用于(游戏时)计数器上显示的时间,和arbiter一致
    #[cfg(any(feature = "py", feature = "rs"))]
    pub fn get_time(&self) -> f64 {
        match self.game_board_state {
            GameBoardState::Playing => {
                let now = Instant::now();
                // return now.duration_since(self.game_start_instant).as_millis() as f64 / 1000.0;
                let time_ms = now.duration_since(self.video_start_instant).as_millis() as u32;
                return (time_ms - self.game_start_ms) as f64 / 1000.0;
            }
            GameBoardState::PreFlaging => {
                let now = Instant::now();
                return now.duration_since(self.video_start_instant).as_millis() as f64 / 1000.0;
            }
            GameBoardState::Loss | GameBoardState::Win => self.game_dynamic_params.rtime,
            GameBoardState::Ready => 0.0,
            GameBoardState::Display => self.current_time,
        }
    }
    pub fn get_rtime(&self) -> Result<f64, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Display
        {
            return Err(());
        }
        Ok(self.game_dynamic_params.rtime)
    }
    pub fn get_rtime_ms(&self) -> Result<u32, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Display
        {
            return Err(());
        }
        Ok(self.game_dynamic_params.rtime_ms)
    }
    /// 录像播放器时间的开始值
    /// 理论上：video_start_time = -self.delta_time
    pub fn get_video_start_time(&self) -> Result<f64, ()> {
        if self.game_board_state != GameBoardState::Display {
            return Err(());
        }
        Ok(-self.delta_time)
    }
    /// 录像播放器时间的结束值
    /// 理论上：video_end_time = rtime
    pub fn get_video_end_time(&self) -> Result<f64, ()> {
        if self.game_board_state != GameBoardState::Display {
            return Err(());
        }
        // end_time的计算方法是特殊的，直接返回rtime，而不是用减法
        // 因为此处减法会带来浮点数误差
        // Ok(self.video_action_state_recorder.last().unwrap().time - self.delta_time)
        Ok(self.game_dynamic_params.rtime)
    }
    /// 录像播放时，按时间设置current_time；超出两端范围取两端。
    /// 游戏时不要调用。
    pub fn set_current_time(&mut self, mut time: f64) {
        self.current_time = time;
        if self.current_time < self.get_video_start_time().unwrap() {
            self.current_time = self.get_video_start_time().unwrap()
        }
        if self.current_time > self.get_video_end_time().unwrap() {
            self.current_time = self.get_video_end_time().unwrap()
        }
        time += self.delta_time;
        if time > self.video_action_state_recorder[self.current_event_id].time {
            loop {
                if self.current_event_id >= self.video_action_state_recorder.len() - 1 {
                    // 最后一帧
                    break;
                }
                self.current_event_id += 1;
                if self.video_action_state_recorder[self.current_event_id].time <= time {
                    continue;
                } else {
                    self.current_event_id -= 1;
                    break;
                }
            }
        } else {
            loop {
                if self.current_event_id == 0 {
                    break;
                }
                self.current_event_id -= 1;
                if self.video_action_state_recorder[self.current_event_id].time > time {
                    continue;
                } else {
                    break;
                }
            }
        }
        // self.current_time =
        //     self.video_action_state_recorder[self.current_event_id].time - self.delta_time;
    }
    /// 设置current_event_id
    pub fn set_current_event_id(&mut self, id: usize) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Display {
            return Err(());
        };
        self.current_event_id = id;
        self.current_time = self.video_action_state_recorder[id].time - self.delta_time;
        Ok(0)
    }
    pub fn set_use_question(&mut self, use_question: bool) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.use_question = use_question;
        Ok(0)
    }
    pub fn set_use_cursor_pos_lim(&mut self, use_cursor_pos_lim: bool) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.use_cursor_pos_lim = use_cursor_pos_lim;
        Ok(0)
    }
    pub fn set_use_auto_replay(&mut self, use_auto_replay: bool) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.use_auto_replay = use_auto_replay;
        Ok(0)
    }
    pub fn set_is_official(&mut self, is_official: bool) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.is_official = is_official;
        Ok(0)
    }
    pub fn set_is_fair(&mut self, is_fair: bool) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.is_fair = is_fair;
        Ok(0)
    }
    /// 可猜模式必须在ready时设置模式，其它模式扫完再设置也可以
    pub fn set_mode(&mut self, mode: u16) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Ready
        {
            return Err(());
        };
        self.mode = mode;
        Ok(0)
    }
    pub fn set_software(&mut self, software: String) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Ready
        {
            return Err(());
        };
        self.software = software;
        Ok(0)
    }

    pub fn set_player_identifier(&mut self, player_identifier: String) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.player_identifier = player_identifier;
        Ok(0)
    }
    pub fn set_race_identifier(&mut self, race_identifier: String) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.race_identifier = race_identifier;
        Ok(0)
    }
    pub fn set_uniqueness_identifier(&mut self, uniqueness_identifier: String) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.uniqueness_identifier = uniqueness_identifier;
        Ok(0)
    }
    /// 拟弃用，会自动记录
    // pub fn set_start_time(&mut self, start_time: Vec<u8>) -> Result<u8, ()> {
    //     if self.game_board_state != GameBoardState::Loss
    //         && self.game_board_state != GameBoardState::Win
    //     {
    //         return Err(());
    //     };
    //     self.start_time = start_time;
    //     Ok(0)
    // }
    /// 拟弃用，会自动记录
    // pub fn set_end_time(&mut self, end_time: Vec<u8>) -> Result<u8, ()> {
    //     if self.game_board_state != GameBoardState::Loss
    //         && self.game_board_state != GameBoardState::Win
    //     {
    //         return Err(());
    //     };
    //     self.end_time = end_time;
    //     Ok(0)
    // }
    pub fn set_country(&mut self, country: String) -> Result<u8, ()> {
        // 读取录像文件后，有可能要转存。此时，例如rmv，录像中的国家信息是用户手动输入的
        // 需要在外部传入。因为所有国家的文本信息有39k，放在工具箱里不合适
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Display
        {
            return Err(());
        };
        self.country = country;
        Ok(0)
    }
    pub fn set_device_uuid(&mut self, device_uuid: Vec<u8>) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Loss
        {
            return Err(());
        }
        self.device_uuid = device_uuid;
        Ok(0)
    }
    // 根据raw_data的版本号，自动调用不同的checksum设置方法
    pub fn set_checksum(&mut self, checksum: Vec<u8>) -> Result<u8, ()> {
        match self.raw_data.get(0).unwrap() {
            0..=3 => {
                return self.set_checksum_evf_v3(checksum);
            }
            4 => {
                return self.set_checksum_evf_v4(checksum);
            }
            _ => {
                return Err(());
            }
        }
    }
    /// 在生成二进制数据后，在raw_data里添加checksum
    /// 按照evf0-3的标准添加，即删除末尾的/255，添加/0、32位checksum
    fn set_checksum_evf_v3(&mut self, checksum: Vec<u8>) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        if self.checksum.is_empty() {
            *self.raw_data.last_mut().unwrap() = 0;
            self.raw_data
                .append(&mut checksum.clone().to_vec().to_owned());
            self.checksum = checksum;
            // self.has_checksum = true;
            return Ok(0);
        } else {
            let ptr = self.raw_data.len() - 32;
            for i in 0..32 {
                self.raw_data[ptr + i] = checksum[i];
            }
            return Ok(0);
        }
    }
    /// 在生成二进制数据后，在raw_data里添加checksum
    /// 按照evf4的标准添加，即添加u16的长度、若干位checksum
    fn set_checksum_evf_v4(&mut self, checksum: Vec<u8>) -> Result<u8, ()> {
        // avf、evfv3、evfv4的典型高级录像体积对比，单位kB
        // 压缩前：64.2，63.9，47.9
        // 压缩后(zip)：25.4，24.6，6.84
        // 压缩后(gzip)：25.2，24.7，6.6
        // 压缩后(xz-6)：10.9，11.1，4.98
        if self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Win
        {
            return Err(());
        };
        self.raw_data
            .truncate(self.raw_data.len() - self.checksum.len() - 2);
        self.raw_data
            .extend_from_slice(&(checksum.len() as u16).to_be_bytes());
        self.raw_data
            .append(&mut checksum.clone().to_vec().to_owned());
        return Ok(0);
    }
    pub fn get_raw_data(&self) -> Result<Vec<u8>, ()> {
        if self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Display
        {
            return Err(());
        }
        Ok(self.raw_data.clone())
    }
    pub fn get_left(&self) -> usize {
        match self.game_board_state {
            GameBoardState::Display => {
                self.video_action_state_recorder[self.current_event_id]
                    .key_dynamic_params
                    .left
            }
            _ => self.minesweeper_board.left,
        }
    }
    pub fn get_right(&self) -> usize {
        match self.game_board_state {
            GameBoardState::Display => {
                self.video_action_state_recorder[self.current_event_id]
                    .key_dynamic_params
                    .right
            }
            _ => self.minesweeper_board.right,
        }
    }
    pub fn get_double(&self) -> usize {
        match self.game_board_state {
            GameBoardState::Display => {
                self.video_action_state_recorder[self.current_event_id]
                    .key_dynamic_params
                    .double
            }
            _ => self.minesweeper_board.double,
        }
    }
    pub fn get_cl(&self) -> usize {
        self.get_left() + self.get_right() + self.get_double()
    }
    pub fn get_flag(&self) -> usize {
        match self.game_board_state {
            GameBoardState::Display => {
                self.video_action_state_recorder[self.current_event_id]
                    .key_dynamic_params
                    .flag
            }
            _ => self.minesweeper_board.flag,
        }
    }
    pub fn get_left_s(&self) -> f64 {
        match self.game_board_state {
            GameBoardState::Display => {
                if self.current_time < 0.00099 {
                    return 0.0;
                }
                self.get_left() as f64 / self.current_time
            }
            GameBoardState::Loss | GameBoardState::Win => self.game_dynamic_params.left_s,
            GameBoardState::PreFlaging | GameBoardState::Ready => 0.0,
            #[cfg(any(feature = "py", feature = "rs"))]
            GameBoardState::Playing => {
                let now = Instant::now();
                let t_ms = now.duration_since(self.video_start_instant).as_millis() as u32;
                self.get_left() as f64 * 1000.0 / (t_ms - self.game_start_ms) as f64
            }
            #[allow(unreachable_patterns)]
            #[cfg(any(feature = "js"))]
            GameBoardState::Playing => 0.0,
        }
    }
    pub fn get_right_s(&self) -> f64 {
        match self.game_board_state {
            GameBoardState::Display => {
                if self.current_time < 0.00099 {
                    return 0.0;
                }
                self.get_right() as f64 / self.current_time
            }
            GameBoardState::Loss | GameBoardState::Win => self.game_dynamic_params.right_s,
            GameBoardState::PreFlaging | GameBoardState::Ready => 0.0,
            #[cfg(any(feature = "py", feature = "rs"))]
            GameBoardState::Playing => {
                let now = Instant::now();
                let t_ms = now.duration_since(self.video_start_instant).as_millis() as u32;
                self.get_right() as f64 * 1000.0 / (t_ms - self.game_start_ms) as f64
            }
            #[allow(unreachable_patterns)]
            #[cfg(any(feature = "js"))]
            GameBoardState::Playing => 0.0,
        }
    }
    pub fn get_double_s(&self) -> f64 {
        match self.game_board_state {
            GameBoardState::Display => {
                if self.current_time < 0.00099 {
                    return 0.0;
                }
                self.get_double() as f64 / self.current_time
            }
            GameBoardState::Loss | GameBoardState::Win => self.game_dynamic_params.double_s,
            GameBoardState::PreFlaging | GameBoardState::Ready => 0.0,
            #[cfg(any(feature = "py", feature = "rs"))]
            GameBoardState::Playing => {
                let now = Instant::now();
                let t_ms = now.duration_since(self.video_start_instant).as_millis() as u32;
                self.get_double() as f64 * 1000.0 / (t_ms - self.game_start_ms) as f64
            }
            #[allow(unreachable_patterns)]
            #[cfg(any(feature = "js"))]
            GameBoardState::Playing => 0.0,
        }
    }
    pub fn get_cl_s(&self) -> f64 {
        match self.game_board_state {
            GameBoardState::Display => {
                if self.current_time < 0.00099 {
                    return 0.0;
                }
                self.get_cl() as f64 / self.current_time
            }
            GameBoardState::Loss | GameBoardState::Win => self.game_dynamic_params.cl_s,
            GameBoardState::PreFlaging | GameBoardState::Ready => 0.0,
            #[cfg(any(feature = "py", feature = "rs"))]
            GameBoardState::Playing => {
                let now = Instant::now();
                let t_ms = now.duration_since(self.video_start_instant).as_millis() as u32;
                self.get_cl() as f64 * 1000.0 / (t_ms - self.game_start_ms) as f64
            }
            #[allow(unreachable_patterns)]
            #[cfg(any(feature = "js"))]
            GameBoardState::Playing => 0.0,
        }
    }
    pub fn get_flag_s(&self) -> f64 {
        match self.game_board_state {
            GameBoardState::Display => {
                if self.current_time < 0.00099 {
                    return 0.0;
                }
                self.get_flag() as f64 / self.current_time
            }
            GameBoardState::Loss | GameBoardState::Win => self.game_dynamic_params.flag_s,
            GameBoardState::PreFlaging | GameBoardState::Ready => 0.0,
            #[cfg(any(feature = "py", feature = "rs"))]
            GameBoardState::Playing => {
                let now = Instant::now();
                let t_ms = now.duration_since(self.video_start_instant).as_millis() as u32;
                self.get_flag() as f64 * 1000.0 / (t_ms - self.game_start_ms) as f64
            }
            #[allow(unreachable_patterns)]
            #[cfg(any(feature = "js"))]
            GameBoardState::Playing => 0.0,
        }
    }
    pub fn get_path(&self) -> f64 {
        if self.video_action_state_recorder.is_empty() {
            return 0.0;
        }
        if self.game_board_state == GameBoardState::Display {
            self.video_action_state_recorder[self.current_event_id].path
        } else {
            self.video_action_state_recorder.last().unwrap().path
        }
    }
    pub fn get_etime(&self) -> Result<f64, ()> {
        let bbbv_solved = self.get_bbbv_solved()?;
        if bbbv_solved == 0 {
            return Ok(0.0);
        }
        if self.game_board_state == GameBoardState::Display {
            Ok(self.current_time / bbbv_solved as f64 * self.static_params.bbbv as f64)
        } else {
            let t = self.game_dynamic_params.rtime;
            Ok(t / bbbv_solved as f64 * self.static_params.bbbv as f64)
        }
    }
    pub fn get_bbbv_s(&self) -> Result<f64, ()> {
        let bbbv_solved = self.get_bbbv_solved()?;
        if self.game_board_state == GameBoardState::Display {
            if self.current_time < 0.00099 {
                return Ok(0.0);
            }
            return Ok(bbbv_solved as f64 / self.current_time);
        }
        Ok(bbbv_solved as f64 / self.game_dynamic_params.rtime)
    }
    pub fn get_bbbv_solved(&self) -> Result<usize, ()> {
        match self.game_board_state {
            GameBoardState::Display => Ok(self.video_action_state_recorder[self.current_event_id]
                .key_dynamic_params
                .bbbv_solved),
            // GameBoardState::Win | GameBoardState::Loss => Ok(self
            //     .video_dynamic_params
            //     .bbbv_solved),
            GameBoardState::Win | GameBoardState::Loss => Ok(self
                .video_action_state_recorder
                .last()
                .unwrap()
                .key_dynamic_params
                .bbbv_solved),
            _ => Err(()),
        }
    }
    pub fn get_stnb(&self) -> Result<f64, ()> {
        let bbbv_solved = self.get_bbbv_solved()?;
        if self.game_board_state == GameBoardState::Display && self.current_time < 0.00099 {
            return Ok(0.0);
        }
        let c;
        match (self.height, self.width, self.mine_num) {
            (8, 8, 10) => c = 47.299,
            (16, 16, 40) => c = 153.73,
            (16, 30, 99) => c = 435.001,
            _ => return Ok(0.0),
        }

        if self.game_board_state == GameBoardState::Display {
            Ok(c * bbbv_solved as f64 / self.current_time.powf(1.7)
                * (bbbv_solved as f64 / self.static_params.bbbv as f64).powf(0.5))
        } else {
            Ok(
                c * bbbv_solved as f64 / self.game_dynamic_params.rtime.powf(1.7)
                    * (bbbv_solved as f64 / self.static_params.bbbv as f64).powf(0.5),
            )
        }
    }
    pub fn get_rqp(&self) -> Result<f64, ()> {
        let bbbv_solved = self.get_bbbv_solved()?;
        if bbbv_solved == 0 {
            return Ok(0.0);
        }
        Ok(self.current_time.powf(2.0) / bbbv_solved as f64)
    }
    pub fn get_qg(&self) -> Result<f64, ()> {
        let bbbv_solved = self.get_bbbv_solved()?;
        if bbbv_solved == 0 {
            return Ok(0.0);
        }
        Ok(self.current_time.powf(1.7) / bbbv_solved as f64)
    }
    pub fn get_lce(&self) -> Result<usize, ()> {
        match self.game_board_state {
            GameBoardState::Display => Ok(self.video_action_state_recorder[self.current_event_id]
                .key_dynamic_params
                .lce),
            GameBoardState::Win | GameBoardState::Loss => Ok(self
                .video_action_state_recorder
                .last()
                .unwrap()
                .key_dynamic_params
                .lce),
            _ => Err(()),
        }
    }
    pub fn get_rce(&self) -> Result<usize, ()> {
        match self.game_board_state {
            GameBoardState::Display => Ok(self.video_action_state_recorder[self.current_event_id]
                .key_dynamic_params
                .rce),
            GameBoardState::Win | GameBoardState::Loss => Ok(self
                .video_action_state_recorder
                .last()
                .unwrap()
                .key_dynamic_params
                .rce),
            _ => Err(()),
        }
    }
    pub fn get_dce(&self) -> Result<usize, ()> {
        match self.game_board_state {
            GameBoardState::Display => Ok(self.video_action_state_recorder[self.current_event_id]
                .key_dynamic_params
                .dce),
            GameBoardState::Win | GameBoardState::Loss => Ok(self
                .video_action_state_recorder
                .last()
                .unwrap()
                .key_dynamic_params
                .dce),
            _ => Err(()),
        }
    }
    pub fn get_ce(&self) -> Result<usize, ()> {
        match self.game_board_state {
            GameBoardState::Display => {
                let p = &self.video_action_state_recorder[self.current_event_id].key_dynamic_params;
                Ok(p.lce + p.rce + p.dce)
            }
            GameBoardState::Win | GameBoardState::Loss => {
                let p = &self
                    .video_action_state_recorder
                    .last()
                    .unwrap()
                    .key_dynamic_params;
                Ok(p.lce + p.rce + p.dce)
            }
            _ => Err(()),
        }
    }
    pub fn get_ce_s(&self) -> Result<f64, ()> {
        let ce = self.get_ce()?;
        if self.current_time < 0.00099 {
            return Ok(0.0);
        }
        Ok(ce as f64 / self.current_time)
    }
    pub fn get_corr(&self) -> Result<f64, ()> {
        let ce = self.get_ce()?;
        let cl = self.get_cl();
        if cl == 0 {
            return Ok(0.0);
        }
        Ok(ce as f64 / cl as f64)
    }
    pub fn get_thrp(&self) -> Result<f64, ()> {
        let ce = self.get_ce()?;
        let bbbv_solved = self.get_bbbv_solved().unwrap();
        if ce == 0 {
            return Ok(0.0);
        }
        Ok(bbbv_solved as f64 / ce as f64)
    }
    pub fn get_ioe(&self) -> Result<f64, ()> {
        let bbbv_solved = self.get_bbbv_solved()?;
        let cl = self.get_cl();
        if cl == 0 {
            return Ok(0.0);
        }
        Ok(bbbv_solved as f64 / cl as f64)
    }
    // 未实现
    pub fn get_op_solved(&self) -> Result<usize, ()> {
        if self.game_board_state != GameBoardState::Display
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Loss
        {
            return Err(());
        };
        Ok(self.video_action_state_recorder[self.current_event_id]
            .key_dynamic_params
            .op_solved)
    }
    // 未实现
    pub fn get_isl_solved(&self) -> Result<usize, ()> {
        if self.game_board_state != GameBoardState::Display
            && self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Loss
        {
            return Err(());
        };
        Ok(self.video_action_state_recorder[self.current_event_id]
            .key_dynamic_params
            .isl_solved)
    }
    /// 跨语言调用时，不能传递枚举体用这个
    pub fn get_mouse_state(&self) -> usize {
        let m_s;
        if self.game_board_state == GameBoardState::Display {
            m_s = self.video_action_state_recorder[self.current_event_id].mouse_state;
        } else {
            m_s = self.minesweeper_board.mouse_state;
        }
        match m_s {
            MouseState::UpUp => 1,
            MouseState::UpDown => 2,
            MouseState::UpDownNotFlag => 3,
            MouseState::DownUp => 4,
            MouseState::Chording => 5,
            MouseState::ChordingNotFlag => 6,
            MouseState::DownUpAfterChording => 7,
            MouseState::Undefined => 8,
        }
    }
    pub fn get_checksum(&self) -> Result<Vec<u8>, ()> {
        if self.game_board_state != GameBoardState::Win
            && self.game_board_state != GameBoardState::Loss
            && self.game_board_state != GameBoardState::Display
        {
            return Err(());
        }
        Ok(self.checksum.clone())
    }
    /// 录像播放时，返回鼠标的坐标。
    /// 避开局面外的操作（记为最右下角）
    pub fn get_x_y(&self) -> Result<(u16, u16), ()> {
        if self.game_board_state != GameBoardState::Display {
            return Err(());
        };
        let mut k = 0;
        loop {
            if let Some(Event::Mouse(mouse_event)) =
                &self.video_action_state_recorder[self.current_event_id - k].event
            {
                if mouse_event.x < self.cell_pixel_size as u16 * self.width as u16 {
                    return Ok((
                        (mouse_event.x as f64 * self.video_playing_pix_size_k) as u16,
                        (mouse_event.y as f64 * self.video_playing_pix_size_k) as u16,
                    ));
                }
            }
            k += 1;
        }
    }
    // 返回录像文件里记录的方格尺寸。flop_new播放器里会用到。这是因为元扫雷和flop播放器的播放机制不同。
    pub fn get_pix_size(&self) -> Result<u8, ()> {
        if self.game_board_state != GameBoardState::Display {
            return Err(());
        };
        Ok(self.cell_pixel_size)
    }
    // 录像播放时，设置按何种像素播放，涉及鼠标位置回报
    pub fn set_video_playing_pix_size(&mut self, pix_size: u8) {
        if self.game_board_state != GameBoardState::Display {
            panic!("");
        };
        self.video_playing_pix_size_k = pix_size as f64 / self.cell_pixel_size as f64;
    }
}
