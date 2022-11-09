// 测试录像分析模块
use ms_toollib::{AvfVideo, MinesweeperBoard, RmvVideo};

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
    my_board
        .step_flow(vec![
            ("lr", (0, 2)),
        ])
        .unwrap();
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
    println!("标识：{:?}", video.data.player);
    println!("3BV：{:?}", video.data.static_params.bbbv);
    // println!("3BV：{:?}", video.s.s);
    println!("time：{:?}", video.data.dynamic_params.r_time);
    println!("time_ms：{:?}", video.data.dynamic_params.r_time_ms);
    println!("is win: {:?}", video.data.win);
    println!("STNB: {:?}", video.data.dynamic_params.stnb);
    // video.analyse_for_features(vec!["super_fl_local", "mouse_trace"]);
    video.data.analyse_for_features(vec!["needless_guess", "high_risk_guess", "jump_judge", "survive_poss"]);
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
    println!("标识：{:?}", video.data.player);
    println!("3BV：{:?}", video.data.static_params.bbbv);
    println!("宽度：{:?}", video.data.width);
    println!("高度：{:?}", video.data.height);
    println!("雷数：{:?}", video.data.mine_num);
    // println!("3BV：{:?}", video.s.s);
    println!("time：{:?}", video.data.dynamic_params.r_time);
    println!("is win: {:?}", video.data.win);
    println!("STNB: {:?}", video.data.dynamic_params.stnb);
    // video.analyse_for_features(vec!["super_fl_local", "mouse_trace"]);
    // video.data.analyse_for_features(vec!["jump_judge", "survive_poss"]);
    // video.data.print_comments();
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
    board.iter_mut().for_each(|x| x.iter_mut().for_each(|xx| if *xx > 10 { *xx = 10 }));
    println!("{:?}", board);
}



