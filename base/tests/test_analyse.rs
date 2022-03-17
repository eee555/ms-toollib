// 测试录像分析模块
use ms_toollib::{MinesweeperBoard};

#[test]
fn minesweeper_board_works() {
    // 局面状态机测试
    let board = vec![
        vec![0, 0, 1, -1, 2, 1, 1, -1],
        vec![0, 0, 2, 3, -1, 3, 3, 2],
        vec![1, 1, 3, -1, 4, -1, -1, 2],
        vec![2, -1, 4, -1, 3, 4, -1, 4],
        vec![3, -1, 5, 2, 1, 3, -1, -1],
        vec![3, -1, -1, 2, 1, 2, -1, 3],
        vec![-1, 5, 4, -1, 1, 1, 2, 2],
        vec![-1, 3, -1, 2, 1, 0, 1, -1],
    ];
    let mut my_board = MinesweeperBoard::new(board);
    my_board.step_flow(vec![("lc", (0, 0)), ("lr", (0, 0)), ("lc", (0, 3)), ("lr", (0, 3))]).unwrap();
}

