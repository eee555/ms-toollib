use ms_toollib::videos::base_video::NewBaseVideo2;
use ms_toollib::videos::NewSomeVideo;
use ms_toollib::{
    BaseVideo, Event, GameBoard, GameBoardState, GameStateEvent, KeyDynamicParams, MouseEvent,
    VideoActionStateRecorder,
};
use std::cell::RefCell;
use std::rc::Rc;

type Video = BaseVideo<Vec<Vec<i32>>>;

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

fn board(data: Vec<Vec<i32>>) -> Rc<RefCell<GameBoard>> {
    let mut board = GameBoard::new(1);
    board.set_game_board(&data);
    Rc::new(RefCell::new(board))
}

fn mouse(
    time: f64,
    action: &str,
    cell: (u16, u16),
    useful: u8,
    solved: usize,
    prior: Rc<RefCell<GameBoard>>,
) -> VideoActionStateRecorder {
    VideoActionStateRecorder {
        time,
        event: Some(Event::Mouse(MouseEvent {
            mouse: action.into(),
            x: cell.1 * 16 + 1,
            y: cell.0 * 16 + 1,
        })),
        useful_level: useful,
        prior_game_board: Some(prior),
        key_dynamic_params: KeyDynamicParams {
            bbbv_solved: solved,
            ..KeyDynamicParams::default()
        },
        ..VideoActionStateRecorder::default()
    }
}

fn replay() -> Video {
    let mut video = Video::default();
    video.width = 2;
    video.height = 2;
    video.mine_num = 1;
    video.delta_time = 5.0;
    video.game_dynamic_params.rtime = 20.0;
    let initial = board(vec![vec![10, 10], vec![10, 10]]);
    let first = board(vec![vec![1, 10], vec![10, 10]]);
    let second = board(vec![vec![1, 11], vec![1, 10]]);
    video.video_action_state_recorder = vec![
        mouse(1.0, "mv", (0, 0), 0, 0, initial.clone()),
        mouse(5.0, "lr", (0, 0), 2, 1, initial),
        mouse(7.0, "mv", (0, 0), 0, 1, first.clone()),
        mouse(15.0, "lr", (1, 0), 2, 2, first),
        mouse(18.0, "rc", (0, 1), 1, 2, second.clone()),
        mouse(25.0, "lr", (1, 1), 2, 3, second),
    ];
    video
}

fn result(video: &mut Video) -> [f64; 6] {
    [
        video.get_pluck().unwrap(),
        video.get_luck().unwrap(),
        video.get_expected_bbbv_solved_per_attempt().unwrap(),
        video.get_expected_bbbv_solved_per_win().unwrap(),
        video.get_expected_time_per_attempt().unwrap(),
        video.get_expected_time_per_win().unwrap(),
    ]
}

#[test]
fn replay_uses_post_action_luck_for_work_and_pre_action_luck_for_time() {
    let mut video = replay();
    assert!(video.get_luck().unwrap().is_nan());
    video.analyse_for_features(&vec!["pluck"]);
    video.set_current_time(f64::INFINITY);
    let values = result(&mut video);
    close(values[0], -(2.0_f64 / 3.0).log10() - 0.5_f64.log10());
    close(values[1], 1.0 / 3.0);
    close(values[2], 2.0); // 1 + 2/3 + 1/3。
    close(values[3], 6.0);
    close(values[4], 10.0 + (2.0 / 3.0) * 10.0);
    close(values[5], 50.0);
    // 同一轮feature分析重复执行不能重复累计。
    video.analyse_for_features(&vec!["pluck"]);
    let repeated = result(&mut video);
    for (actual, expected) in repeated.into_iter().zip(values) {
        close(actual, expected);
    }
}

#[test]
fn risky_action_without_bv_progress_still_changes_luck_and_time_weights() {
    let mut video = replay();
    // 控制解决量快照，验证概率计算不依赖本次是否推进BV。
    video.video_action_state_recorder[3]
        .key_dynamic_params
        .bbbv_solved = 1;
    video.video_action_state_recorder[4]
        .key_dynamic_params
        .bbbv_solved = 1;
    video.analyse_for_features(&vec!["pluck"]);
    video.set_current_time(f64::INFINITY);
    close(video.get_luck().unwrap(), 1.0 / 3.0);
    close(
        video.get_expected_bbbv_solved_per_attempt().unwrap(),
        1.0 + 2.0 / 3.0,
    );
    close(video.get_expected_bbbv_solved_per_win().unwrap(), 5.0);
    close(video.get_expected_time_per_win().unwrap(), 50.0);
}

#[test]
fn seek_continues_time_between_events_without_changing_work() {
    let mut video = replay();
    video.analyse_for_features(&vec!["pluck"]);
    video.set_current_time(15.0); // 绝对时间20，位于18秒的标旗和25秒的猜雷之间。
    close(video.get_luck().unwrap(), 2.0 / 3.0);
    close(
        video.get_expected_bbbv_solved_per_attempt().unwrap(),
        1.0 + 2.0 / 3.0,
    );
    close(
        video.get_expected_time_per_attempt().unwrap(),
        10.0 + 5.0 * 2.0 / 3.0,
    );
    close(video.get_expected_time_per_win().unwrap(), 20.0);
    video.set_current_time(0.0);
    close(video.get_expected_bbbv_solved_per_attempt().unwrap(), 1.0);
    close(video.get_expected_time_per_attempt().unwrap(), 0.0);
    video.set_current_time(-1.0);
    close(video.get_expected_bbbv_solved_per_attempt().unwrap(), 0.0);
    close(video.get_luck().unwrap(), 1.0);
}

#[test]
fn lazy_finished_game_matches_replay_and_caches_the_whole_result() {
    let mut expected = replay();
    expected.analyse_for_features(&vec!["pluck"]);
    expected.set_current_time(f64::INFINITY);
    let expected = result(&mut expected);
    let mut finished = replay();
    finished.game_board_state = GameBoardState::Win;
    let actual = result(&mut finished);
    for (actual, expected) in actual.into_iter().zip(expected) {
        close(actual, expected);
    }
    // 持有概率局面的可变借用：重复读取若再次计算概率，会触发RefCell借用冲突。
    let prior = finished.video_action_state_recorder[3]
        .prior_game_board
        .clone()
        .unwrap();
    let _borrow = prior.borrow_mut();
    close(finished.get_expected_time_per_win().unwrap(), 50.0);
    close(finished.get_luck().unwrap(), 1.0 / 3.0);
}

#[test]
fn chord_uses_joint_safety_and_counts_all_bv_solved_at_once() {
    let mut video = replay();
    let prior = board(vec![vec![1, 11], vec![10, 10]]);
    video.video_action_state_recorder.truncate(2);
    // 旗不被概率引擎当作已知雷。两个待开格同时安全的概率为1/3，非(2/3)^2。
    video
        .video_action_state_recorder
        .push(mouse(15.0, "lr", (0, 0), 3, 3, prior));
    video.game_dynamic_params.rtime = 10.0;
    video.analyse_for_features(&vec!["pluck"]);
    video.set_current_time(f64::INFINITY);
    close(video.get_luck().unwrap(), 1.0 / 3.0);
    close(
        video.get_expected_bbbv_solved_per_attempt().unwrap(),
        1.0 + 2.0 / 3.0,
    );
    close(video.get_expected_bbbv_solved_per_win().unwrap(), 5.0);
    close(video.get_expected_time_per_attempt().unwrap(), 10.0);
    close(video.get_expected_time_per_win().unwrap(), 30.0);
}

#[test]
fn fatal_action_keeps_spent_time_and_existing_pluck_failure_markers() {
    for display in [false, true] {
        let mut video = replay();
        video.video_action_state_recorder[5].useful_level = 4;
        video.game_board_state = if display {
            GameBoardState::Display
        } else {
            GameBoardState::Loss
        };
        if display {
            video.analyse_for_features(&vec!["pluck"]);
            video.set_current_time(f64::INFINITY);
            assert_eq!(video.get_pluck().unwrap(), f64::MAX);
        } else {
            assert_eq!(video.get_pluck().unwrap(), f64::INFINITY);
        }
        assert_eq!(video.get_luck().unwrap(), 0.0);
        close(
            video.get_expected_bbbv_solved_per_attempt().unwrap(),
            1.0 + 2.0 / 3.0,
        );
        close(
            video.get_expected_time_per_attempt().unwrap(),
            10.0 + 10.0 * 2.0 / 3.0,
        );
        assert_eq!(
            video.get_expected_bbbv_solved_per_win().unwrap(),
            f64::INFINITY
        );
        assert_eq!(video.get_expected_time_per_win().unwrap(), f64::INFINITY);
    }
}

#[test]
fn non_mouse_event_inherits_luck_and_final_idle_time_is_included() {
    let mut video = replay();
    video
        .video_action_state_recorder
        .push(VideoActionStateRecorder {
            time: 27.0,
            event: Some(Event::GameState(GameStateEvent {
                game_state: "win".into(),
            })),
            ..VideoActionStateRecorder::default()
        });
    video.game_dynamic_params.rtime = 25.0;
    video.analyse_for_features(&vec!["pluck"]);
    video.set_current_time(f64::INFINITY);
    close(video.get_luck().unwrap(), 1.0 / 3.0);
    close(video.get_expected_bbbv_solved_per_attempt().unwrap(), 2.0);
    close(
        video.get_expected_time_per_attempt().unwrap(),
        10.0 + 25.0 / 3.0,
    );
    close(video.get_expected_time_per_win().unwrap(), 55.0);
    close(
        video.video_analyse_params.expected_time_per_attempt,
        video.get_expected_time_per_attempt().unwrap(),
    );
}

#[test]
fn reanalysis_and_reset_invalidate_all_cached_metrics() {
    let mut video = Video::new(vec![vec![1, -1], vec![1, 1]], 16);
    for cell in [(0, 0), (1, 0), (1, 1)] {
        for event in ["lc", "lr"] {
            video
                .step(event, (cell.0 * 16 + 1, cell.1 * 16 + 1))
                .unwrap();
        }
    }
    close(video.get_luck().unwrap(), 1.0 / 3.0);
    video.can_analyse = true;
    video.game_board_state = GameBoardState::Display;
    video.analyse();
    assert!(video.video_analyse_params.pluck.is_nan());
    assert!(video.get_expected_time_per_attempt().unwrap().is_nan());
    video.analyse_for_features(&vec!["pluck"]);
    video.set_current_time(f64::INFINITY);
    close(video.get_luck().unwrap(), 1.0 / 3.0);
    video.reset(2, 2, 16);
    assert!(video.video_analyse_params.pluck.is_nan());
    assert!(video
        .video_analyse_params
        .expected_bbbv_solved_per_attempt
        .is_nan());
    assert!(video.video_analyse_params.expected_time_per_win.is_nan());
    assert!(video.get_luck().unwrap().is_nan());
}

#[test]
fn board_change_updates_bv_snapshots_before_lazy_luck_analysis() {
    let mut video = Video::new(vec![vec![1, 1, 1], vec![1, -1, 1], vec![1, 1, 1]], 16);
    for event in ["lc", "lr"] {
        video.step(event, (1, 17)).unwrap();
    }
    assert_eq!(
        video.video_action_state_recorder[1]
            .key_dynamic_params
            .bbbv_solved,
        1
    );
    // (0,1)仍显示1，但新雷盘中它属于空区边缘，不再单独计BV。
    video
        .set_board(vec![vec![-1, 1, 0], vec![1, 1, 0], vec![0, 0, 0]])
        .unwrap();
    for event in ["lc", "lr"] {
        video.step(event, (33, 33)).unwrap();
    }
    assert_eq!(video.game_board_state, GameBoardState::Win);
    assert_eq!(
        video.video_action_state_recorder[1]
            .key_dynamic_params
            .bbbv_solved,
        0
    );
    close(video.get_luck().unwrap(), 1.0);
    close(video.get_expected_bbbv_solved_per_attempt().unwrap(), 1.0);
    close(
        video.video_action_state_recorder[1]
            .key_dynamic_params
            .expected_bbbv_solved_per_attempt,
        0.0,
    );
}

#[test]
fn mvf_fixture_preserves_pluck_and_exposes_weighted_metrics() {
    let mut video = ms_toollib::MvfVideo::new("../test_files/Zhang Shen Jia_Exp_38.82(3bv122).mvf");
    video.parse().unwrap();
    video.data.analyse();
    video.data.set_current_time(12.0);
    video.data.analyse_for_features(&vec!["pluck"]);
    close(video.data.get_pluck().unwrap(), 0.4612441009087633);
    let luck = video.data.get_luck().unwrap();
    let time = video.data.get_expected_time_per_attempt().unwrap();
    let solved = video.data.get_expected_bbbv_solved_per_attempt().unwrap();
    assert!(time > 0.0 && time <= 12.0);
    assert!(solved > 0.0);
    close(video.data.get_expected_time_per_win().unwrap(), time / luck);
    close(
        video.data.get_expected_bbbv_solved_per_win().unwrap(),
        solved / luck,
    );
}
