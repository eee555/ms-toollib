// 杂项工具
#[cfg(any(feature = "py", feature = "rs"))]
use std::time::{Instant};

/// 将游戏时间从IEEE754标准规定的64位浮点数转为精确的以毫秒为单位的整数。四舍五入。
pub fn s_to_ms(time: f64) -> u32 {
    (time * 1000.0).round() as u32
}

/// 返回以毫秒为单位的时间。四舍五入。
#[cfg(any(feature = "py", feature = "rs"))]
pub fn time_ms_between(futurn: Instant, past: Instant) -> u32 {
    (futurn.duration_since(past).as_micros() as f64 / 1000.0).round() as u32
}


