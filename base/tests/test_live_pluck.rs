use ms_toollib::videos::base_video::NewBaseVideo2;
use ms_toollib::BaseVideo;

#[test]
fn live_pluck_excludes_fatal_click_and_resets() {
    let mut video = BaseVideo::<Vec<Vec<i32>>>::new(vec![vec![1, -1], vec![1, 1]], 16);
    assert!(!video.live_pluck_enabled());
    assert_eq!(video.get_live_pluck(), None);
    assert!(video.live_pluck_snapshot().wait().is_err());
    video.set_live_pluck_enabled(true);
    for (row, col) in [(0, 0), (1, 0), (0, 1)] {
        video.step("lc", (row * 16 + 1, col * 16 + 1)).unwrap();
        video.step("lr", (row * 16 + 1, col * 16 + 1)).unwrap();
    }
    let snapshot = video.live_pluck_snapshot();
    video.reset(2, 2, 16);
    assert!(video.live_pluck_enabled());
    assert_eq!(video.get_live_pluck(), Some(0.0));
    let value = snapshot.wait().unwrap();
    assert!((value + (2.0_f64 / 3.0).log10()).abs() < 1e-10, "{value}");
    assert_eq!(snapshot.wait().unwrap(), value);
    assert_eq!(snapshot.ready().unwrap(), Some(value));
}

#[test]
fn invalid_replacement_board_keeps_live_game_usable() {
    let mut video = BaseVideo::<Vec<Vec<i32>>>::new(vec![vec![1, -1], vec![1, 1]], 16);
    for event in ["lc", "lr"] {
        video.step(event, (1, 1)).unwrap();
    }
    assert!(video.set_board(vec![]).is_err());
    assert!(video.set_board(vec![vec![1, -1], vec![1]]).is_err());
    assert!(video.set_board(vec![vec![1, -1], vec![1, 1, 1]]).is_err());
    for event in ["lc", "lr"] {
        video.step(event, (17, 1)).unwrap();
    }
    assert_eq!(video.video_action_state_recorder.len(), 4);
}

#[test]
fn enabling_mid_game_replays_history_and_disabling_preserves_snapshots() {
    let mut video = BaseVideo::<Vec<Vec<i32>>>::new(vec![vec![1, -1], vec![1, 1]], 16);
    for (row, col) in [(0, 0), (1, 0)] {
        for event in ["lc", "lr"] { video.step(event, (row * 16 + 1, col * 16 + 1)).unwrap(); }
    }
    video.set_live_pluck_enabled(true);
    let snapshot = video.live_pluck_snapshot();
    video.set_live_pluck_enabled(false);
    assert_eq!(video.get_live_pluck(), None);
    assert!((snapshot.wait().unwrap() + (2.0_f64 / 3.0).log10()).abs() < 1e-10);
    video.set_live_pluck_enabled(true);
    assert_eq!(video.live_pluck_snapshot().wait().unwrap(), snapshot.wait().unwrap());
}

#[test]
fn synchronous_pluck_remains_available_without_live_calculation() {
    let mut video = BaseVideo::<Vec<Vec<i32>>>::new(vec![vec![1, -1], vec![1, 1]], 16);
    for (row, col) in [(0, 0), (1, 0), (0, 1)] {
        for event in ["lc", "lr"] { video.step(event, (row * 16 + 1, col * 16 + 1)).unwrap(); }
    }
    assert!(!video.live_pluck_enabled());
    assert_eq!(video.get_live_pluck(), None);
    assert_eq!(video.get_pluck().unwrap(), f64::INFINITY);
}