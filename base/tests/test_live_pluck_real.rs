use ms_toollib::videos::{base_video::NewBaseVideo2, NewSomeVideo};
use ms_toollib::{AvfVideo, BaseVideo, Event, GameBoardState};

const RECORDING: &str = "../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf";

fn recording() -> BaseVideo<Vec<Vec<i32>>> {
    let mut video = AvfVideo::new(RECORDING);
    video.parse().unwrap();
    video.data.analyse();
    video.data.analyse_for_features(&vec!["pluck"]);
    video.data
}

fn close(actual: f64, expected: f64, context: &str) {
    assert!((actual - expected).abs() < 1e-10, "{context}: {actual} != {expected}");
}

#[test]
fn real_win_matches_each_single_and_chord_and_midgame_enable() {
    let original = recording();
    let mut live = BaseVideo::<Vec<Vec<i32>>>::new(original.board.clone(), original.cell_pixel_size);
    let mut late = BaseVideo::<Vec<Vec<i32>>>::new(original.board.clone(), original.cell_pixel_size);
    live.set_live_pluck_enabled(true);
    let mut singles = 0;
    let mut chords = 0;
    let mut late_checks = 0;
    let mut enabled = false;
    for (index, record) in original.video_action_state_recorder.iter().enumerate() {
        let Some(Event::Mouse(mouse)) = &record.event else { continue };
        let pos = (mouse.y as usize, mouse.x as usize);
        live.step(&mouse.mouse, pos).unwrap();
        late.step(&mouse.mouse, pos).unwrap();
        if !enabled && singles + chords >= 25 {
            late.set_live_pluck_enabled(true);
            enabled = true;
        }
        if matches!(record.useful_level, 2 | 3) {
            let actual_level = live.video_action_state_recorder.last().unwrap().useful_level;
            assert_eq!(actual_level, record.useful_level, "event {index}");
            if record.useful_level == 2 { singles += 1 } else { chords += 1 }
            let expected = record.key_dynamic_params.pluck;
            close(live.live_pluck_snapshot().wait().unwrap(), expected, &format!("event {index}"));
            if enabled {
                close(late.live_pluck_snapshot().wait().unwrap(), expected, &format!("late event {index}"));
                late_checks += 1;
            }
        }
    }
    assert_eq!(live.game_board_state, GameBoardState::Win);
    assert!(singles > 0 && chords > 0 && late_checks > 0);
    close(live.get_pluck().unwrap(), live.live_pluck_snapshot().wait().unwrap(), "finished win");
    println!("real win: singles={singles} chords={chords} late_checks={late_checks} pluck={}", live.get_pluck().unwrap());
}

#[test]
fn real_board_loss_skips_fatal_click_and_reset_clears_history() {
    let original = recording();
    let mut live = BaseVideo::<Vec<Vec<i32>>>::new(original.board.clone(), original.cell_pixel_size);
    live.set_live_pluck_enabled(true);
    let mut safe = 0;
    for record in &original.video_action_state_recorder {
        let Some(Event::Mouse(mouse)) = &record.event else { continue };
        live.step(&mouse.mouse, (mouse.y as usize, mouse.x as usize)).unwrap();
        if record.useful_level == 2 { safe += 1; }
        if safe == 10 { break; }
    }
    let before = live.live_pluck_snapshot();
    let expected = before.wait().unwrap();
    let (row, col) = original.board.iter().enumerate().find_map(|(r, line)| {
        line.iter().enumerate().find_map(|(c, &cell)| {
            (cell == -1 && live.minesweeper_board.game_board[r][c] == 10).then_some((r, c))
        })
    }).unwrap();
    let pixel = original.cell_pixel_size as usize;
    let pos = (row * pixel + 1, col * pixel + 1);
    live.step("lc", pos).unwrap();
    live.step("lr", pos).unwrap();
    assert_eq!(live.game_board_state, GameBoardState::Loss);
    assert_eq!(live.video_action_state_recorder.last().unwrap().useful_level, 4);
    close(live.live_pluck_snapshot().wait().unwrap(), expected, "fatal click skipped");
    assert_eq!(live.get_pluck().unwrap(), f64::INFINITY);
    live.reset(original.height, original.width, original.cell_pixel_size);
    assert_eq!(live.live_pluck_snapshot().wait().unwrap(), 0.0);
    close(before.wait().unwrap(), expected, "old snapshot after reset");
    live.set_board(original.board.clone()).unwrap();
    assert_eq!(live.live_pluck_snapshot().wait().unwrap(), 0.0);
    println!("real board loss: safe_singles={safe} pre_fatal_pluck={expected} legacy=inf live={expected}");
}

#[test]
fn midgame_board_replacement_preserves_history_and_new_actions_match() {
    let original = recording();
    let mut live = BaseVideo::<Vec<Vec<i32>>>::new(original.board.clone(), original.cell_pixel_size);
    live.set_live_pluck_enabled(true);
    for record in &original.video_action_state_recorder {
        let Some(Event::Mouse(mouse)) = &record.event else { continue };
        live.step(&mouse.mouse, (mouse.y as usize, mouse.x as usize)).unwrap();
        if record.useful_level == 2 { break; }
    }
    let before = live.live_pluck_snapshot();
    let visible = &live.minesweeper_board.game_board;
    let candidates: Vec<_> = (0..original.height).flat_map(|r| (0..original.width).map(move |c| (r, c)))
        .filter(|&(r, c)| visible[r][c] == 10 &&
            (r.saturating_sub(1)..=(r + 1).min(original.height - 1)).all(|rr|
                (c.saturating_sub(1)..=(c + 1).min(original.width - 1)).all(|cc| visible[rr][cc] >= 10)))
        .collect();
    let mine = *candidates.iter().find(|&&(r, c)| original.board[r][c] == -1).unwrap();
    let safe = *candidates.iter().find(|&&(r, c)| original.board[r][c] != -1).unwrap();
    let mut replacement: Vec<Vec<i32>> = original.board.iter().map(|row| row.iter().map(|&v| if v == -1 { -1 } else { 0 }).collect()).collect();
    replacement[mine.0][mine.1] = 0;
    replacement[safe.0][safe.1] = -1;
    let mines = replacement.clone();
    for (r, row) in replacement.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            if *cell != -1 {
                *cell = (r.saturating_sub(1)..=(r + 1).min(original.height - 1))
                    .flat_map(|rr| (c.saturating_sub(1)..=(c + 1).min(original.width - 1)).map(move |cc| (rr, cc)))
                    .filter(|&(rr, cc)| mines[rr][cc] == -1).count() as i32;
            }
        }
    }
    live.set_board(replacement).unwrap();
    close(live.live_pluck_snapshot().wait().unwrap(), before.wait().unwrap(), "replacement preserves history");
    let pixel = original.cell_pixel_size as usize;
    let pos = (mine.0 * pixel + 1, mine.1 * pixel + 1);
    live.step("lc", pos).unwrap();
    live.step("lr", pos).unwrap();
    assert_eq!(live.video_action_state_recorder.last().unwrap().useful_level, 2);
    let actual = live.live_pluck_snapshot().wait().unwrap();
    // Exercise the synchronous end-of-game analyser over the exact same recorded actions.
    live.game_board_state = GameBoardState::Win;
    close(actual, live.get_pluck().unwrap(), "after replacement");
    println!("midgame replacement: moved mine {mine:?} -> {safe:?}, post_swap_pluck={actual}");
}