// use ms_toollib::refresh_matrixs;
#[cfg(any(feature = "py", feature = "rs"))]
use ms_toollib::laymine_solvable_thread;
use ms_toollib::{
    laymine, laymine_op, laymine_solvable, laymine_solvable_adjust,
};
use std::time::Instant; // timer

// 测试各种埋雷类的函数

#[test]
#[cfg(any(feature = "py", feature = "rs"))]
fn laymine_solvable_thread_works() {
    // 测试多线程筛选法无猜埋雷
    let start = Instant::now();
    for _ in 0..10 {
        let game_board = laymine_solvable_thread(16, 30, 99, 0, 0, 100000);
        // game_board.0.iter().for_each(|i| println!("{:?}", i));
    }
    println!("time cost: {:?} us", start.elapsed().as_micros()); // us
                                                                 // print!("{:?}", game_board.1);
}

#[test]
fn laymine_solvable_works() {
    // 测试筛选法无猜埋雷
    let game_board = laymine_solvable(8, 8, 25, 0, 0, 100000);
    game_board.0.iter().for_each(|i| println!("{:?}", i));
    print!("{:?}", game_board.1);
}

// cargo test -- --nocapture laymine_solvable_adjust_works
// cargo run | head -100
#[test]
fn laymine_solvable_adjust_works() {
    // 测试调整法无猜埋雷
    let game_board = laymine_solvable_adjust(16, 30, 200, 0, 0);
    game_board.0.iter().for_each(|i| println!("{:?}", i));
    if game_board.1 {
        print!("成功！！！");
    } else {
        print!("失败！！！");
    }
}

#[test]
fn laymine_works() {
    print!("{:?}", laymine(8, 8, 57, 1, 3));
}

#[test]
fn laymine_op_works() {
    print!("{:?}", laymine_op(8, 8, 43, 0, 0));
    print!("{:?}", laymine_op(3, 5, 3, 1, 4));
}

