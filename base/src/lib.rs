//! # 扫雷算法工具箱
//! 基于Rust语言，提供扫雷相关算法的高效、内存安全的实现，并发布到各个平台。目前包括[crates.io](https://crates.io/crates/ms_toollib)、[pypi.org](https://pypi.org/project/ms-toollib/)、[npmjs.com](https://www.npmjs.com/package/ms-toollib)这三个平台。以下是快速入门。
//! ## 局面格式说明
//! - 游戏局面的变量名为game_board: Vec<Vec<i32>>；在Python中为List[List[int]]；在Javascript中对应Array(Array())之类的格式。其中0代表空；1到8代表数字1到8；10代表未打开；11代表算法确定是雷，12代表算法确定不是雷。第一个索引是行，第二个索引是列，例如：高级中，game_board[0][0]代表最左上角位置、game_board[15][29]代表最右下角位置  
//! - 局面的变量名为board: Vec<Vec<i32>>。其中0代表空；1到8代表数字1到8；-1代表是雷。  
//! - 解释：游戏局面和局面的区别在于，游戏局面是游戏时玩家看见的局面，随鼠标的点击操作而变化；而局面是实际的真实局面，
//! 不会随操作而变化。  
//! - 注意：游戏局面中11的作用类似于游戏时的标雷，但是区别在于，玩家标出的雷可能是错误的，而算法的判断一定是正确的。
//! 12的作用是算法标出不是雷的位置，但是玩家暂时还没有点击。  
//! ## 函数签名说明
//! Rust是一门强类型的语言，其函数签名反映了诸多信息。以下为不熟悉本语言的开发人员提供简要的说明。
//! - 变量名+冒号+格式：表明参数的格式。例如i32代表有符号4字节的整数、u8代表无符号1字节的整数；Vec<>代表内存分配在堆上的可变长度的向量。
//! - mut：代表这个参数是可变的。例如pub fn mark_board(board: &mut Vec<Vec<i32>>)中，会对传入的局面直接修改。如果不带mut，则不会修改。
//! ## API命名原则
//! - 原则1：为方便开发人员使用，本工具箱在所有平台所有的api都是相同的。
//! - 原则2：类名均使用大驼峰命名法、方法名和函数名均使用蛇形命名法（亦称下划线命名法）。
//! ## 安全性
//! 本工具箱不直接提供机扫相关工具；同时，不提倡纯粹机扫相关的研究，尤其不提倡那些通过机扫模拟人类扫雷的研究；使用机扫的录像攻击排名网站的审查体系是严格禁止的，任何相关尝试都是不道德的！

// cargo doc --features rs --no-deps --open
// cargo build --release --features py
// cargo bench
// cargo publish --features rs --features py --features js
// 需要换成官方的源，不能用镜像
// cargo yank --vers 0.0.1
mod utils;
pub use utils::{
    cal3BV, cal_op, cal_table_minenum_recursion, combine, laymine_number, laymine_op_number,
    refresh_board, refresh_matrix, refresh_matrixs, unsolvable_structure,
};

mod algorithms;
mod analyse_methods;
#[cfg(any(feature = "py", feature = "rs"))]
pub use algorithms::{laymine_solvable_thread, sample_3BVs_exp, OBR_board};
// #[cfg(feature = "js")]
pub use algorithms::{
    cal_is_op_possibility_cells, cal_possibility, cal_possibility_onboard, is_solvable, laymine,
    laymine_op, laymine_solvable, mark_board, solve_direct, solve_enumerate, solve_minus,
};

mod board;
pub use board::{AvfVideo, MinesweeperBoard};

#[cfg(any(feature = "py", feature = "rs"))]
mod OBR;
#[cfg(any(feature = "py", feature = "rs"))]
pub use OBR::ImageBoard;
