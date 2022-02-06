// use ms_toollib::refresh_matrixs;
use ms_toollib::{laymine_solvable_thread, laymine_solvable, laymine_solvable_adjust};

// 测试各种埋雷类的函数

#[test]
fn laymine_solvable_thread_works() {
    // 测试多线程筛选法无猜埋雷
    let game_board = laymine_solvable_thread(16, 30, 99, 0, 0, 0, 1000, 100000);
    game_board.0.iter().for_each(|i| println!("{:?}", i));
    print!("{:?}", game_board.1);
}

#[test]
fn laymine_solvable_works() {
    // 测试筛选法无猜埋雷
    let game_board = laymine_solvable(8, 8, 20, 0, 0, 0, 1000, 100000);
    game_board.0.iter().for_each(|i| println!("{:?}", i));
    print!("{:?}", game_board.1);
}

#[test]
fn laymine_solvable_adjust_works() {
    // 测试筛选法无猜埋雷
    let game_board = laymine_solvable_adjust(8, 8, 20, 0, 0);
    game_board.0.iter().for_each(|i| println!("{:?}", i));
    print!("{:?}", game_board.1);
}

