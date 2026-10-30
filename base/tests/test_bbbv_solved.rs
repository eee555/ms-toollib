use ms_toollib::videos::base_video::NewBaseVideo2;
use ms_toollib::videos::NewSomeVideo2;
use ms_toollib::{
    BaseVideo, Event, EvfVideo, GameBoardState, GameStateEvent, VideoActionStateRecorder,
};

type Video = BaseVideo<Vec<Vec<i32>>>;

fn numbered_board() -> Vec<Vec<i32>> {
    vec![vec![1, 1, 1], vec![1, -1, 1], vec![1, 1, 1]]
}

fn opening_board() -> Vec<Vec<i32>> {
    vec![vec![-1, 1, 0], vec![1, 1, 0], vec![0, 0, 0]]
}

fn action(video: &mut Video, event: &str, cell: (usize, usize)) {
    video
        .step(event, (cell.0 * 24 + 2, cell.1 * 24 + 3))
        .unwrap();
}

fn left(video: &mut Video, cell: (usize, usize)) {
    action(video, "lc", cell);
    action(video, "lr", cell);
}

fn right(video: &mut Video, cell: (usize, usize)) {
    action(video, "rc", cell);
    action(video, "rr", cell);
}

fn chord(video: &mut Video, cell: (usize, usize)) {
    action(video, "cc", cell);
    action(video, "lr", cell);
    action(video, "rr", cell);
}

fn assert_solved(video: &Video, nf: usize, fl: usize) {
    let board = &video.minesweeper_board;
    assert_eq!((board.bbbv_solved_nf, board.bbbv_solved_fl), (nf, fl));
    assert_eq!(board.get_bbbv_solved(), nf + fl);
    if let Some(event) = video.video_action_state_recorder.last() {
        let params = &event.key_dynamic_params;
        assert_eq!((params.bbbv_solved_nf, params.bbbv_solved_fl), (nf, fl));
        assert_eq!(params.get_bbbv_solved(), nf + fl);
    }
}

fn mixed_video() -> Video {
    let mut video = Video::new(numbered_board(), 24);
    left(&mut video, (0, 0));
    assert_solved(&video, 1, 0);
    // 重复点已打开的格子、标雷和退旗均不解决3BV。
    left(&mut video, (0, 0));
    right(&mut video, (1, 1));
    right(&mut video, (1, 1));
    right(&mut video, (1, 1));
    assert_solved(&video, 1, 0);
    chord(&mut video, (0, 0));
    assert_solved(&video, 1, 2);
    left(&mut video, (0, 2));
    assert_solved(&video, 2, 2);
    chord(&mut video, (1, 0));
    assert_solved(&video, 2, 4);
    left(&mut video, (2, 2));
    left(&mut video, (1, 2));
    assert_eq!(video.game_board_state, GameBoardState::Win);
    assert_solved(&video, 4, 4);
    video
}

#[test]
fn left_and_chord_solutions_are_counted_separately() {
    let video = mixed_video();
    assert_eq!(video.get_bbbv_solved_nf().unwrap(), 4);
    assert_eq!(video.get_bbbv_solved_fl().unwrap(), 4);
    assert_eq!(video.get_bbbv_solved().unwrap(), video.static_params.bbbv);
    assert_eq!(video.video_dynamic_params.bbbv_solved_nf, 4);
    assert_eq!(video.video_dynamic_params.bbbv_solved_fl, 4);
    assert_eq!(video.video_dynamic_params.get_bbbv_solved(), 8);
}

#[test]
fn opening_credit_belongs_to_the_action_that_opens_it() {
    let mut left_video = Video::new(opening_board(), 24);
    left(&mut left_video, (2, 2));
    assert_solved(&left_video, 1, 0);

    let mut chord_video = Video::new(opening_board(), 24);
    left(&mut chord_video, (1, 1)); // 空边数字是有效左击，但不增加3BV。
    assert_eq!(chord_video.minesweeper_board.lce, 1);
    assert_solved(&chord_video, 0, 0);
    right(&mut chord_video, (0, 0));
    chord(&mut chord_video, (1, 1));
    assert_eq!(chord_video.game_board_state, GameBoardState::Win);
    assert_solved(&chord_video, 0, 1);
}

#[test]
fn partially_blocked_opening_is_credited_when_completed() {
    for finish_with_chord in [false, true] {
        let mut video = Video::new(opening_board(), 24);
        right(&mut video, (2, 2)); // 在零格上错标旗，阻止空区完整展开。
        left(&mut video, (0, 2));
        assert_solved(&video, 0, 0);
        right(&mut video, (2, 2));
        if finish_with_chord {
            right(&mut video, (0, 0));
            chord(&mut video, (1, 1));
            assert_solved(&video, 0, 1);
        } else {
            left(&mut video, (2, 2));
            assert_solved(&video, 1, 0);
        }
        assert_eq!(video.game_board_state, GameBoardState::Win);
    }
}

#[test]
fn losing_chord_still_credits_safe_cells_to_fl() {
    let mut video = Video::new(numbered_board(), 24);
    left(&mut video, (0, 0));
    right(&mut video, (0, 1)); // 错旗使双击同时打开安全格(1,0)并触雷。
    chord(&mut video, (0, 0));
    assert_eq!(video.game_board_state, GameBoardState::Loss);
    assert_solved(&video, 1, 1);
    assert_eq!(video.get_bbbv_solved().unwrap(), 2);
}

#[test]
fn recursive_chords_credit_fl_even_without_manual_dce() {
    for mode in [11, 13] {
        let mut video = Video::new(numbered_board(), 24);
        video.set_mode(mode).unwrap();
        left(&mut video, (0, 0));
        right(&mut video, (1, 1));
        if mode == 11 {
            chord(&mut video, (0, 0));
        }
        assert_eq!(video.game_board_state, GameBoardState::Win);
        assert_solved(&video, 1, 7);
        assert_eq!(video.minesweeper_board.dce, if mode == 11 { 1 } else { 0 });
    }
}

#[test]
fn reset_clears_both_counters() {
    let mut video = mixed_video();
    video.minesweeper_board.reset();
    assert_eq!(video.minesweeper_board.bbbv_solved_nf, 0);
    assert_eq!(video.minesweeper_board.bbbv_solved_fl, 0);
    video.reset(3, 3, 24);
    video.set_board(numbered_board()).unwrap();
    assert_eq!(video.video_dynamic_params.get_bbbv_solved(), 0);
    assert_solved(&video, 0, 0);
    left(&mut video, (0, 0));
    assert_solved(&video, 1, 0);
}

#[test]
fn evf_roundtrip_and_seek_preserve_both_counters_and_total_metrics() {
    let mut live = mixed_video();
    live.set_software("元3.2.1".into()).unwrap();
    live.generate_evf_v4_raw_data();
    let mut parsed = EvfVideo::new(live.raw_data.clone(), "split-bbbv.evf");
    parsed.parse().unwrap();
    parsed.data.analyse();
    for (id, expected) in live.video_action_state_recorder.iter().enumerate().rev() {
        parsed.data.set_current_event_id(id).unwrap();
        assert_eq!(
            parsed.data.get_bbbv_solved_nf().unwrap(),
            expected.key_dynamic_params.bbbv_solved_nf
        );
        assert_eq!(
            parsed.data.get_bbbv_solved_fl().unwrap(),
            expected.key_dynamic_params.bbbv_solved_fl
        );
        assert_eq!(
            parsed.data.get_bbbv_solved().unwrap(),
            expected.key_dynamic_params.get_bbbv_solved()
        );
    }
    parsed.data.game_dynamic_params.rtime = 2.0;
    parsed.data.set_current_time(f64::INFINITY);
    assert_eq!(parsed.data.get_bbbv_s().unwrap(), 4.0);
    assert_eq!(parsed.data.get_ioe().unwrap(), 8.0 / live.get_cl() as f64);
    assert_eq!(
        parsed.data.get_thrp().unwrap(),
        8.0 / live.get_ce().unwrap() as f64
    );
    // 非鼠标事件仍应继承累计的两项解决量。
    parsed
        .data
        .video_action_state_recorder
        .push(VideoActionStateRecorder {
            time: 2.0,
            event: Some(Event::GameState(GameStateEvent {
                game_state: "win".into(),
            })),
            ..VideoActionStateRecorder::default()
        });
    parsed.data.analyse();
    parsed.data.set_current_time(f64::INFINITY);
    assert_eq!(parsed.data.get_bbbv_solved_nf().unwrap(), 4);
    assert_eq!(parsed.data.get_bbbv_solved_fl().unwrap(), 4);
    assert_eq!(parsed.data.get_bbbv_solved().unwrap(), 8);
}

#[test]
fn board_change_rescores_both_counters_and_prior_snapshots() {
    let mut video = Video::new(numbered_board(), 24);
    left(&mut video, (0, 1));
    assert_solved(&video, 1, 0);
    // 移动地雷后，已打开的(0,1)仍是1，却成为空边数字，不再单独计3BV。
    video.set_board(opening_board()).unwrap();
    left(&mut video, (2, 2));
    assert_eq!(video.game_board_state, GameBoardState::Win);
    assert_solved(&video, 1, 0);
    assert_eq!(
        video.video_action_state_recorder[1]
            .key_dynamic_params
            .get_bbbv_solved(),
        0
    );
    assert_eq!(video.get_bbbv_solved().unwrap(), video.static_params.bbbv);
}
