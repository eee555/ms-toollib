use ms_toollib::videos::NewSomeVideo;
use ms_toollib::{
    AvfVideo, EvfVideo,
};
use ms_toollib::Event;

// 测试能正确播放录像
#[test]
// cargo test --features rs -- --nocapture play_video_works_avf
fn play_video_works_avf() {
    let mut video =
        AvfVideo::new("../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf");

    let r = video.parse();
    assert_eq!(r.unwrap(), ());
    // video.data.print_event();
    video.data.analyse();

    for t in 0..100 {
        video.data.set_current_time(t as f64 * 0.1);
        println!("x, y: {:?}", video.data.get_x_y());
    }
}

#[test]
// cargo test --features rs -- --nocapture play_video_works_evf
fn play_video_works_evf() {
    let mut video =
        EvfVideo::new("../test_files/temp.evf");

    let r = video.parse();
    assert_eq!(r.unwrap(), ());
    // video.data.print_event();
    video.data.analyse();

    // for t in 0..37{
    //     let time = t as f64 * 0.1;
    //     video.data.set_current_time(time);
    //     println!("time: {:?}, x, y: {:?}", time, video.data.get_x_y());
    // }
    for t in video.data.video_action_state_recorder {
        if let Some(Event::Mouse(mouse_event)) = &t.event {
            println!("{:?}, {:?}, {:?}, {:?}", mouse_event.mouse, t.time, mouse_event.x, mouse_event.y);
        }
    }
}
