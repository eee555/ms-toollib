//! # 扫雷算法工具箱
//! 提供扫雷相关算法的rust实现，并发布到各个平台

// cargo doc --features rs --no-deps --open
// cargo build --release --features py
// cargo bench
// cargo publish --features rs --features py --features js
// cargo yank --vers 0.0.1
mod utils;
pub use utils::{
    cal3BV, cal_op, cal_table_minenum_recursion, combine, laymine_number, laymine_op_number,
    refresh_board, refresh_matrix, refresh_matrixs, unsolvable_structure,
};

mod algorithms;
#[cfg(any(feature = "py", feature = "rs"))]
pub use algorithms::{laymine_solvable_thread, sample_3BVs_exp, OBR_board};
// #[cfg(feature = "js")]
pub use algorithms::{
    cal_is_op_possibility_cells, cal_possibility, cal_possibility_onboard, is_solvable, laymine,
    laymine_op, laymine_solvable, mark_board, solve_direct, solve_enumerate, solve_minus,
};

mod board;
pub use board::MinesweeperBoard;
#[cfg(any(feature = "py", feature = "rs"))]
mod OBR;
#[cfg(any(feature = "py", feature = "rs"))]
pub use OBR::ImageBoard;



