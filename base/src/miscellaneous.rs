// 杂项工具

/// 将游戏时间从IEEE754标准规定的64位浮点数转为精确的以毫秒为单位的整数。
pub fn s_to_ms(time: f64) -> u32 {
    (time * 1000.0).round() as u32
}










