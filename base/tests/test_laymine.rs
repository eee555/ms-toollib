// use ms_toollib::refresh_matrixs;
use ms_toollib::{laymine_solvable, laymine_solvable_adjust, laymine_solvable_thread};

// 测试各种埋雷类的函数

#[test]
fn laymine_solvable_thread_works() {
    // 测试多线程筛选法无猜埋雷
    let game_board = laymine_solvable_thread(16, 30, 99, 0, 0, 100000);
    game_board.0.iter().for_each(|i| println!("{:?}", i));
    print!("{:?}", game_board.1);
}

#[test]
fn laymine_solvable_works() {
    // 测试筛选法无猜埋雷
    let game_board = laymine_solvable(16, 30, 99, 0, 0, 100000);
    game_board.0.iter().for_each(|i| println!("{:?}", i));
    print!("{:?}", game_board.1);
}

#[test]
fn laymine_solvable_adjust_works() {
    // 测试调整法无猜埋雷
    let game_board = laymine_solvable_adjust(8, 8, 30, 0, 0);
    game_board.0.iter().for_each(|i| println!("{:?}", i));
    if game_board.1 {
        print!("成功！！！");
    } else {
        print!("失败！！！");
    }
}
