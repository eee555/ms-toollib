// 杂项工具
#[cfg(any(feature = "py", feature = "rs"))]
use std::time::Instant;

/// 将游戏时间从IEEE754标准规定的64位浮点数转为精确的以毫秒为单位的整数。四舍五入。
pub fn s_to_ms(time: f64) -> u32 {
    (time * 1000.0).round() as u32
}

/// 返回以毫秒为单位的时间。四舍五入。
// 拟弃用，被代替：let mut time_ms = step_instant
// .duration_since(self.video_start_instant)
// .as_millis() as u32;
#[cfg(any(feature = "py", feature = "rs"))]
pub fn time_ms_between(future: Instant, past: Instant) -> u32 {
    (future.duration_since(past).as_micros() as f64 / 1000.0).round() as u32
}


