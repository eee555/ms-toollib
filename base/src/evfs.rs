use crate::videos::{ErrReadVideoReason, EvfVideo, NewSomeVideo, NewSomeVideo2};
#[cfg(any(feature = "py", feature = "rs"))]
use std::fs;
use std::ops::{Index, IndexMut};
#[cfg(any(feature = "py", feature = "rs"))]
use std::path::Path;

use itertools::Itertools;

/// evfs文件
pub struct Evfs {
    pub raw_data: Vec<u8>,
    pub cells: Vec<EvfsCell>,
    /// 解析raw_data的偏移量
    pub offset: usize,
}

/// evfs重复结构的一个单元
#[derive(Clone)]
pub struct EvfsCell {
    pub evf_video: EvfVideo,
    pub checksum: Vec<u8>,
}

impl Evfs {
    /// 创建一个空白的evfs文件
    pub fn new() -> Self {
        Evfs {
            raw_data: vec![],
            cells: vec![],
            offset: 0,
        }
    }
    /// 解析已有的evfs文件的二进制数据
    pub fn new_with_data(data: Vec<u8>) -> Self {
        Evfs {
            raw_data: data,
            cells: vec![],
            offset: 0,
        }
    }
    /// 向末尾追加录像，文件名不要带后缀
    pub fn append(&mut self, data: Vec<u8>, file_name: &str, checksum: Vec<u8>) {
        self.cells.push(EvfsCell {
            evf_video: <EvfVideo as NewSomeVideo2<Vec<u8>, &str>>::new(data, file_name),
            checksum,
        });
    }
    pub fn len(&self) -> usize {
        self.cells.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
    pub fn clear(&mut self) {
        self.cells.clear();
    }
    /// 初步验证evfs文件的有效性。适用于网页前端，并不严格。
    pub fn is_valid(&mut self) -> bool {
        if self.cells.is_empty() {
            return false;
        }
        for cell in self.cells.iter_mut() {
            if !cell.evf_video.data.can_analyse {
                if cell.evf_video.parse_video().is_err() {
                    return false;
                }
            }
        }
        if !self.cells.iter().map(|c| c.evf_video.version).all_equal() {
            return false;
        }
        if !self
            .cells
            .iter()
            .map(|c| &c.evf_video.data.software)
            .all_equal()
        {
            return false;
        }
        if !self
            .cells
            .iter()
            .map(|c| &c.evf_video.data.country)
            .all_equal()
        {
            return false;
        }
        if !self
            .cells
            .iter()
            .map(|c| &c.evf_video.data.player_identifier)
            .all_equal()
        {
            return false;
        }
        if !self
            .cells
            .iter()
            .map(|c| &c.evf_video.data.race_identifier)
            .all_equal()
        {
            return false;
        }
        if !self
            .cells
            .iter()
            .map(|c| &c.evf_video.data.uniqueness_identifier)
            .all_equal()
        {
            return false;
        }
        // 验证时间递增
        if self.cells[0].evf_video.data.start_time > self.cells[0].evf_video.data.end_time {
            return false;
        }
        if self.cells.len() > 1 {
            for i in 1..self.cells.len() {
                if self.cells[i - 1].evf_video.data.end_time
                    > self.cells[i].evf_video.data.start_time
                {
                    return false;
                }
                if self.cells[i].evf_video.data.start_time > self.cells[i].evf_video.data.end_time {
                    return false;
                }
            }
        }

        if !self.cells.iter().all(|c| {
            c.evf_video.data.is_fair
                && c.evf_video.version >= 4
                && !c.evf_video.data.checksum.is_empty()
        }) {
            return false;
        }
        true
    }
    pub fn get_software(&self) -> &str {
        &self.cells[0].evf_video.data.software
    }
    pub fn get_evf_version(&self) -> u8 {
        self.cells[0].evf_video.version
    }
    pub fn get_start_time(&self) -> u64 {
        self.cells[0].evf_video.data.start_time
    }
    pub fn get_end_time(&self) -> u64 {
        self.cells.last().unwrap().evf_video.data.end_time
    }

    /// 生成evfs_v0文件的二进制数据
    pub fn generate_evfs_v0_raw_data(&mut self) {
        if self.cells.is_empty() {
            return;
        }
        if !self.cells.iter().map(|c| c.checksum.len()).all_equal() {
            panic!("Evfs cells have different checksum lengths");
        }
        self.raw_data = vec![0];
        let chechsum_len = self.cells[0].checksum.len() as u16;
        self.raw_data.push((chechsum_len >> 8).try_into().unwrap());
        self.raw_data.push((chechsum_len % 256).try_into().unwrap());

        for cell in self.cells.iter_mut() {
            self.raw_data
                .append(&mut cell.evf_video.file_name.clone().into_bytes());
            self.raw_data.push(0);
            let evf_raw_data = cell.evf_video.data.get_raw_data().unwrap();
            let evf_size = evf_raw_data.len() as u32;
            self.raw_data.push((evf_size >> 24).try_into().unwrap());
            self.raw_data.push((evf_size >> 16).try_into().unwrap());
            self.raw_data.push((evf_size >> 8).try_into().unwrap());
            self.raw_data.push((evf_size % 256).try_into().unwrap());
            self.raw_data.extend_from_slice(&evf_raw_data);
            self.raw_data.extend_from_slice(&cell.checksum);
        }
    }
    pub fn parse(&mut self) -> Result<(), ErrReadVideoReason> {
        let version = self.get_u8()?;
        match version {
            0 => self.parse_v0()?,
            _ => {},
        }
        
        for cell in self.cells.iter_mut() {
            if !cell.evf_video.data.can_analyse {
                cell.evf_video.parse_video()?;
            }
        }
        Ok(())
    }
    /// 0.0-0.1版本
    fn parse_v0(&mut self) -> Result<(), ErrReadVideoReason> {
        let checksum_len = self.get_u16()?;
        while self.offset < self.raw_data.len() - 1 {
            let file_name = self.get_utf8_c_string('\0')?;
            let file_size = self.get_u32()?;
            let evf_data = self.get_buffer(file_size as usize)?;
            let checksum = self.get_buffer(checksum_len)?;
            self.cells.push(EvfsCell {
                evf_video: <EvfVideo as NewSomeVideo2<Vec<u8>, &str>>::new(evf_data, &file_name),
                checksum,
            });
        }
        Ok(())
    }
}

impl Evfs {
    pub fn get_u8(&mut self) -> Result<u8, ErrReadVideoReason> {
        let t = self.raw_data.get(self.offset);
        self.offset += 1;
        match t {
            Some(x) => Ok(*x),
            None => Err(ErrReadVideoReason::FileIsTooShort),
        }
    }
    /// 都是大端法
    pub fn get_u16(&mut self) -> Result<u16, ErrReadVideoReason> {
        let a = self.get_u8()?;
        let b = self.get_u8()?;
        Ok((a as u16) << 8 | (b as u16))
    }
    pub fn get_i16(&mut self) -> Result<i16, ErrReadVideoReason> {
        let a = self.get_u8()?;
        let b = self.get_u8()?;
        Ok((a as i16) << 8 | (b as i16))
    }
    pub fn get_u24(&mut self) -> Result<u32, ErrReadVideoReason> {
        let a = self.get_u8()?;
        let b = self.get_u8()?;
        let c = self.get_u8()?;
        Ok((a as u32) << 16 | (b as u32) << 8 | (c as u32))
    }
    pub fn get_u32(&mut self) -> Result<u32, ErrReadVideoReason> {
        let a = self.get_u8()?;
        let b = self.get_u8()?;
        let c = self.get_u8()?;
        let d = self.get_u8()?;
        Ok((a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | (d as u32))
    }
    pub fn get_u64(&mut self) -> Result<u64, ErrReadVideoReason> {
        let a = self.get_u32()?;
        let b = self.get_u32()?;
        Ok((a as u64) << 32 | (b as u64))
    }
    pub fn get_char(&mut self) -> Result<char, ErrReadVideoReason> {
        let a = self.get_u8()?;
        Ok(a as char)
    }
    pub fn get_buffer<U>(&mut self, length: U) -> Result<Vec<u8>, ErrReadVideoReason>
    where
        U: Into<usize>,
    {
        let length = length.into();
        self.offset += length;
        self.raw_data
            .get((self.offset - length)..self.offset)
            .map(|vv| vv.to_vec())
            .ok_or(ErrReadVideoReason::FileIsTooShort)
    }
    pub fn get_c_buffer(&mut self, end: char) -> Result<Vec<u8>, ErrReadVideoReason> {
        let mut s = vec![];
        loop {
            let the_byte = self.get_char()?;
            if the_byte == end {
                break;
            }
            s.push(the_byte as u8);
        }
        Ok(s)
    }
    pub fn get_utf8_string<U>(&mut self, length: U) -> Result<String, ErrReadVideoReason>
    where
        U: Into<usize>,
    {
        let length = length.into();
        String::from_utf8(self.get_buffer(length)?).map_err(|_e| ErrReadVideoReason::Utf8Error)
    }
    /// 读取以end结尾的合法utf-8字符串
    pub fn get_utf8_c_string(&mut self, end: char) -> Result<String, ErrReadVideoReason> {
        String::from_utf8(self.get_c_buffer(end)?).map_err(|_e| ErrReadVideoReason::Utf8Error)
    }
}

#[cfg(any(feature = "py", feature = "rs"))]
impl Evfs {
    /// 解析已有的evfs文件的二进制数据
    pub fn new_with_file(filename: &str) -> Self {
        let data = fs::read(filename).unwrap();
        Evfs {
            raw_data: data,
            cells: vec![],
            offset: 0,
        }
    }
    /// 将evfs中的所有录像保存到指定目录，文件名为原文件名加上.evf后缀
    pub fn save_evf_files(&self, dir: &str) {
        let path = Path::new(dir);
        for cell in self.cells.iter() {
            cell.evf_video.data.save_to_evf_file(
                path.join(cell.evf_video.file_name.clone())
                    .to_str()
                    .unwrap(),
            );
        }
    }

    /// 将单个evfs文件保存到指定目录(绝对路径)，文件名为原文件名加上.evfs后缀
    /// 重复文件，xxx.evfs变成xxx(2).evfs
    pub fn save_evfs_file(&self, file_name: &str) -> String {
        if self.raw_data.is_empty() {
            panic!("Evfs raw data is empty, please generate it first.");
        }

        let file_exist =
            std::path::Path::new((file_name.to_string() + &(".evfs".to_string())).as_str())
                .exists();
        if !file_exist {
            fs::write(
                (file_name.to_string() + &(".evfs".to_string())).as_str(),
                &self.raw_data,
            )
            .unwrap();
            return (file_name.to_string() + &(".evfs".to_string()))
                .as_str()
                .to_string();
        } else {
            let mut id = 2;
            let mut format_name;
            loop {
                format_name = file_name.to_string() + &(format!("({}).evfs", id).to_string());
                let new_file_name = format_name.as_str();
                let file_exist = std::path::Path::new(new_file_name).exists();
                if !file_exist {
                    fs::write(new_file_name, &self.raw_data).unwrap();
                    return new_file_name.to_string();
                }
                id += 1;
            }
        }
    }
}

// 为 Evfs 实现 Index trait，使其支持不可变索引（只读访问）
impl Index<usize> for Evfs {
    type Output = EvfsCell;
    fn index(&self, index: usize) -> &Self::Output {
        &self.cells[index]
    }
}

// 为 Evfs 实现 IndexMut trait，使其支持可变索引（可修改访问）
impl IndexMut<usize> for Evfs {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.cells[index]
    }
}

impl Index<std::ops::Range<usize>> for Evfs {
    type Output = [EvfsCell];
    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        let cells = &self.cells[index.start..index.end];
        cells
    }
}
