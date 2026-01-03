use crate::videos::types::ErrReadVideoReason;
use encoding_rs::{GB18030, WINDOWS_1252};
// 实现了文件字节读取的 trait，读取各种整数、字符串，解析阿比特时间戳等

pub trait ByteReader {
    /// 返回底层字节切片
    fn raw_data(&self) -> &[u8];
    /// 返回 offset 的可变引用，用于自动推进
    fn offset_mut(&mut self) -> &mut usize;
    fn get_u8(&mut self) -> Result<u8, ErrReadVideoReason> {
        let offset = *self.offset_mut();
        if let Some(&b) = self.raw_data().get(offset) {
            *self.offset_mut() += 1;
            Ok(b)
        } else {
            Err(ErrReadVideoReason::FileIsTooShort)
        }
    }
    /// 都是大端法
    fn get_u16(&mut self) -> Result<u16, ErrReadVideoReason> {
        let a = self.get_u8()?;
        let b = self.get_u8()?;
        Ok((a as u16) << 8 | (b as u16))
    }
    fn get_i16(&mut self) -> Result<i16, ErrReadVideoReason> {
        let a = self.get_u8()?;
        let b = self.get_u8()?;
        Ok((a as i16) << 8 | (b as i16))
    }
    fn get_u24(&mut self) -> Result<u32, ErrReadVideoReason> {
        let a = self.get_u8()?;
        let b = self.get_u8()?;
        let c = self.get_u8()?;
        Ok((a as u32) << 16 | (b as u32) << 8 | (c as u32))
    }
    fn get_u32(&mut self) -> Result<u32, ErrReadVideoReason> {
        let a = self.get_u8()?;
        let b = self.get_u8()?;
        let c = self.get_u8()?;
        let d = self.get_u8()?;
        Ok((a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | (d as u32))
    }
    fn get_u64(&mut self) -> Result<u64, ErrReadVideoReason> {
        let a = self.get_u32()?;
        let b = self.get_u32()?;
        Ok((a as u64) << 32 | (b as u64))
    }
    fn get_char(&mut self) -> Result<char, ErrReadVideoReason> {
        let a = self.get_u8()?;
        Ok(a as char)
    }
    fn get_buffer<U>(&mut self, length: U) -> Result<Vec<u8>, ErrReadVideoReason>
    where
        U: Into<usize>,
    {
        let length = length.into();
        let offset = *self.offset_mut();
        *self.offset_mut() += length;
        self.raw_data()
            .get(offset..(offset + length))
            .map(|vv| vv.to_vec())
            .ok_or(ErrReadVideoReason::FileIsTooShort)
    }
    fn get_c_buffer(&mut self, end: char) -> Result<Vec<u8>, ErrReadVideoReason> {
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
    fn get_utf8_string<U>(&mut self, length: U) -> Result<String, ErrReadVideoReason>
    where
        U: Into<usize>,
    {
        let length = length.into();
        String::from_utf8(self.get_buffer(length)?).map_err(|_e| ErrReadVideoReason::Utf8Error)
    }
    /// 读取以end结尾的合法utf-8字符串
    fn get_utf8_c_string(&mut self, end: char) -> Result<String, ErrReadVideoReason> {
        String::from_utf8(self.get_c_buffer(end)?).map_err(|_e| ErrReadVideoReason::Utf8Error)
    }
    fn get_unknown_encoding_string<U>(&mut self, length: U) -> Result<String, ErrReadVideoReason>
    where
        U: Into<usize>,
    {
        Self::get_unknown_encoding_string_from_buf(self.get_buffer(length)?)
    }
    fn get_unknown_encoding_string_from_buf(code: Vec<u8>) -> Result<String, ErrReadVideoReason> {
        if let Ok(s) = String::from_utf8(code.clone()) {
            return Ok(s);
        }
        match Self::get_unknown_cp_encoding_string_from_buf(code.clone()) {
            Ok(str) => Ok(str),
            Err(_) => Ok(String::from_utf8_lossy(&code).to_string()),
        }
    }
    // won't consider utf-8 at all - useful for replay versions only produced by
    // clones that never produce utf-8
    fn get_unknown_cp_encoding_string_from_buf(code: Vec<u8>) -> Result<String, ErrReadVideoReason> {
        let (cow, _, had_errors) = GB18030.decode(&code);
        if !had_errors {
            return Ok(cow.into_owned());
        };
        let (cow, _, had_errors) = WINDOWS_1252.decode(&code);
        if !had_errors {
            return Ok(cow.into_owned());
        };
        return Err(ErrReadVideoReason::InvalidParams);
    }
    /// 读取以end结尾的未知编码字符串，假如所有编码都失败，返回utf-8乱码
    fn get_unknown_encoding_c_string(&mut self, end: char) -> Result<String, ErrReadVideoReason> {
        let code = self.get_c_buffer(end)?;
        Self::get_unknown_encoding_string_from_buf(code)
    }
    // 是否闰年，计算阿比特时间戳
    fn is_leap_year(&self, year: u64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
    // 一个月有几天，计算阿比特时间戳
    fn days_in_month(&self, year: u64, month: u64) -> u32 {
        let days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        if month == 2 && self.is_leap_year(year) {
            29
        } else {
            days[(month - 1) as usize]
        }
    }

    fn days_since_epoch(&self, year: u64, month: u64, day: u64) -> u64 {
        let mut total_days = 0;
        for y in 1970..year {
            total_days += if self.is_leap_year(y) { 366 } else { 365 };
        }
        for m in 1..month {
            total_days += self.days_in_month(year, m) as u64;
        }
        total_days + day as u64 - 1
    }

    /// 解析avf里的开始时间戳，返回时间戳，微秒。“6606”只取后三位“606”，三位数取后两位
    /// "18.10.2022.20:15:35:6606" -> 1666124135606000
    fn parse_avf_start_timestamp(
        &mut self,
        start_timestamp: &str,
    ) -> Result<u64, ErrReadVideoReason> {
        let mut timestamp_parts = start_timestamp.split('.');
        let day = timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let month = timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let year = timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        timestamp_parts = timestamp_parts.next().unwrap().split(':');
        let hour = timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let minute = timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let second = timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let sub_second = timestamp_parts.next().unwrap()[1..]
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;

        let days = self.days_since_epoch(year, month, day);
        let total_seconds = days * 24 * 60 * 60 + hour * 60 * 60 + minute * 60 + second;
        let microseconds = total_seconds * 1_000_000 + sub_second * 1_000;

        Ok(microseconds)
    }

    // 解析avf里的结束时间戳，返回时间戳，微秒
    // "18.10.2022.20:15:35:6606", "18.20:16:24:8868" -> 1666124184868000
    fn parse_avf_end_timestamp(
        &mut self,
        start_timestamp: &str,
        end_timestamp: &str,
    ) -> Result<u64, ErrReadVideoReason> {
        let mut start_timestamp_parts = start_timestamp.split('.');
        let mut end_timestamp_parts = end_timestamp.split('.');
        let start_day = start_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let end_day = end_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let mut month = start_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let mut year = start_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        if start_day > end_day {
            // 跨月
            month += 1;
            if month >= 13 {
                month = 1;
                year += 1;
            }
        }
        end_timestamp_parts = end_timestamp_parts.next().unwrap().split(':');
        let hour = end_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let minute = end_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let second = end_timestamp_parts
            .next()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;
        let sub_second = end_timestamp_parts.next().unwrap()[1..]
            .parse::<u64>()
            .map_err(|_| ErrReadVideoReason::InvalidParams)?;

        let days = self.days_since_epoch(year, month, end_day);
        let total_seconds = days * 24 * 60 * 60 + hour * 60 * 60 + minute * 60 + second;
        let microseconds = total_seconds * 1_000_000 + sub_second * 1_000;

        Ok(microseconds)
    }
}
