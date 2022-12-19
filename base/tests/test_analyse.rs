// 测试录像分析模块
use ms_toollib::{AvfVideo, BaseVideo, EvfVideo, MinesweeperBoard, RmvVideo};
use std::thread;

#[test]
fn minesweeper_board_works() {
    // 局面状态机测试
    // [("lc", (4, 4)), ("lr", (4, 4)), ("rc", (2, 6)), ("rr", (2, 6)), ("lc", (3, 6)), ("cc", (3, 6)), ("rr", (3, 6))]
    let board = vec![
        vec![1, 1, 1, 1, 1, 2, 2, 2],
        vec![1, -1, 1, 2, -1, 3, -1, -1],
        vec![1, 1, 1, 3, -1, 5, 3, 3],
        vec![0, 0, 0, 2, -1, 3, -1, 1],
        vec![0, 0, 0, 1, 1, 3, 2, 2],
        vec![0, 0, 0, 0, 0, 2, -1, 2],
        vec![0, 1, 1, 1, 0, 2, -1, 2],
        vec![0, 1, -1, 1, 0, 1, 1, 1],
    ];
    let mut my_board = MinesweeperBoard::new(board);
    my_board.step_flow(vec![("lr", (0, 2))]).unwrap();
    my_board.board.iter().for_each(|x| println!("{:?}", x));
    my_board.game_board.iter().for_each(|x| println!("{:?}", x));
    println!("{:?}", my_board.game_board_state);
}

#[test]
// cargo test --features rs -- --nocapture AvfVideo_works
fn AvfVideo_works() {
    // 录像解析工具测试
    let mut video = AvfVideo::new("jze.avf");

    let r = video.parse_video();
    println!("结果：{:?}", r);
    video.data.print_event();
    video.data.analyse();
    println!("标识：{:?}", video.data.player_designator);
    println!("3BV：{:?}", video.data.static_params.bbbv);
    // println!("3BV：{:?}", video.s.s);
    println!("time：{:?}", video.data.get_rtime().unwrap());
    println!("time_ms：{:?}", video.data.get_rtime_ms().unwrap());
    println!("is win: {:?}", video.data.is_completed);
    println!("STNB: {:?}", video.data.get_stnb().unwrap());
    // video.analyse_for_features(vec!["super_fl_local", "mouse_trace"]);
    video.data.analyse_for_features(vec![
        "needless_guess",
        "high_risk_guess",
        "jump_judge",
        "survive_poss",
    ]);
    video.data.print_comments();
}

#[test]
// cargo test --features rs -- --nocapture RmvVideo_works
fn RmvVideo_works() {
    // 录像解析工具测试
    let mut video = RmvVideo::new("18175.rmv");

    let r = video.parse_video();
    // video.data.print_event();
    video.data.analyse();
    println!("结果：{:?}", r);
    println!("标识：{:?}", video.data.player_designator);
    println!("3BV：{:?}", video.data.static_params.bbbv);
    println!("宽度：{:?}", video.data.width);
    println!("高度：{:?}", video.data.height);
    println!("雷数：{:?}", video.data.mine_num);
    // println!("3BV：{:?}", video.s.s);
    println!("time：{:?}", video.data.get_rtime().unwrap());
    println!("time_ms：{:?}", video.data.get_rtime_ms().unwrap());
    println!("is win: {:?}", video.data.is_completed);
    println!("STNB: {:?}", video.data.get_stnb().unwrap());
    // video.analyse_for_features(vec!["super_fl_local", "mouse_trace"]);
    // video.data.analyse_for_features(vec!["jump_judge", "survive_poss"]);
    // video.data.print_comments();
}

#[test]
fn BaseVideo_works() {
    let board = vec![
        vec![1, 1, 2, 1, 1, 0, 0, 0],
        vec![1, -1, 2, -1, 1, 0, 0, 0],
        vec![1, 1, 2, 1, 1, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1, 1, 1],
        vec![0, 0, 0, 0, 0, 1, -1, 1],
    ];
    let mut video = BaseVideo::new_before_game(board, 16);
    video.step("pf", (17, 16)).unwrap();
    video.step("rc", (16, 49)).unwrap();
    thread::sleep_ms(200);
    video.step("rr", (16, 50)).unwrap();
    video.step("mv", (48, 51)).unwrap();
    video.step("mv", (42, 48)).unwrap();
    thread::sleep_ms(200);
    video.step("lc", (16, 32)).unwrap();
    thread::sleep_ms(200);
    video.step("lr", (16, 32)).unwrap();
    thread::sleep_ms(888);
    video.step("lc", (52, 0)).unwrap();
    video.step("lr", (53, 0)).unwrap();
    video.step("lc", (16, 32)).unwrap();
    video.step("rc", (16, 32)).unwrap();
    thread::sleep_ms(50);
    video.step("rr", (16, 32)).unwrap();
    thread::sleep_ms(50);
    video.step("lr", (16, 32)).unwrap();
    thread::sleep_ms(50);
    video.step("lc", (0, 16)).unwrap();
    thread::sleep_ms(50);
    video.step("rc", (0, 16)).unwrap();
    thread::sleep_ms(50);
    video.step("rr", (0, 16)).unwrap();
    println!("left_s：{:?}", video.get_left_s());
    thread::sleep_ms(50);
    video.step("lr", (0, 16)).unwrap();
    video.step("lc", (80, 112)).unwrap();
    video.step("lr", (80, 112)).unwrap();
    video.print_event();

    println!("局面：{:?}", video.get_game_board());
    println!("标识：{:?}", video.player_designator);
    println!("局面状态：{:?}", video.game_board_state);
    println!("3BV：{:?}", video.get_bbbv_solved());
    println!("宽度：{:?}", video.get_ce());
    println!("高度：{:?}", video.height);
    println!("雷数：{:?}", video.mine_num);
    println!("time：{:?}", video.get_rtime());
    println!("time_ms：{:?}", video.get_rtime_ms());
    println!("is win: {:?}", video.is_completed);
    println!("STNB: {:?}", video.get_stnb());
    println!("start_time: {:?}", video.start_time);
    println!("end_time: {:?}", video.end_time);
    println!("path: {:?}", video.get_path());
    println!("etime: {:?}", video.get_etime());
    println!("op: {:?}", video.static_params.op);
    println!("cell0: {:?}", video.static_params.cell0);
    


}

#[test]
fn temp() {
    let mut board = vec![
        vec![-1, 8, 2, 1, 0, 0, 1, -1],
        vec![-1, 5, -8, 1, 0, 0, 2, 2],
        vec![-1, 3, 1, 1, 888, 1, 3, -1],
        vec![1, 1, 0, 0, 0, 1, -1, -1],
        vec![0, 777, 0, 0, 0, -1, 2, 2],
        vec![0, 0, 0, 0, 1, 1, 1, 0],
        vec![0, 0, 0, 0, 999, -1, 1, 0],
        vec![0, 0, 0, 0, 1, 1, 1, 0],
    ];
    board.iter_mut().for_each(|x| {
        x.iter_mut().for_each(|xx| {
            if *xx > 10 {
                *xx = 10
            }
        })
    });
    println!("{:?}", board);
}
