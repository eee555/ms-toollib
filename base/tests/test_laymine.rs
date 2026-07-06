// use ms_toollib::refresh_matrixs;
#[cfg(any(feature = "py", feature = "rs"))]
use ms_toollib::laymine_solvable_thread;
use ms_toollib::{
    is_solvable, laymine, laymine_op, laymine_solvable, laymine_solvable_adjust, try_solve,
};
use std::time::Instant; // timer

// 测试各种埋雷类的函数

#[test]
#[cfg(any(feature = "py", feature = "rs"))]
fn laymine_solvable_thread_works() {
    // 测试多线程筛选法无猜埋雷
    let start = Instant::now();
    for _ in 0..10 {
        let _game_board = laymine_solvable_thread(16, 30, 99, 0, 0, 100000);
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

fn print_matrix(matrix: &Vec<Vec<i32>>) {
    println!("[");
    for row in matrix {
        for num in row {
            // 使用格式化字符串将数字格式化为宽度为 4 的字符串
            print!("{:4}", num);
        }
        // 每一行结束后换行
        println!();
    }
    println!("]");
}

#[test]
// cargo test --package ms_toollib --test test_laymine -- laymine_solvable_adjust_works --exact --show-output --nocapture
fn laymine_solvable_adjust_works() {
    // 测试调整法无猜埋雷
    let board = laymine_solvable_adjust(16, 30, 170, 0, 0);
    // board.0.iter().for_each(|i| println!("{:?}", i));
    print_matrix(&board.0);
    if board.1 {
        if is_solvable(&board.0, 0, 0) {
            print!("成功！！！");
        } else {
            println!("失败！！！");
            let (board_end, _bbbv_solved) = try_solve(&board.0, 0, 0);
            print_matrix(&board_end);
        }
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

#[test]
#[ignore]
fn laymine_solvable_adjust_high_density() {
    const ROW: usize = 16;
    const COL: usize = 30;
    const MINES: usize = 320;
    const X0: usize = 7;
    const Y0: usize = 8;
    const ITERATIONS: usize = 100;

    let mut success_count = 0;
    let mut total_mines_correct = 0;
    let mut start_safe = 0;

    for i in 0..ITERATIONS {
        println!("第{}次", i);
        let (board, flag) = laymine_solvable_adjust(ROW, COL, MINES, X0, Y0);

        if board[X0][Y0] != -1 {
            start_safe += 1;
        } else {
            println!("第{}次: 起手位置是雷！", i);
        }

        let mine_count = board.iter().flatten().filter(|&&v| v == -1).count();
        if mine_count == MINES {
            total_mines_correct += 1;
        } else {
            println!("第{}次: 雷数不对 {} != {}", i, mine_count, MINES);
        }

        if flag && is_solvable(&board, X0, Y0) {
            success_count += 1;
        }
    }

    println!(
        "高密度测试结果: 起手安全 {}/{}, 雷数正确 {}/{}, 无猜通过 {}/{}",
        start_safe, ITERATIONS, total_mines_correct, ITERATIONS, success_count, ITERATIONS
    );

    assert_eq!(start_safe, ITERATIONS, "起手位置不应为雷");
    assert_eq!(total_mines_correct, ITERATIONS, "总雷数必须等于传入值");
}
