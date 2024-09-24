//! # 扫雷算法工具箱
//! 基于Rust语言，提供扫雷游戏相关算法的高效、内存安全的实现，并发布到各个平台。目前包括[crates.io](https://crates.io/crates/ms_toollib)、[pypi.org](https://pypi.org/project/ms-toollib/)、[npmjs.com](https://www.npmjs.com/package/ms-toollib)这三个平台。Python、Rust、Javascript、Typescript的用户可以流畅地使用相应的功能，C、C++的用户也可以使用。安装、使用这些工具箱需要有对应语言的基本的知识。项目地址在[ms_toollib](https://github.com/eee555/ms_toollib)。以下是快速入门。
//! ## 特殊名词解释
//! 注意：只包含本工具箱的文档中特有的名词解释。读者需具备“扫雷术语”等前置知识。
//! 1. “游戏局面”：变量名为game_board: Vec<Vec<i32>>；其类型在Python中为List[List[int]]；在Javascript、Typescript中为Array(Array())；在C中为struct Board { struct Row *rows; size_t n_row; }; struct Row { int32_t *cells; size_t n_column; }；在C++中为std::vector<int32_t>。其中0代表空；1到8代表数字1到8；10代表未打开；11代表算法认为是雷（百分百正确的），或玩家在游戏中标的雷（玩家认为这是雷，但玩家可能犯错）；12代表算法已经确定该位置不是雷，但玩家暂时还没点开，模样仍然是未打开的样子；14表示踩到了雷游戏失败以后显示的标错的雷对应叉雷；15表示踩到了雷游戏失败了对应红雷；15表示背景不透明的白雷，失败后显示出来的其他的雷；18表示局面中，由于双击的高亮，导致看起来像0的格子。第一个索引是行，第二个索引是列，例如：高级中，game_board[0][0]代表最左上角位置、game_board[15][29]代表最右下角位置  
//! 1. “真实局面”或“局面”：变量名为board: Vec<Vec<i32>>。其中0代表空；1到8代表数字1到8；-1代表是雷。  
//!     - 解释：游戏局面和局面的区别在于，游戏局面是游戏时玩家看见的局面，随鼠标的点击操作而变化；而真实局面是可以看见雷的实际局面，不会随操作而变化。  
//!     - 注意：游戏局面中11的作用类似于游戏时的标雷，但是区别在于，玩家标出的雷可能是错误的，而算法的判断一定是正确的。这两种情况都用同一个数字表示。因为算法需要保证百分百的正确性，通俗地讲，玩家标出来的雷，算法一个也不相信；意味着这两种含义不可能同时出现。  
//! 1. “矩阵”：判雷的本质是求解带有0-1约束的非齐次线性欠定方程组。在线性代数方程Ax=b中，矩阵A的变量名为matrix_a，代表系数矩阵；矩阵x的变量名为matrix_x，代表未知量矩阵；矩阵b的变量名为matrix_b，代表常数矩阵。变量后加上s代表分段；加上ses代表分块且分段。  
//! 1. “分块”：计算概率时，假如把整个局面全列成一个方程式，那么，方程中的变量数等于空的边缘的格子数。一般数目较大不易求解，需要使用分块技巧来加速。首先直观地看，游戏局面中，贯通上下两边的空可以把游戏局面分成两块；复杂的空可以把游戏局面分割成更多块。因此计算概率时，可以独立讨论被空分割成的若干块的情况（即系数矩阵可以分块），最后汇总计算出最终结果。
//! 1. “分段”：仅仅分块还是不够，一些块的边缘仍然很长。但是在块的边缘，未知格往往是两两相邻的（而不是多个未知格互相相邻，即无向图的割点很多），而且其中部分未知格往往是可以简单求出的。只要求出这些未知格并代入原方程，就可以把大方程分割成小方程（即系数矩阵为分块对角矩阵），从而加速求解。
//! ## 函数签名说明
//! Rust是一门强类型的语言，其函数签名反映了诸多信息。以下为不熟悉本语言的开发人员提供简要的说明。
//! - 变量名+冒号+格式：表明参数的格式。例如i32代表有符号4字节的整数、u8代表无符号1字节的整数；Vec<>代表内存分配在堆上的可变长度的向量。
//! - mut：代表这个参数是可变的。例如pub fn mark_board(board: &mut Vec<Vec<i32>>)中，会对传入的局面直接修改。如果不带mut，则不会修改。
//! ## API命名原则
//! 约定如下原则：
//! - 原则1：为方便开发人员使用，本工具箱在所有平台所有的api都是相同的。
//! - 原则2：所有平台的版本号原则上相同，如果不相同，代表还未更新到。
//! - 原则3：结构体和类名均使用大驼峰命名法(CamelCase)、方法名和函数名均使用蛇形命名法（snake_case亦称下划线命名法）。
//! ## 项目背景
//! [ms_toollib](https://github.com/eee555/ms_toollib)工具箱的设计绝不是纸上谈兵，是由[元扫雷](https://github.com/eee555/Solvable-Minesweeper)(亦称黑猫扫雷)项目的算法部分拆分而来，算法经过实际使用验证，具有深厚的项目背景。
//! ## 安全性
//! 本工具箱不直接提供机扫相关工具；同时，不提倡纯粹机扫相关的研究，尤其不提倡那些通过机扫模拟人类扫雷的研究；使用机扫的录像攻击排名网站的审查体系是严格禁止的，任何相关尝试都是不道德的！

// cargo doc --features rs --no-deps --open
// cargo build --release --features py
// cargo bench
// cargo publish --features rs
// 需要换成官方的源，不能用镜像
// cargo yank --vers 0.0.1
mod utils;
#[cfg(any(feature = "js"))]
pub use utils::get_random_int;
pub use utils::{
    cal_all_solution, cal_bbbv, cal_board_numbers, cal_cell_nums, cal_isl, cal_op,
    cal_table_minenum_recursion, combine, is_good_chording, laymine, laymine_op, refresh_board,
    refresh_matrix, refresh_matrixs, refresh_matrixses, unsolvable_structure,
};

mod miscellaneous;

mod algorithms;
#[cfg(any(feature = "py", feature = "rs"))]
pub use algorithms::{agent_step, laymine_solvable_thread, sample_3BVs_exp, OBR_board};

pub use algorithms::{
    cal_is_op_possibility_cells, cal_possibility, cal_possibility_onboard,
    get_all_not_and_is_mine_on_board, is_able_to_solve, is_guess_while_needless, is_solvable,
    laymine_solvable, laymine_solvable_adjust, mark_board, solve_direct, solve_enumerate,
    solve_minus, try_solve,
};
// #[cfg(any(feature = "rs"))]
// pub use algorithms::{mark_board, solve_direct, solve_enumerate, solve_minus};

mod board;
pub use board::{Board, GameBoard};

pub mod videos;
pub use videos::{
    valid_time_period, AvfVideo, BaseVideo, EvfVideo, GameBoardState, MinesweeperBoard, MouseState,
    MvfVideo, RmvVideo,
};

#[cfg(any(feature = "py", feature = "rs"))]
mod OBR;
#[cfg(any(feature = "py", feature = "rs"))]
pub use OBR::ImageBoard;

// #[cfg(any(feature = "py", feature = "rs"))]
mod safe_board;
#[cfg(any(feature = "py", feature = "rs"))]
pub use crate::safe_board::SafeBoard;
#[cfg(any(feature = "py", feature = "rs"))]
pub use crate::safe_board::SafeBoardRow;

#[cfg(any(feature = "py", feature = "rs"))]
use crate::safe_board::BoardSize;

// 最大枚举长度限制。超过这个长度，概率计算不准。全雷网的高级没有发现超出此限制的。
const ENUM_LIMIT: usize = 55;
