// 测试录像分析模块
use ms_toollib::videos::base_video::NewBaseVideo2;
use ms_toollib::{BaseVideo, Evfs, SafeBoard};
use std::time::Duration;
use std::{thread, vec};

fn _sleep_ms(ms: u32) {
    thread::sleep(Duration::from_millis(ms as u64));
}

#[test]
fn evfs_save_works() {
    let mut evfs = Evfs::new();
    // 第1盘，成果
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
    _sleep_ms(60);
    video.step("rc", (17, 16)).unwrap();
    video.step("rr", (17, 16)).unwrap();
    video.step("rc", (16, 49)).unwrap();
    _sleep_ms(20);
    video.step("rr", (16, 50)).unwrap();
    video.step("lc", (16, 32)).unwrap();
    _sleep_ms(20);
    video.step("lr", (16, 32)).unwrap();
    _sleep_ms(20);
    video.step("lc", (52, 0)).unwrap();
    video.step("lr", (53, 0)).unwrap();
    video.step("lc", (16, 32)).unwrap();
    video.step("rc", (16, 32)).unwrap();
    _sleep_ms(5);
    video.step("rr", (16, 32)).unwrap();
    _sleep_ms(5);
    video.step("lr", (16, 32)).unwrap();
    _sleep_ms(5);
    video.step("lc", (0, 16)).unwrap();
    _sleep_ms(5);
    video.step("rc", (0, 16)).unwrap();
    _sleep_ms(5);
    video.step("rr", (0, 16)).unwrap();
    _sleep_ms(5);
    video.step("lr", (0, 16)).unwrap();
    video.step("mv", (4800, 51)).unwrap();
    video.step("lc", (112, 112)).unwrap();
    video.step("lr", (112, 112)).unwrap();
    video.step("lc", (97, 112)).unwrap();
    video.step("lr", (97, 112)).unwrap();
    video.set_player_identifier("eee555".to_string()).unwrap();
    video.set_race_identifier("G8888".to_string()).unwrap();
    video.set_software("a test software".to_string()).unwrap();
    video.set_country("CN".to_string()).unwrap();

    video.generate_evf_v4_raw_data();
    let check_sum_evf = vec![8; 32];
    video.set_checksum(check_sum_evf).unwrap();
    let check_sum_cell = vec![9; 32];
    evfs.push(video.get_raw_data().unwrap(), "test_1", check_sum_cell);

    // 第2盘，失败
    let board = vec![
        vec![1, 1, 2, 1, 1],
        vec![1, -1, 2, -1, 1],
        vec![1, 1, 2, 1, 1],
        vec![0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0],
    ];
    let mut video = BaseVideo::<SafeBoard>::new(board, 20);
    _sleep_ms(60);
    // println!("3BV：{:?}", video.static_params.bbbv);
    video.step("lc", (7, 7)).unwrap();
    video.step("lr", (7, 7)).unwrap();
    video.step("mv", (48, 51)).unwrap();
    video.step("mv", (20, 77)).unwrap();
    _sleep_ms(20);
    video.step("lc", (20, 21)).unwrap();
    video.step("lr", (21, 25)).unwrap();

    video.generate_evf_v4_raw_data();
    let check_sum_evf = vec![12; 32];
    video.set_checksum(check_sum_evf).unwrap();
    let check_sum_cell = vec![13; 32];
    evfs.push(video.get_raw_data().unwrap(), "test_2", check_sum_cell);

    // 第3盘，成果
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
    _sleep_ms(60);
    video.step("rc", (32, 49)).unwrap();
    video.step("rr", (32, 49)).unwrap();
    _sleep_ms(20);
    video.step("lc", (48, 64)).unwrap();
    _sleep_ms(20);
    video.step("mv", (1, 51)).unwrap();
    video.step("mv", (2, 51)).unwrap();
    video.step("mv", (3, 51)).unwrap();
    video.step("mv", (3, 4)).unwrap();
    video.step("mv", (48, 5)).unwrap();
    video.step("mv", (48, 6)).unwrap();
    video.step("lr", (48, 64)).unwrap();
    _sleep_ms(20);
    video.step("lc", (48, 64)).unwrap();
    _sleep_ms(20);
    video.step("rc", (48, 64)).unwrap();
    _sleep_ms(20);
    video.step("lr", (48, 64)).unwrap();
    _sleep_ms(20);
    video.step("rr", (48, 64)).unwrap();
    video.generate_evf_v4_raw_data();
    let check_sum_cell = vec![15; 32];
    evfs.push(video.get_raw_data().unwrap(), "test_3", check_sum_cell);

    // 生成 EVFS V0 原始数据，保存到文件
    evfs.generate_evfs_v0_raw_data();
    let name = evfs.save_evfs_file("test");
    // test.evfs、test(2).evfs、test(3).evfs ...
    println!("Saved evfs file: {}", name);

    // 重新读取evfs文件，并测试解析
    let mut evfs = Evfs::new_with_file("test.evfs");
    evfs.parse().unwrap();

    let cell1_3 = &evfs[0..3];
    assert_eq!(
        cell1_3[0].evf_video.data.software,
        "a test software".to_string()
    );
    let cell2 = &evfs[1];
    assert!(!cell2.evf_video.data.is_completed);

    evfs.save_evf_files("./");
}

#[test]
fn evfs_save_works_2() {
    // 重新读取evfs文件，并测试解析
    let mut evfs = Evfs::new_with_file("../test_files/[lag]二问题无法_20251113_105617_17.evfs");
    evfs.parse().unwrap();
    evfs.analyse().unwrap();
    evfs.analyse_for_features(&vec![
        "high_risk_guess",
        "jump_judge",
        "needless_guess",
        "mouse_trace",
        "vision_transfer",
        "pluck",
        "super_fl_local",
    ]).expect("feature analysis should work!");
}
