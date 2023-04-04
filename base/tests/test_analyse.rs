// 测试录像分析模块
use ms_toollib::{AvfVideo, BaseVideo, EvfVideo, MinesweeperBoard, RmvVideo, MvfVideo};
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
    my_board.step_flow(vec![("rc", (0, 0))]).unwrap();
    my_board.step_flow(vec![("rr", (0, 0))]).unwrap();
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
    println!("局面：{:?}", video.data.board);
    video.data.set_current_time(0.0);
    // println!("game_board_stream：{:?}", video.data.game_board_stream[0]);
    println!("局面：{:?}", video.data.get_game_board());
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
    video.data.set_current_time(1000.0);
    println!("solved_3BV：{:?}", video.data.get_bbbv_solved());
    println!("thrp{:?}", video.data.get_thrp());
}

#[test]
// cargo test --features rs -- --nocapture RmvVideo_works
fn RmvVideo_works() {
    // 录像解析工具测试
    let mut video = RmvVideo::new("large_path.rmv");

    let r = video.parse_video();
    video.data.print_event();
    video.data.analyse();
    video.data.set_pix_size(60);
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
    video.data.set_current_time(40.0);
    println!("STNB: {:?}", video.data.get_stnb().unwrap());
    println!("path: {:?}", video.data.get_path());
    video.data.set_current_time(-1.0);
    println!("game_board: {:?}", video.data.get_game_board());
    // video.analyse_for_features(vec!["super_fl_local", "mouse_trace"]);
    // video.data.analyse_for_features(vec!["jump_judge", "survive_poss"]);
    // video.data.print_comments();
}

#[test]
fn MvfVideo_works() {
    // 录像解析工具测试
    let mut video = MvfVideo::new("Zhang Shen Jia_Exp_38.82(3bv122).mvf");

    let r = video.parse_video();
    // video.data.print_event();
    video.data.analyse();
    // video.data.analyse_for_features(vec![
    //     "high_risk_guess",
    //     "jump_judge",
    //     "needless_guess",
    //     "mouse_trace",
    //     "vision_transfer",
    //     "survive_poss",
    // ]);

    // video.data.print_raw_data(400);
    println!("board: {:?}", video.data.board);
    println!("结果：{:?}", r);
    println!("标识：{:?}", String::from_utf8(video.data.player_designator.clone()).unwrap());
    println!("软件：{:?}", video.data.software);
    println!("race_designator：{:?}", video.data.race_designator);
    println!("3BV：{:?}", video.data.static_params.bbbv);
    println!("宽度：{:?}", video.data.width);
    println!("高度：{:?}", video.data.height);
    println!("雷数：{:?}", video.data.mine_num);
    // println!("3BV：{:?}", video.s.s);
    println!("time：{:?}", video.data.get_rtime().unwrap());
    println!("time_ms：{:?}", video.data.get_rtime_ms().unwrap());
    println!("video_time: {:?}", video.data.get_video_time().unwrap());
    println!("is win: {:?}", video.data.is_completed);
    video.data.set_current_time(12.0);
    println!("STNB: {:?}", video.data.get_stnb().unwrap());
    println!("game_board: {:?}", video.data.get_game_board());
    println!("game_board_poss: {:?}", video.data.get_game_board_poss());
    // video.analyse_for_features(vec!["super_fl_local", "mouse_trace"]);
    // video.data.analyse_for_features(vec!["jump_judge", "survive_poss"]);
    // video.data.print_comments();
}

#[test]
// cargo test --features rs -- --nocapture RmvVideo_works
fn EvfVideo_works() {
    // 录像解析工具测试
    let mut video = EvfVideo::new("t.evf");

    let r = video.parse_video();
    // video.data.print_event();
    video.data.analyse();
    video.data.analyse_for_features(vec![
        "high_risk_guess",
        "jump_judge",
        "needless_guess",
        "mouse_trace",
        "vision_transfer",
        "survive_poss",
    ]);

    // video.data.print_raw_data(400);
    println!("board: {:?}", video.data.board);
    println!("结果：{:?}", r);
    println!("标识：{:?}", video.data.player_designator);
    println!("软件：{:?}", video.data.software);
    println!("race_designator：{:?}", video.data.race_designator);
    println!("3BV：{:?}", video.data.static_params.bbbv);
    println!("宽度：{:?}", video.data.width);
    println!("高度：{:?}", video.data.height);
    println!("雷数：{:?}", video.data.mine_num);
    // println!("3BV：{:?}", video.s.s);
    println!("time：{:?}", video.data.get_rtime().unwrap());
    println!("time_ms：{:?}", video.data.get_rtime_ms().unwrap());
    println!("video_time: {:?}", video.data.get_video_time().unwrap());
    println!("is win: {:?}", video.data.is_completed);
    video.data.set_current_time(12.0);
    println!("STNB: {:?}", video.data.get_stnb().unwrap());
    println!("game_board: {:?}", video.data.get_game_board());
    println!("game_board_poss: {:?}", video.data.get_game_board_poss());
    // video.analyse_for_features(vec!["super_fl_local", "mouse_trace"]);
    // video.data.analyse_for_features(vec!["jump_judge", "survive_poss"]);
    video.data.print_comments();
}

#[test]
fn BaseVideo_works() {
    let board = vec![
        vec![1, 1, 2, 1, 1, 0, 0, 0],
        vec![1, -1, 2, -1, 1, 0, 0, 0],
        vec![1, 1, 2, 1, 1, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![2, 2, 1, 0, 0, 0, 0, 0],
        vec![-1, -1, 2, 0, 0, 1, 1, 1],
        vec![-1, -1, 3, 0, 0, 2, -1, 2],
        vec![-1, -1, 2, 0, 0, 2, -1, 2],
    ];
    let mut video = BaseVideo::new_before_game(board, 16);
    thread::sleep_ms(600);
    // println!("3BV：{:?}", video.static_params.bbbv);
    video.step("rc", (17, 16)).unwrap();
    video.step("rr", (17, 16)).unwrap();
    video.step("rc", (16, 49)).unwrap();
    thread::sleep_ms(20);
    video.step("rr", (16, 50)).unwrap();
    video.step("mv", (48, 51)).unwrap();
    video.step("mv", (42, 48)).unwrap();
    thread::sleep_ms(20);
    video.step("lc", (16, 32)).unwrap();
    thread::sleep_ms(20);
    video.step("lr", (16, 32)).unwrap();
    thread::sleep_ms(20);
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
    video.step("mv", (4800, 51)).unwrap();
    video.step("lc", (112, 112)).unwrap();
    video.step("lr", (112, 112)).unwrap();
    video.step("lc", (97, 112)).unwrap();
    video.step("lr", (97, 112)).unwrap();
    video.set_player_designator("eee".as_bytes().to_vec()).unwrap();
    video.set_race_designator("555".as_bytes().to_vec()).unwrap();
    video.set_software("888".as_bytes().to_vec()).unwrap();
    video.set_country("666".as_bytes().to_vec()).unwrap();
    video.print_event();

    println!("局面：{:?}", video.get_game_board());
    println!("标识：{:?}", video.player_designator);
    println!("局面状态：{:?}", video.game_board_state);
    println!(
        "3BV：{:?}/{:?}",
        video.get_bbbv_solved(),
        video.static_params.bbbv
    );
    println!("ce：{:?}", video.get_ce());
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

    video.generate_evf_v0_raw_data();
    video.save_to_evf_file("test");

    let mut video = EvfVideo::new("test.evf");
    let r = video.parse_video();
    video.data.print_event();
    // video.data.print_raw_data(400);
    video.data.analyse();
    // video.data.set_current_time(1.9);
    println!("结果：{:?}", r);
    println!("board：{:?}", video.data.board);
    println!("game_board: {:?}", video.data.get_game_board());
    println!("game_board_state: {:?}", video.data.game_board_state);
    println!("标识：{:?}", video.data.player_designator);
    println!("局面状态：{:?}", video.data.game_board_state);
    println!("软件：{:?}", video.data.software);
    println!("国家：{:?}", video.data.country);
    println!("race_designator：{:?}", video.data.race_designator);
    println!("3BV：{:?}", video.data.static_params.bbbv);
    println!("宽度：{:?}", video.data.width);
    println!("高度：{:?}", video.data.height);
    println!("雷数：{:?}", video.data.mine_num);
    // println!("3BV：{:?}", video.s.s);
    println!("time：{:?}", video.data.get_rtime().unwrap());
    println!("time_ms：{:?}", video.data.get_rtime_ms().unwrap());
    println!("start_time：{:?}", String::from_utf8(video.data.start_time.clone()).unwrap());
    println!("end_time：{:?}", String::from_utf8(video.data.end_time.clone()).unwrap());
    println!("is win: {:?}", video.data.is_completed);
    video.data.set_current_time(1.9);
    println!("bbbv_solved(1.9s): {:?}", video.data.get_bbbv_solved());
    println!("etime(1.9s): {:?}", video.data.get_etime());
    println!("STNB(1.9s): {:?}", video.data.get_stnb().unwrap());
}

#[test]
fn BaseVideo_works_2() {
    let board = vec![
        vec![1, 1, 2, 1, 1, 0, 0, 0],
        vec![1, -1, 2, -1, 1, 0, 0, 0],
        vec![1, 1, 2, 1, 1, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![2, 2, 1, 0, 0, 0, 0, 0],
        vec![-1, -1, 2, 0, 0, 1, 1, 1],
        vec![-1, -1, 3, 0, 0, 2, -1, 2],
        vec![-1, -1, 2, 0, 0, 2, -1, 2],
    ];
    let mut video = BaseVideo::new_before_game(board, 16);
    video.step("rc", (17, 16)).unwrap();
    println!("{:?}", video.minesweeper_board.mouse_state);
    println!("{:?}", video.game_board_state);


    video.step("cc", (17, 16)).unwrap();
    println!("{:?}", video.minesweeper_board.mouse_state);
    println!("{:?}", video.game_board_state);

    // thread::sleep_ms(2000);


    video.step("lr", (17, 16)).unwrap();
    // thread::sleep_ms(3000);

    println!("{:?}", video.minesweeper_board.mouse_state);
    println!("{:?}", video.game_board_state);
    video.step("rr", (17, 16)).unwrap();
    println!("44:{:?}", video.minesweeper_board.mouse_state);
    println!("55:{:?}", video.game_board_state);
    // println!("666:{:?}", video.minesweeper_board.game_board);

    video.step("rc", (17, 16)).unwrap();
    println!("66:{:?}", video.minesweeper_board.mouse_state);
    println!("77:{:?}", video.game_board_state);
    video.step("rr", (17, 16)).unwrap();
    println!("88:{:?}", video.minesweeper_board.mouse_state);
    println!("99:{:?}", video.game_board_state);
}



