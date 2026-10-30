use ms_toollib::videos::base_video::NewBaseVideo2;
use ms_toollib::videos::NewSomeVideo2;
use ms_toollib::{
    BaseVideo, Event, EvfVideo, GameBoardState, GameStateEvent, VideoActionStateRecorder,
};

fn board() -> Vec<Vec<i32>> {
    vec![vec![1, 1, 1], vec![1, -1, 1], vec![1, 1, 1]]
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}

fn left(video: &mut BaseVideo<Vec<Vec<i32>>>, pos: (usize, usize)) {
    video.step("lc", pos).unwrap();
    video.step("lr", pos).unwrap();
}

fn right(video: &mut BaseVideo<Vec<Vec<i32>>>, pos: (usize, usize)) {
    video.step("rc", pos).unwrap();
    video.step("rr", pos).unwrap();
}

fn chord(video: &mut BaseVideo<Vec<Vec<i32>>>, pos: (usize, usize)) {
    video.step("cc", pos).unwrap();
    video.step("lr", pos).unwrap();
    video.step("rr", pos).unwrap();
}

fn recorded_video(size: u8, mode: u16) -> BaseVideo<Vec<Vec<i32>>> {
    let mut video = BaseVideo::<Vec<Vec<i32>>>::new(board(), size);
    video.set_mode(mode).unwrap();
    let scale = size as usize / 24;
    assert_eq!(video.get_c_path(), 0.0);
    assert_eq!(video.get_e_path(), 0.0);
    left(&mut video, (2 * scale, 3 * scale));
    // 移动、界外标记、无效左击/双击、错旗都不增加ce。
    video.step("mv", (60 * scale, 60 * scale)).unwrap();
    video.step("mv", (72 * scale, 72 * scale)).unwrap();
    left(&mut video, (20 * scale, 20 * scale));
    chord(&mut video, (10 * scale, 10 * scale));
    right(&mut video, (53 * scale, 51 * scale));
    right(&mut video, (53 * scale, 51 * scale));
    assert_eq!(video.get_c_path(), 0.0);
    assert_eq!(video.get_e_path(), 0.0);

    right(&mut video, (29 * scale, 31 * scale));
    let c_path = video.get_c_path();
    assert_close(c_path, 27.0_f64.hypot(28.0) * 16.0 / 24.0);
    assert_close(video.get_e_path(), 16.0 * 2.0_f64.sqrt());
    // 退旗、换个像素重复标同一雷不增加rce，也不改变距离累计的起点。
    right(&mut video, (47 * scale, 47 * scale));
    right(&mut video, (47 * scale, 47 * scale));
    assert_eq!(video.get_c_path(), c_path);
    assert_close(video.get_e_path(), 16.0 * 2.0_f64.sqrt());
    chord(&mut video, (22 * scale, 21 * scale));
    assert_eq!(video.minesweeper_board.lce, 1);
    assert_eq!(video.minesweeper_board.rce, 1);
    assert_eq!(video.minesweeper_board.dce, 1);
    assert_close(
        video.get_c_path(),
        c_path + 7.0_f64.hypot(10.0) * 16.0 / 24.0,
    );
    assert_close(video.get_e_path(), 32.0 * 2.0_f64.sqrt());
    video
}

#[test]
fn effective_click_paths_use_ce_and_preserve_subcell_coordinates() {
    let mut video = recorded_video(24, 0);
    let expected = (video.get_c_path(), video.get_e_path());
    // 左击踩雷不增加lce；最后一次无效点击不能延长有效点击路径。
    right(&mut video, (29, 31));
    left(&mut video, (29, 31));
    assert_eq!(video.game_board_state, GameBoardState::Loss);
    assert_eq!((video.get_c_path(), video.get_e_path()), expected);
}

#[test]
fn click_paths_are_independent_of_cell_pixel_size() {
    let small = recorded_video(24, 0);
    let large = recorded_video(48, 0);
    assert_close(small.get_c_path(), large.get_c_path());
    assert_close(small.get_e_path(), large.get_e_path());
}

#[test]
fn losing_chord_counts_as_ce_and_same_cell_has_zero_e_path() {
    let mut video = BaseVideo::<Vec<Vec<i32>>>::new(board(), 24);
    left(&mut video, (2, 3));
    right(&mut video, (2, 29)); // 错旗不增加rce，但使双击触雷。
    chord(&mut video, (22, 21));
    assert_eq!(video.game_board_state, GameBoardState::Loss);
    assert_eq!(video.get_ce().unwrap(), 2);
    assert_close(video.get_c_path(), 20.0_f64.hypot(18.0) * 16.0 / 24.0);
    assert_eq!(video.get_e_path(), 0.0);
}

#[test]
fn recursive_chords_only_count_the_manual_effective_click() {
    let video = recorded_video(24, 11);
    assert_eq!(video.game_board_state, GameBoardState::Win);
    assert_eq!(video.get_ce().unwrap(), 3);
    assert_close(video.get_e_path(), 32.0 * 2.0_f64.sqrt());
}

#[test]
fn preflags_count_and_reset_clears_both_distances_and_origin() {
    let mut video = BaseVideo::<Vec<Vec<i32>>>::new(board(), 24);
    right(&mut video, (29, 31));
    left(&mut video, (2, 3));
    assert_close(video.get_c_path(), 27.0_f64.hypot(28.0) * 16.0 / 24.0);
    assert_close(video.get_e_path(), 16.0 * 2.0_f64.sqrt());

    video.reset(3, 3, 24);
    video.set_board(board()).unwrap();
    assert_eq!(video.get_c_path(), 0.0);
    assert_eq!(video.get_e_path(), 0.0);
    left(&mut video, (50, 50));
    assert_eq!(video.get_c_path(), 0.0);
    assert_eq!(video.get_e_path(), 0.0);
}

#[test]
fn cancelling_all_preflags_starts_a_new_click_path() {
    let two_mines = vec![vec![-1, 1, 0], vec![1, 2, 1], vec![0, 1, -1]];
    let mut video = BaseVideo::<Vec<Vec<i32>>>::new(two_mines, 24);
    right(&mut video, (2, 3));
    right(&mut video, (50, 51));
    assert!(video.get_c_path() > 0.0);
    assert_close(video.get_e_path(), 32.0 * 2.0_f64.sqrt());
    right(&mut video, (2, 3));
    right(&mut video, (50, 51));
    assert_eq!(video.game_board_state, GameBoardState::Ready);
    assert_eq!(video.get_c_path(), 0.0);
    assert_eq!(video.get_e_path(), 0.0);
    left(&mut video, (29, 31));
    assert_eq!(video.get_c_path(), 0.0);
    assert_eq!(video.get_e_path(), 0.0);
}

#[test]
fn replay_seek_and_non_mouse_events_preserve_click_paths() {
    let live = recorded_video(24, 0);
    let mut replay = BaseVideo::<Vec<Vec<i32>>>::new(board(), 24);
    replay.game_board_state = GameBoardState::Display;
    replay.can_analyse = true;
    replay.video_action_state_recorder = live.video_action_state_recorder.clone();
    replay
        .video_action_state_recorder
        .push(VideoActionStateRecorder {
            time: 1.0,
            event: Some(Event::GameState(GameStateEvent {
                game_state: "fail".into(),
            })),
            ..VideoActionStateRecorder::default()
        });
    // 重复分析新指标应保持不变。
    for _ in 0..2 {
        replay.analyse();
        for (id, expected) in live.video_action_state_recorder.iter().enumerate().rev() {
            replay.set_current_event_id(id).unwrap();
            assert_close(replay.get_c_path(), expected.c_path);
            assert_close(replay.get_e_path(), expected.e_path);
        }
        replay.set_current_time(f64::INFINITY);
        assert_close(replay.get_c_path(), live.get_c_path());
        assert_close(replay.get_e_path(), live.get_e_path());
    }
}

#[test]
fn evf_roundtrip_recomputes_click_paths() {
    let mut live = recorded_video(24, 11);
    live.set_software("元3.2.1".into()).unwrap();
    live.generate_evf_v4_raw_data();
    let mut parsed = EvfVideo::new(live.raw_data.clone(), "click-paths.evf");
    parsed.parse().unwrap();
    parsed.data.analyse();
    parsed.data.set_current_time(f64::INFINITY);
    assert_close(parsed.data.get_c_path(), live.get_c_path());
    assert_close(parsed.data.get_e_path(), live.get_e_path());
}

#[test]
fn final_board_rescoring_reclassifies_effective_click_paths() {
    for finish in ["state", "loss", "win"] {
        let mut video = BaseVideo::<Vec<Vec<i32>>>::new(board(), 24);
        left(&mut video, (2, 3));
        right(&mut video, (29, 31));
        assert!(video.get_c_path() > 0.0);
        assert_close(video.get_e_path(), 16.0 * 2.0_f64.sqrt());
        // 将雷移到(0,1)，已打开的(0,0)仍为1；原来正确的旗变为错旗。
        video
            .set_board(vec![vec![1, -1, 1], vec![1, 1, 1], vec![0, 0, 0]])
            .unwrap();
        if finish == "win" {
            right(&mut video, (29, 31));
            left(&mut video, (50, 3));
            left(&mut video, (2, 51));
            assert_eq!(video.game_board_state, GameBoardState::Win);
            assert_eq!(video.get_ce().unwrap(), 3);
            assert_close(video.get_c_path(), 32.0 * (1.0 + 2.0_f64.sqrt()));
            assert_close(video.get_e_path(), 32.0 * (1.0 + 2.0_f64.sqrt()));
            let mut replay = video.clone();
            replay.can_analyse = true;
            replay.game_board_state = GameBoardState::Display;
            replay.analyse();
            replay.set_current_time(f64::INFINITY);
            assert_close(replay.get_c_path(), video.get_c_path());
            assert_close(replay.get_e_path(), video.get_e_path());
            continue;
        } else if finish == "loss" {
            right(&mut video, (29, 31));
            left(&mut video, (2, 29));
            assert_eq!(video.get_ce().unwrap(), 1);
        } else {
            video.step_game_state("fail").unwrap();
        }
        assert_eq!(video.game_board_state, GameBoardState::Loss);
        assert_eq!(video.get_c_path(), 0.0);
        assert_eq!(video.get_e_path(), 0.0);
    }
}
