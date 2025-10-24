// 测试录像分析模块
use ms_toollib::videos::base_video::NewBaseVideo2;
use ms_toollib::videos::NewSomeVideo;
use ms_toollib::{BaseVideo, EvfVideo, Evfs, SafeBoard};
use std::mem::transmute;
use std::time::Duration;
use std::{thread, vec};

fn _sleep_ms(ms: u32) {
    thread::sleep(Duration::from_millis(ms as u64));
}

#[test]
fn evfs_save_works() {
    let mut evfs = Evfs::new();

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
    println!("left_s：{:?}", video.get_left_s());
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
    let check_sum = vec![8; 32];
    video.set_checksum_evf_v4(check_sum.clone()).unwrap();
    evfs.append(video.get_raw_data().unwrap(), "test_1", check_sum);



    // 第2盘，失败
    let board = vec![
        vec![1,  1, 2,  1, 1],
        vec![1, -1, 2, -1, 1],
        vec![1,  1, 2,  1, 1],
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
    let check_sum = vec![8; 32];
    video.set_checksum_evf_v4(check_sum.clone()).unwrap();
    evfs.append(video.get_raw_data().unwrap(), "test_1", check_sum);



}
