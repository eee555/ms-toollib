use ms_toollib::{videos::NewSomeVideo, EvfVideo};

#[test]
// cargo test --features rs -- --nocapture evf_video_works_v3
fn pluck_works_1() {
    let mut video = EvfVideo::new("../test_files/c_10_1184.575_1021_0.862_Pu Tian Yi(Hu Bei).evf");

    let _ = video.parse();
    video.data.analyse();
    video.data.analyse_for_features(&vec!["pluck"]);
    video.data.set_current_time(99999.9);

    // ENUM: 3.961835438041647
    assert_eq!(video.data.get_pluck().unwrap(), 4.65781553494747);
}
