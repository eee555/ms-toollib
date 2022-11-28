// 测试录像分析模块
use ms_toollib::{AvfVideo, BaseVideo, EvfVideo, MinesweeperBoard, RmvVideo};

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
    println!("time：{:?}", video.data.game_dynamic_params.rtime);
    println!("time_ms：{:?}", video.data.game_dynamic_params.rtime_ms);
    println!("is win: {:?}", video.data.is_completed);
    println!("STNB: {:?}", video.data.video_dynamic_params.stnb);
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
    println!("time：{:?}", video.data.game_dynamic_params.rtime);
    println!("time_ms：{:?}", video.data.game_dynamic_params.rtime_ms);
    println!("is win: {:?}", video.data.is_completed);
    println!("STNB: {:?}", video.data.video_dynamic_params.stnb);
    // video.analyse_for_features(vec!["super_fl_local", "mouse_trace"]);
    // video.data.analyse_for_features(vec!["jump_judge", "survive_poss"]);
    // video.data.print_comments();
}

#[test]
fn EvfVideo_works() {
    let board = vec![
        vec![1, 1, 2, 1, 1, 0, 0, 0],
        vec![1, -1, 2, -1, 1, 0, 0, 0],
        vec![1, 1, 2, 1, 1, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1, 1, 1],
        vec![0, 0, 0, 0, 0, 1, -1, 1],
    ];
    let mut video = BaseVideo::new_with_data(
        "元3",
        board,
        vec![
            (0.0, "pf", 16, 16),
            (0.01, "rc", 16, 48),
            (0.02, "rr", 16, 48),
            (0.03, "lc", 16, 32),
            (0.04, "lr", 16, 32),
            (0.05, "lc", 48, 0),
            (0.06, "lr", 48, 0),
            (0.1, "lc", 16, 32),
            (0.12, "rc", 16, 32),
            (0.14, "rr", 16, 32),
            (0.5, "lr", 16, 32),
            (5.2, "lc", 0, 16),
            (5.8, "rc", 0, 16),
            (6.0, "rr", 0, 16),
            (7.0, "lr", 0, 16),
            (8.0, "lc", 80, 112),
            (9.0, "lr", 80, 112),
        ],
        9.0,
        "eee555",
        "金羊杯GS101",
        "18201",
        "2022.11.22.12.45.30:256478",
        "2022.11.22.12.45.39:256478",
        "CN",
        6,
        0,
        16,
        true,
        true,
        true,
        "00000000000000000000000000000088",
    );
    video.generate_evf_v0_raw_data();
    video.print_raw_data(200);
    video.save_to_evf_file("temp");
    println!("软件：{:?}", video.software);
    println!("标识：{:?}", video.player_designator);
    println!("比赛标识：{:?}", video.race_designator);

    let mut video_read = EvfVideo::new("temp.evf");
    let r = video_read.parse_video();
    video_read.data.analyse();
    println!("结果：{:?}", r);
    println!("软件：{:?}", video_read.data.software);
    println!("标识：{:?}", video_read.data.player_designator);
    println!("比赛标识：{:?}", video_read.data.race_designator);
    println!("3BV：{:?}", video_read.data.static_params.bbbv);
    println!("宽度：{:?}", video_read.data.width);
    println!("高度：{:?}", video_read.data.height);
    println!("雷数：{:?}", video_read.data.mine_num);
    // println!("雷数：{:?}", 'å' as u16);
    println!(
        "time：{:?}s, {:?}ms",
        video_read.data.game_dynamic_params.rtime, video_read.data.game_dynamic_params.rtime_ms
    );
    println!("局面：{:?}", video_read.data.board);
    println!("是否扫完: {:?}", video_read.data.is_completed);
    println!("STNB: {:?}", video_read.data.video_dynamic_params.stnb);
    println!("校验码: {:?}", video_read.data.checksum);
    println!("国家: {:?}", video_read.data.country);

    // println!("{:?}", video.mine_num);
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
