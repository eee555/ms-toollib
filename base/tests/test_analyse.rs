// 测试录像分析模块
use ms_toollib::videos::base_video::{NewBaseVideo, NewBaseVideo2};
use ms_toollib::videos::{NewSomeVideo};
use ms_toollib::{AvfVideo, BaseVideo, EvfVideo, MinesweeperBoard, MvfVideo, RmvVideo, SafeBoard};
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
    let mut my_board = MinesweeperBoard::<Vec<Vec<i32>>>::new(board);
    my_board.step_flow(vec![("rc", (4, 1))]).unwrap();
    my_board.step_flow(vec![("rr", (4, 1))]).unwrap();
    my_board.step_flow(vec![("lc", (5, 1))]).unwrap();
    my_board.step_flow(vec![("lr", (5, 1))]).unwrap();
    my_board.step_flow(vec![("rc", (4, 1))]).unwrap();
    my_board.step_flow(vec![("rr", (4, 1))]).unwrap();
    my_board.step_flow(vec![("lc", (4, 1))]).unwrap();
    my_board.step_flow(vec![("lr", (4, 1))]).unwrap();
    my_board.board.iter().for_each(|x| println!("{:?}", x));
    my_board.game_board.iter().for_each(|x| println!("{:?}", x));
    println!("{:?}", my_board.game_board_state);
    println!("bbbv_solved:{:?}", my_board.bbbv_solved);
}

#[test]
// cargo test --features rs -- --nocapture AvfVideo_works
fn AvfVideo_works() {
    // 录像解析工具测试
    let mut video = AvfVideo::new("wjn.avf");

    let r = video.parse_video();
    println!("结果：{:?}", r);
    video.data.print_event();
    video.data.analyse();
    println!("标识：{:?}", video.data.player_identifier);
    println!("局面：{:?}", video.data.board);
    video.data.set_current_time(0.0);
    // println!("game_board_stream：{:?}", video.data.game_board_stream[0]);
    // println!("局面：{:?}", video.data.get_game_board());
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
    println!("局面: {:?}", video.data.get_game_board());
    println!("solved_3BV：{:?}", video.data.get_bbbv_solved());
    println!("3BV/s：{:?}", video.data.get_bbbv_s());
    println!("thrp: {:?}", video.data.get_thrp());
    println!("level: {:?}", video.data.level);
    println!("is_valid: {:?}", video.data.is_valid());
    println!("get_right: {:?}", video.data.get_right());
    println!("get_flag: {:?}", video.data.get_flag());
    println!("get_left: {:?}", video.data.get_left());
    println!("get_double: {:?}", video.data.get_double());
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
    println!("标识：{:?}", video.data.player_identifier);
    println!("3BV：{:?}", video.data.static_params.bbbv);
    println!("宽度：{:?}", video.data.width);
    println!("高度：{:?}", video.data.height);
    println!("雷数：{:?}", video.data.mine_num);
    println!("level：{:?}", video.data.level);
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
    // video.data.is_valid();
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
    println!(
        "标识：{:?}",
        String::from_utf8(video.data.player_identifier.clone()).unwrap()
    );
    println!("软件：{:?}", video.data.software);
    println!("race_identifier：{:?}", video.data.race_identifier);
    println!("3BV：{:?}", video.data.static_params.bbbv);
    println!("宽度：{:?}", video.data.width);
    println!("高度：{:?}", video.data.height);
    println!("雷数：{:?}", video.data.mine_num);
    println!("level：{:?}", video.data.level);
    // println!("3BV：{:?}", video.s.s);
    println!("time：{:?}", video.data.get_rtime().unwrap());
    println!("time_ms：{:?}", video.data.get_rtime_ms().unwrap());
    println!(
        "video_start_time: {:?}",
        video.data.get_video_start_time().unwrap()
    );
    println!(
        "video_end_time: {:?}",
        video.data.get_video_end_time().unwrap()
    );
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
// cargo test --features rs -- --nocapture EvfVideo_works
fn EvfVideo_works() {
    // 录像解析工具测试
    let mut video = EvfVideo::new("b_0_2.452_12_4.894_Mao Dun (China).evf");

    let r = video.parse_video();
    println!("board: {:?}", video.data.board);
    println!("cell_pixel_size：{:?}", video.data.cell_pixel_size);
    video.data.print_event();
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
    println!("结果：{:?}", r);
    println!("标识：{:?}", video.data.player_identifier);
    println!("软件：{:?}", video.data.software);
    println!("比较：{:?}", "元3.1.9".as_bytes().to_vec());
    println!("race_identifier：{:?}", video.data.race_identifier);
    println!("3BV：{:?}", video.data.static_params.bbbv);
    println!("宽度：{:?}", video.data.width);
    println!("高度：{:?}", video.data.height);
    println!("雷数：{:?}", video.data.mine_num);
    // println!("3BV：{:?}", video.s.s);
    println!("rtime：{:?}", video.data.get_rtime().unwrap());
    println!("rtime_ms：{:?}", video.data.get_rtime_ms().unwrap());
    println!(
        "video_start_time: {:?}",
        video.data.get_video_start_time().unwrap()
    );
    println!(
        "video_end_time: {:?}",
        video.data.get_video_end_time().unwrap()
    );
    println!("is win: {:?}", video.data.is_completed);
    println!("is_official: {:?}", video.data.is_official);
    println!("is_fair: {:?}", video.data.is_fair);
    println!("is_valid: {:?}", video.data.is_valid());
    video.data.set_current_time(0.001);
    println!("time：{:?}", video.data.get_time());
    println!("STNB: {:?}", video.data.get_stnb().unwrap());
    println!("bbbv_solved: {:?}", video.data.get_bbbv_solved().unwrap());
    video.data.set_current_time(999.999);
    println!("get_right: {:?}", video.data.get_right());
    println!("get_flag: {:?}", video.data.get_flag());
    println!("get_left: {:?}", video.data.get_left());
    println!("get_double: {:?}", video.data.get_double());
    println!("game_board: {:?}", video.data.get_game_board());
    // println!("game_board_poss: {:?}", video.data.get_game_board_poss());
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
        vec![2, 2, 1, 0, 0, 0, 0, 0],
        vec![-1, -1, 2, 0, 0, 1, 1, 1],
        vec![-1, -1, 3, 0, 0, 2, -1, 2],
        vec![-1, -1, 2, 0, 0, 2, -1, 2],
    ];
    let mut video = BaseVideo::<SafeBoard>::new(board, 16);
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
    video
        .set_player_identifier("eee".as_bytes().to_vec())
        .unwrap();
    video
        .set_race_identifier("555".as_bytes().to_vec())
        .unwrap();
    video.set_software("888".as_bytes().to_vec()).unwrap();
    video.set_country("666".as_bytes().to_vec()).unwrap();
    video.print_event();

    println!("局面：{:?}", video.get_game_board());
    println!("标识：{:?}", video.player_identifier);
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
    video.set_checksum([8; 32]).unwrap();
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
    println!("标识：{:?}", video.data.player_identifier);
    println!("局面状态：{:?}", video.data.game_board_state);
    println!("软件：{:?}", video.data.software);
    println!("国家：{:?}", video.data.country);
    println!("race_identifier：{:?}", video.data.race_identifier);
    println!("3BV：{:?}", video.data.static_params.bbbv);
    println!("宽度：{:?}", video.data.width);
    println!("高度：{:?}", video.data.height);
    println!("雷数：{:?}", video.data.mine_num);
    // println!("3BV：{:?}", video.s.s);
    println!("time：{:?}", video.data.get_rtime().unwrap());
    println!("time_ms：{:?}", video.data.get_rtime_ms().unwrap());
    println!(
        "start_time：{:?}",
        String::from_utf8(video.data.start_time.clone()).unwrap()
    );
    println!(
        "end_time：{:?}",
        String::from_utf8(video.data.end_time.clone()).unwrap()
    );
    println!("is win: {:?}", video.data.is_completed);
    video.data.set_current_time(1.9);
    println!("bbbv_solved(1.9s): {:?}", video.data.get_bbbv_solved());
    println!("etime(1.9s): {:?}", video.data.get_etime());
    println!("STNB(1.9s): {:?}", video.data.get_stnb().unwrap());
}

#[test]
fn BaseVideo_works_2() {
    let board = vec![
        vec![0, 0, 0, 0, 1, 1, 1, 0],
        vec![0, 0, 0, 0, 1, -1, 2, 1],
        vec![0, 1, 1, 1, 2, 2, 3, -1],
        vec![0, 1, -1, 2, 2, -1, 2, 1],
        vec![1, 3, 4, -1, 2, 1, 1, 0],
        vec![2, -1, -1, 3, 2, 0, 0, 0],
        vec![-1, 3, 3, -1, 1, 0, 1, 1],
        vec![1, 1, 1, 1, 1, 0, 1, -1],
    ];
    // println!("{:?}", ms_toollib::cal_bbbv(&board));
    let mut my_board = MinesweeperBoard::<Vec<Vec<i32>>>::new(board);
    my_board.step_flow(vec![("lc", (2, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (2, 3))]).unwrap();
    my_board.step_flow(vec![("lc", (0, 3))]).unwrap();
    my_board.step_flow(vec![("cc", (0, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (0, 3))]).unwrap();
    my_board.step_flow(vec![("rr", (0, 3))]).unwrap();
    my_board.step_flow(vec![("rr", (1, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (1, 3))]).unwrap();
    my_board.step_flow(vec![("lc", (1, 3))]).unwrap();
    my_board.step_flow(vec![("cc", (1, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (0, 3))]).unwrap();
    my_board.step_flow(vec![("rr", (0, 3))]).unwrap();
    my_board.step_flow(vec![("rc", (0, 3))]).unwrap();
    my_board.step_flow(vec![("cc", (0, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (0, 3))]).unwrap();
    my_board.step_flow(vec![("rr", (0, 3))]).unwrap();
    my_board.step_flow(vec![("lc", (1, 3))]).unwrap();
    my_board.step_flow(vec![("cc", (1, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (1, 3))]).unwrap();
    my_board.step_flow(vec![("rr", (1, 3))]).unwrap();
    my_board.step_flow(vec![("rc", (1, 3))]).unwrap();
    my_board.step_flow(vec![("rr", (1, 3))]).unwrap();
    my_board.step_flow(vec![("lc", (1, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (1, 3))]).unwrap();
    my_board.step_flow(vec![("lc", (1, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (1, 3))]).unwrap();
    my_board.step_flow(vec![("rc", (1, 3))]).unwrap();
    my_board.step_flow(vec![("rr", (1, 3))]).unwrap();
    my_board.step_flow(vec![("lc", (1, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (1, 3))]).unwrap();
    my_board.step_flow(vec![("lc", (0, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (0, 3))]).unwrap();
    my_board.step_flow(vec![("rc", (0, 3))]).unwrap();
    my_board.step_flow(vec![("rr", (0, 3))]).unwrap();
    my_board.step_flow(vec![("lc", (0, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (0, 3))]).unwrap();
    my_board.step_flow(vec![("lc", (3, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (3, 3))]).unwrap();
    my_board.step_flow(vec![("lc", (3, 4))]).unwrap();
    my_board.step_flow(vec![("lr", (3, 4))]).unwrap();
    my_board.step_flow(vec![("rc", (3, 2))]).unwrap();
    my_board.step_flow(vec![("rr", (3, 2))]).unwrap();
    my_board.step_flow(vec![("lc", (4, 2))]).unwrap();
    my_board.step_flow(vec![("lr", (4, 2))]).unwrap();
    my_board.step_flow(vec![("lc", (4, 2))]).unwrap();
    my_board.step_flow(vec![("cc", (4, 2))]).unwrap();
    my_board.step_flow(vec![("lr", (4, 2))]).unwrap();
    my_board.step_flow(vec![("rr", (4, 2))]).unwrap();
    my_board.step_flow(vec![("rc", (4, 2))]).unwrap();
    my_board.step_flow(vec![("cc", (4, 2))]).unwrap();
    my_board.step_flow(vec![("rr", (4, 2))]).unwrap();
    my_board.step_flow(vec![("lr", (4, 2))]).unwrap();
    my_board.step_flow(vec![("lc", (3, 4))]).unwrap();
    my_board.step_flow(vec![("cc", (3, 4))]).unwrap();
    my_board.step_flow(vec![("rr", (3, 4))]).unwrap();
    my_board.step_flow(vec![("lr", (3, 4))]).unwrap();
    my_board.step_flow(vec![("rc", (3, 2))]).unwrap();
    my_board.step_flow(vec![("rr", (3, 2))]).unwrap();
    my_board.step_flow(vec![("rc", (3, 2))]).unwrap();
    my_board.step_flow(vec![("rr", (3, 2))]).unwrap();
    my_board.step_flow(vec![("rc", (3, 2))]).unwrap();
    my_board.step_flow(vec![("rr", (3, 2))]).unwrap();
    my_board.step_flow(vec![("rc", (3, 2))]).unwrap();
    my_board.step_flow(vec![("rr", (3, 2))]).unwrap();
    my_board.step_flow(vec![("lc", (2, 2))]).unwrap();
    my_board.step_flow(vec![("lr", (3, 2))]).unwrap();
    my_board.step_flow(vec![("lc", (3, 2))]).unwrap();
    my_board.step_flow(vec![("lr", (3, 2))]).unwrap();
    my_board.step_flow(vec![("lc", (3, 2))]).unwrap();
    my_board.step_flow(vec![("lr", (4, 4))]).unwrap();
    my_board.step_flow(vec![("rc", (4, 3))]).unwrap();
    my_board.step_flow(vec![("rr", (4, 3))]).unwrap();
    my_board.step_flow(vec![("rc", (3, 5))]).unwrap();
    my_board.step_flow(vec![("rr", (3, 5))]).unwrap();
    my_board.step_flow(vec![("lc", (3, 4))]).unwrap();
    my_board.step_flow(vec![("cc", (3, 4))]).unwrap();
    my_board.step_flow(vec![("lr", (3, 4))]).unwrap();
    my_board.step_flow(vec![("rr", (3, 4))]).unwrap();
    my_board.step_flow(vec![("lc", (4, 4))]).unwrap();
    my_board.step_flow(vec![("cc", (4, 4))]).unwrap();
    my_board.step_flow(vec![("rr", (4, 4))]).unwrap();
    my_board.step_flow(vec![("lr", (4, 4))]).unwrap();
    my_board.step_flow(vec![("rc", (1, 5))]).unwrap();
    my_board.step_flow(vec![("rr", (1, 5))]).unwrap();
    my_board.step_flow(vec![("lc", (2, 5))]).unwrap();
    my_board.step_flow(vec![("cc", (2, 5))]).unwrap();
    my_board.step_flow(vec![("rr", (2, 5))]).unwrap();
    my_board.step_flow(vec![("lr", (2, 5))]).unwrap();
    my_board.step_flow(vec![("lc", (2, 5))]).unwrap();
    my_board.step_flow(vec![("cc", (2, 6))]).unwrap();
    my_board.step_flow(vec![("rr", (2, 6))]).unwrap();
    my_board.step_flow(vec![("lr", (2, 6))]).unwrap();
    my_board.step_flow(vec![("lc", (0, 5))]).unwrap();
    my_board.step_flow(vec![("lr", (0, 5))]).unwrap();
    my_board.step_flow(vec![("lc", (8, 8))]).unwrap();
    my_board.step_flow(vec![("lr", (8, 8))]).unwrap();
    my_board.step_flow(vec![("lc", (0, 7))]).unwrap();
    my_board.step_flow(vec![("lr", (0, 7))]).unwrap();
    my_board.step_flow(vec![("lc", (7, 3))]).unwrap();
    my_board.step_flow(vec![("lr", (7, 3))]).unwrap();
    my_board.step_flow(vec![("lc", (6, 2))]).unwrap();
    my_board.step_flow(vec![("lr", (6, 2))]).unwrap();
    my_board.step_flow(vec![("lc", (7, 2))]).unwrap();
    my_board.step_flow(vec![("lr", (7, 2))]).unwrap();
    my_board.step_flow(vec![("lc", (5, 0))]).unwrap();
    my_board.step_flow(vec![("lr", (5, 0))]).unwrap();
    my_board.step_flow(vec![("lc", (7, 1))]).unwrap();
    my_board.step_flow(vec![("lr", (7, 1))]).unwrap();
    my_board.step_flow(vec![("lc", (6, 1))]).unwrap();
    my_board.step_flow(vec![("lr", (6, 1))]).unwrap();
    my_board.step_flow(vec![("lc", (7, 0))]).unwrap();
    my_board.step_flow(vec![("lr", (7, 0))]).unwrap();

    println!("game_board_state:{:?}", my_board.game_board_state);
    println!("bbbv_solved:{:?}", my_board.bbbv_solved);
    println!("game_board:{:?}", my_board.game_board);

    //     let mut video = BaseVideo::<Vec<Vec<i32>>>::new_before_game(board, 27);
    //     video.step("lc", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("rc", (65, 95)).unwrap();
    // video.step("cc", (65, 95)).unwrap();
    // video.step("rr", (65, 95)).unwrap();
    // video.step("cc", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("cc", (65, 95)).unwrap();
    // video.step("rr", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("rc", (65, 95)).unwrap();
    // video.step("cc", (65, 95)).unwrap();
    // video.step("rr", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("lc", (65, 95)).unwrap();
    // video.step("cc", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("rr", (65, 95)).unwrap();
    // video.step("lc", (65, 95)).unwrap();
    // video.step("cc", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("rr", (65, 95)).unwrap();
    // video.step("lc", (65, 95)).unwrap();
    // video.step("cc", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("rr", (65, 95)).unwrap();
    // video.step("lc", (65, 95)).unwrap();
    // video.step("cc", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("rr", (65, 95)).unwrap();
    // video.step("lc", (65, 95)).unwrap();
    // video.step("cc", (65, 95)).unwrap();
    // video.step("rr", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("lc", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("lc", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("lc", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("lc", (65, 95)).unwrap();
    // video.step("lr", (65, 95)).unwrap();
    // video.step("rc", (65, 95)).unwrap();
    // video.step("rr", (65, 95)).unwrap();
    // video.step("rc", (65, 95)).unwrap();
    // video.step("rr", (65, 95)).unwrap();
    // video.step("lc", (65, 95)).unwrap();
    // video.step("lr", (59, 95)).unwrap();
    // video.step("lc", (216, 216)).unwrap();
    // video.step("lr", (216, 216)).unwrap();
    // video.step("lc", (216, 216)).unwrap();
    // video.step("lr", (216, 216)).unwrap();
    // video.step("lc", (216, 216)).unwrap();
    // video.step("lr", (216, 216)).unwrap();
    // video.step("lc", (216, 216)).unwrap();
    // video.step("lr", (216, 216)).unwrap();
    // video.step("lc", (216, 216)).unwrap();
    // video.step("lr", (216, 216)).unwrap();
    // video.step("rc", (216, 216)).unwrap();
    // video.step("rr", (216, 216)).unwrap();
    // video.step("lc", (216, 216)).unwrap();
    // video.step("lr", (216, 216)).unwrap();
    // video.step("rc", (216, 216)).unwrap();
    // video.step("rr", (216, 216)).unwrap();
    // video.step("rc", (216, 216)).unwrap();
    // video.step("rr", (216, 216)).unwrap();
    // video.step("lc", (22, 95)).unwrap();
    // video.step("cc", (22, 95)).unwrap();
    // video.step("lr", (22, 95)).unwrap();
    // video.step("rr", (4, 96)).unwrap();
    // video.step("rc", (216, 216)).unwrap();
    // video.step("rr", (216, 216)).unwrap();
    // video.step("rc", (216, 216)).unwrap();
    // video.step("cc", (216, 216)).unwrap();
    // video.step("lr", (216, 216)).unwrap();
    // video.step("rr", (216, 216)).unwrap();
    // // video.step("lc", (216, 216)).unwrap();
    // // video.step("cc", (216, 216)).unwrap();
    // // video.step("rr", (29, 96)).unwrap();
    // // video.step("lr", (29, 96)).unwrap();
    // // video.step("lc", (29, 96)).unwrap();
    // // video.step("cc", (29, 96)).unwrap();
    // // video.step("lr", (19, 96)).unwrap();
    // // video.step("rr", (16, 96)).unwrap();
    // // video.step("rc", (8, 96)).unwrap();
    // // video.step("cc", (8, 96)).unwrap();
    // // video.step("lr", (8, 96)).unwrap();
    // // video.step("rr", (8, 96)).unwrap();
    // // video.step("lc", (43, 96)).unwrap();
    // // video.step("cc", (43, 96)).unwrap();
    // // video.step("lr", (43, 96)).unwrap();
    // // video.step("rr", (43, 96)).unwrap();
    // // video.step("rc", (38, 98)).unwrap();
    // // video.step("rr", (38, 98)).unwrap();
    // // video.step("lc", (38, 98)).unwrap();
    // // video.step("lr", (38, 98)).unwrap();
    // // video.step("lc", (38, 98)).unwrap();
    // // video.step("lr", (38, 98)).unwrap();
    // // video.step("rc", (38, 98)).unwrap();
    // // video.step("rr", (38, 98)).unwrap();
    // // video.step("lc", (38, 98)).unwrap();
    // // video.step("lr", (38, 98)).unwrap();
    // // video.step("lc", (17, 98)).unwrap();
    // // video.step("lr", (17, 98)).unwrap();
    // // video.step("rc", (17, 97)).unwrap();
    // // video.step("rr", (17, 97)).unwrap();
    // // video.step("lc", (17, 97)).unwrap();
    // // video.step("lr", (17, 97)).unwrap();
    // // video.step("lc", (87, 82)).unwrap();
    // // video.step("lr", (87, 82)).unwrap();
    // // video.step("lc", (92, 111)).unwrap();
    // // video.step("lr", (92, 112)).unwrap();
    // // video.step("rc", (93, 69)).unwrap();
    // // video.step("rr", (93, 69)).unwrap();
    // // video.step("lc", (108, 69)).unwrap();
    // // video.step("lr", (115, 69)).unwrap();
    // // video.step("lc", (114, 69)).unwrap();
    // // video.step("cc", (114, 69)).unwrap();
    // // video.step("lr", (114, 69)).unwrap();
    // // video.step("rr", (114, 69)).unwrap();
    // // video.step("rc", (114, 69)).unwrap();
    // // video.step("cc", (114, 69)).unwrap();
    // // video.step("rr", (114, 69)).unwrap();
    // // video.step("lr", (114, 69)).unwrap();
    // // video.step("lc", (81, 111)).unwrap();
    // // video.step("cc", (81, 111)).unwrap();
    // // video.step("rr", (81, 111)).unwrap();
    // // video.step("lr", (81, 112)).unwrap();
    // // video.step("rc", (93, 64)).unwrap();
    // // video.step("rr", (93, 64)).unwrap();
    // // video.step("rc", (93, 64)).unwrap();
    // // video.step("rr", (93, 64)).unwrap();
    // // video.step("rc", (93, 64)).unwrap();
    // // video.step("rr", (93, 64)).unwrap();
    // // video.step("rc", (92, 66)).unwrap();
    // // video.step("rr", (92, 66)).unwrap();
    // // video.step("lc", (73, 68)).unwrap();
    // // video.step("lr", (104, 72)).unwrap();
    // // video.step("lc", (104, 72)).unwrap();
    // // video.step("lr", (104, 72)).unwrap();
    // // video.step("lc", (90, 71)).unwrap();
    // // video.step("lr", (118, 123)).unwrap();
    // // video.step("rc", (117, 95)).unwrap();
    // // video.step("rr", (117, 95)).unwrap();
    // // video.step("rc", (85, 144)).unwrap();
    // // video.step("rr", (86, 144)).unwrap();
    // // video.step("lc", (88, 119)).unwrap();
    // // video.step("cc", (88, 119)).unwrap();
    // // video.step("lr", (91, 119)).unwrap();
    // // video.step("rr", (91, 119)).unwrap();
    // // video.step("lc", (111, 115)).unwrap();
    // // video.step("cc", (117, 115)).unwrap();
    // // video.step("rr", (122, 115)).unwrap();
    // // video.step("lr", (122, 115)).unwrap();
    // // video.step("rc", (39, 148)).unwrap();
    // // video.step("rr", (39, 148)).unwrap();
    // // video.step("lc", (61, 146)).unwrap();
    // // video.step("cc", (61, 146)).unwrap();
    // // video.step("rr", (62, 146)).unwrap();
    // // video.step("lr", (62, 146)).unwrap();
    // // video.step("lc", (65, 161)).unwrap();
    // // video.step("cc", (66, 163)).unwrap();
    // // video.step("rr", (66, 166)).unwrap();
    // // video.step("lr", (61, 176)).unwrap();
    // // video.step("lc", (16, 145)).unwrap();
    // // video.step("lr", (16, 145)).unwrap();
    // // video.step("lc", (216, 216)).unwrap();
    // // video.step("lr", (216, 216)).unwrap();
    // // video.step("lc", (8, 198)).unwrap();
    // // video.step("lr", (8, 198)).unwrap();
    // // video.step("lc", (193, 91)).unwrap();
    // // video.step("lr", (194, 92)).unwrap();
    // // video.step("lc", (173, 69)).unwrap();
    // // video.step("lr", (173, 69)).unwrap();
    // // video.step("lc", (195, 68)).unwrap();
    // // video.step("lr", (206, 68)).unwrap();
    // // video.step("lc", (150, 24)).unwrap();
    // // video.step("lr", (150, 24)).unwrap();
    // // video.step("lc", (198, 49)).unwrap();
    // // video.step("lr", (199, 49)).unwrap();
    // // video.step("lc", (176, 41)).unwrap();
    // // video.step("lr", (176, 41)).unwrap();
    // // video.step("lc", (200, 13)).unwrap();
    // // video.step("lr", (200, 13)).unwrap();

    //     println!("{:?}", video.minesweeper_board.mouse_state);
    //     println!("{:?}", video.game_board_state);
    //     println!("{:?}", video.get_game_board());
    //     println!("bbbv: {:?}", video.static_params.bbbv);
    //     println!("get_bbbv_solved: {:?}", video.get_bbbv_solved());
}

#[test]
fn BaseVideo_works_3() {
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
    let mut video = BaseVideo::<SafeBoard>::new(board, 16);
    video.step("rc", (128, 128)).unwrap();
    println!("{:?}", video.minesweeper_board.mouse_state);
    println!("{:?}", video.game_board_state);
    video.step("rr", (128, 128)).unwrap();
    println!("{:?}", video.minesweeper_board.mouse_state);
    println!("{:?}", video.game_board_state);
    video.step("rc", (128, 128)).unwrap();
    println!("{:?}", video.minesweeper_board.mouse_state);
    println!("{:?}", video.game_board_state);
    video.step("rr", (128, 128)).unwrap();
    println!("{:?}", video.minesweeper_board.mouse_state);
    println!("{:?}", video.game_board_state);
}

#[test]
fn BaseVideo_works_4() {
    let board = vec![
        vec![0, 0, 2, -1, 2, 0, 0, 0],
        vec![0, 0, 3, -1, 3, 0, 0, 0],
        vec![0, 0, 2, -1, 2, 0, 0, 0],
        vec![0, 0, 1, 1, 1, 1, 1, 1],
        vec![0, 0, 0, 0, 0, 1, -1, -1],
        vec![1, 1, 0, 0, 0, 1, 2, -1],
        vec![-1, 3, 1, 0, 0, 0, 2, -1],
        vec![-1, -1, 1, 0, 0, 0, 2, -1],
    ];
    let mut video = BaseVideo::<SafeBoard>::new(board, 16);
    thread::sleep_ms(600);
    // println!("3BV：{:?}", video.static_params.bbbv);
    video.step("rc", (32, 49)).unwrap();
    video.step("rr", (32, 49)).unwrap();
    thread::sleep_ms(20);
    video.step("lc", (48, 64)).unwrap();
    thread::sleep_ms(20);
    video.step("lr", (48, 64)).unwrap();
    thread::sleep_ms(20);
    video.step("lc", (48, 64)).unwrap();
    thread::sleep_ms(20);
    video.step("rc", (48, 64)).unwrap();
    thread::sleep_ms(20);
    video.step("lr", (48, 64)).unwrap();
    thread::sleep_ms(20);
    video.step("rr", (48, 64)).unwrap();

    println!("局面：{:?}", video.get_game_board());
    println!("标识：{:?}", video.player_identifier);
    println!("局面状态：{:?}", video.game_board_state);

    video
        .set_player_identifier("eee".as_bytes().to_vec())
        .unwrap();
    video
        .set_race_identifier("555".as_bytes().to_vec())
        .unwrap();
    video.set_software("888".as_bytes().to_vec()).unwrap();
    video.set_country("666".as_bytes().to_vec()).unwrap();
    video.print_event();

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
    video.set_checksum([8; 32]).unwrap();
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
    println!("标识：{:?}", video.data.player_identifier);
    println!("局面状态：{:?}", video.data.game_board_state);
    println!("软件：{:?}", video.data.software);
    println!("国家：{:?}", video.data.country);
    println!("race_identifier：{:?}", video.data.race_identifier);
    println!("3BV：{:?}", video.data.static_params.bbbv);
    println!("宽度：{:?}", video.data.width);
    println!("高度：{:?}", video.data.height);
    println!("雷数：{:?}", video.data.mine_num);
    // println!("3BV：{:?}", video.s.s);
    println!("time：{:?}", video.data.get_rtime().unwrap());
    println!("time_ms：{:?}", video.data.get_rtime_ms().unwrap());
    println!(
        "start_time：{:?}",
        String::from_utf8(video.data.start_time.clone()).unwrap()
    );
    println!(
        "end_time：{:?}",
        String::from_utf8(video.data.end_time.clone()).unwrap()
    );
    println!("is win: {:?}", video.data.is_completed);
    video.data.set_current_time(1.9);
    println!("bbbv_solved(1.9s): {:?}", video.data.get_bbbv_solved());
    println!("etime(1.9s): {:?}", video.data.get_etime());
    println!("STNB(1.9s): {:?}", video.data.get_stnb().unwrap());
}

#[test]
fn BaseVideo_works_5_1bv() {
    let board = vec![
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 1, 1],
        vec![0, 0, 0, 0, 0, 0, 1, -1],
    ];
    let mut video = BaseVideo::<SafeBoard>::new(board, 16);

    // println!("3BV：{:?}", video.static_params.bbbv);
    // video.step("lc", (97, 97)).unwrap();
    // video.step("lr", (97, 97)).unwrap();
    // thread::sleep_ms(60);
    video.step("lc", (32, 49)).unwrap();
    thread::sleep_ms(200);
    video.step("lr", (32, 49)).unwrap();
    video.generate_evf_v0_raw_data();
    video.set_checksum([8; 32]).unwrap();
    video.save_to_evf_file("test");

    println!("局面：{:?}", video.get_game_board());
    println!("标识：{:?}", video.player_identifier);
    println!("局面状态：{:?}", video.game_board_state);
    println!("开始时间戳：{:?}", video.start_time);
    println!("结束时间戳：{:?}", video.end_time);
    println!("时间：{:?}", video.get_rtime());
    println!("时间毫秒：{:?}", video.get_rtime_ms());
    println!("时间毫秒：{:?}", video.get_bbbv_s());
}

#[test]
fn BaseVideo_works_set_board() {
    let board = vec![
        vec![1, -1, 3, -1, 1, 0, 1, -1],
        vec![2, 3, -1, 2, 1, 1, 2, 2],
        vec![-1, 3, 2, 1, 0, 1, -1, 1],
        vec![2, -1, 1, 0, 0, 1, 1, 1],
        vec![2, 3, 3, 1, 0, 0, 0, 0],
        vec![1, -1, -1, 1, 0, 0, 0, 0],
        vec![1, 2, 3, 2, 1, 0, 0, 0],
        vec![0, 0, 1, -1, 1, 0, 0, 0],
    ];

    let board2 = vec![
        vec![1, -1, 3, -1, 1, 0, 1, -1],
        vec![2, 3, -1, 2, 1, 1, 2, 2],
        vec![-1, 3, 2, 1, 0, 1, -1, 1],
        vec![2, -1, 1, 0, 0, 1, 1, 1],
        vec![2, 3, 3, 1, 0, 0, 0, 0],
        vec![1, -1, -1, 1, 0, 0, 0, 0],
        vec![1, 2, 3, 2, 1, 0, 0, 0],
        vec![0, 0, 1, -1, 1, 0, 0, 0],
    ];

    let mut video = BaseVideo::<SafeBoard>::new(board, 42);
    video.set_mode(9).unwrap();
    video.step("lc", (163, 210)).unwrap();
    video.step("lr", (163, 210)).unwrap();
    video.step("rc", (113, 99)).unwrap();
    video.step("rr", (115, 86)).unwrap();
    video.step("rc", (321, 133)).unwrap();
    video.step("rr", (310, 159)).unwrap();
    video.step("rc", (281, 229)).unwrap();
    video.step("rr", (273, 239)).unwrap();
    video.step("rc", (185, 266)).unwrap();
    video.step("rr", (126, 255)).unwrap();
    video.step("rc", (58, 88)).unwrap();
    video.step("rr", (58, 82)).unwrap();
    video.step("rc", (84, 43)).unwrap();
    video.step("rr", (133, 37)).unwrap();
    video.step("lc", (164, 149)).unwrap();
    video.step("lr", (163, 151)).unwrap();
    video.step("lc", (187, 180)).unwrap();
    video.step("lr", (201, 180)).unwrap();
    video.step("lc", (190, 98)).unwrap();
    video.step("lr", (182, 84)).unwrap();
    video.step("lc", (232, 54)).unwrap();

    video.set_board(board2).unwrap();

    // video.generate_evf_v0_raw_data();
    // video.set_checksum([8; 32]).unwrap();
    // video.save_to_evf_file("test");

    println!("局面：{:?}", video.get_game_board());
    println!("局面状态：{:?}", video.game_board_state);
}

#[test]
fn BaseVideo_works_err() {
    let board = vec![
        vec![
            1, 1, 1, 1, 2, 2, -1, 1, 1, 1, 1, 0, 0, 1, -1, 1, 2, -1, 2, 0, 0, 0, 0, 1, 1, 1, 1, -1,
            1, 0,
        ],
        vec![
            -1, 1, 1, -1, 3, -1, 2, 2, 2, -1, 2, 1, 0, 1, 1, 1, 2, -1, 2, 0, 0, 0, 0, 1, -1, 2, 2,
            3, 2, 1,
        ],
        vec![
            2, 2, 2, 2, -1, 3, 3, 3, -1, 4, -1, 1, 0, 1, 2, 2, 3, 3, 3, 1, 1, 1, 1, 2, 2, 3, -1, 2,
            -1, 1,
        ],
        vec![
            1, -1, 1, 1, 1, 2, -1, -1, 3, -1, 2, 1, 1, 2, -1, -1, 2, -1, -1, 1, 2, -1, 2, 1, -1, 2,
            1, 3, 2, 2,
        ],
        vec![
            1, 1, 1, 0, 0, 1, 2, 2, 2, 1, 1, 1, 2, -1, 3, 2, 2, 2, 3, 2, 3, -1, 2, 2, 2, 2, 1, 2,
            -1, 1,
        ],
        vec![
            0, 1, 1, 2, 1, 1, 0, 0, 0, 0, 0, 1, -1, 2, 1, 1, 1, 2, 2, -1, 2, 1, 2, 2, -1, 1, 1, -1,
            2, 1,
        ],
        vec![
            2, 3, -1, 3, -1, 2, 1, 2, 2, 2, 1, 2, 1, 2, 1, 2, -1, 2, -1, 2, 2, 1, 2, -1, 2, 1, 1,
            2, 2, 1,
        ],
        vec![
            -1, -1, 2, 3, -1, 2, 1, -1, -1, 3, -1, 2, 1, 2, -1, 2, 1, 3, 2, 2, 2, -1, 4, 2, 3, 2,
            2, 2, -1, 1,
        ],
        vec![
            2, 2, 1, 1, 1, 1, 1, 2, 3, -1, 3, 3, -1, 4, 3, 2, 0, 2, -1, 2, 2, -1, 3, -1, 2, -1, -1,
            2, 1, 1,
        ],
        vec![
            0, 1, 2, 3, 2, 2, 1, 1, 2, 3, 4, -1, 3, -1, -1, 1, 0, 2, -1, 3, 2, 2, 3, 2, 3, 2, 2, 1,
            1, 1,
        ],
        vec![
            1, 3, -1, -1, -1, 3, -1, 1, 1, -1, -1, 3, 3, 3, 3, 2, 0, 1, 1, 2, -1, 1, 1, -1, 1, 0,
            0, 0, 1, -1,
        ],
        vec![
            3, -1, -1, 8, -1, 4, 1, 1, 1, 2, 3, -1, 1, 1, -1, 1, 0, 0, 0, 1, 1, 1, 1, 2, 3, 2, 2,
            1, 2, 1,
        ],
        vec![
            -1, -1, -1, -1, -1, 3, 1, 0, 1, 2, 3, 2, 2, 2, 2, 1, 0, 0, 0, 1, 1, 2, 1, 2, -1, -1, 2,
            -1, 2, 1,
        ],
        vec![
            3, 4, 4, 3, 4, -1, 2, 0, 2, -1, -1, 1, 2, -1, 2, 0, 0, 0, 0, 1, -1, 2, -1, 3, 3, 3, 3,
            3, -1, 1,
        ],
        vec![
            1, -1, 1, 0, 3, -1, 3, 0, 2, -1, 4, 2, 4, -1, 5, 2, 2, 1, 1, 1, 1, 2, 1, 2, -1, 2, 2,
            -1, 2, 1,
        ],
        vec![
            1, 1, 1, 0, 2, -1, 2, 0, 1, 1, 2, -1, 3, -1, -1, -1, 2, -1, 1, 0, 0, 0, 0, 1, 1, 2, -1,
            2, 1, 0,
        ],
    ];
    let mut video = BaseVideo::<SafeBoard>::new(board, 24);
    video.set_mode(9).unwrap();

    video.step("rr", (157, 139)).unwrap();

    // video.generate_evf_v0_raw_data();
    // video.set_checksum([8; 32]).unwrap();
    // video.save_to_evf_file("test");

    println!("局面：{:?}", video.get_game_board());
    println!("局面状态：{:?}", video.game_board_state);
}
