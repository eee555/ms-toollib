use crate::algorithms::{cal_probability_cells_not_mine, mark_board};
use crate::utils::is_good_chording;
use crate::videos::base_video::BaseVideo;
use crate::videos::types::Event;
use crate::MouseState;
use std::cmp::{max, min};

// 录像的事件分析。参与分析的录像必须已经计算出对应的数据。
// error: 高风险的猜雷（猜对概率0.05）√
// feature: 高难度的判雷√
// warning: 可以判雷时视野的转移√
// feature: 双线操作
// feature: 破空（成功率0.98）
// feature: 教科书式的FL局部(步数4)√
// error: 过于弯曲的鼠标轨迹(500%)√
// warning：弯曲的鼠标轨迹(200%)√
// warning: 可以判雷时选择猜雷√
// warning: 没有作用的操作
// suspect: 点击速度过快(0.01)
// suspect: 鼠标移动过快(2)
// suspect: 笔直的鼠标轨迹(101%)√
pub fn analyse_high_risk_guess(video: &mut BaseVideo<Vec<Vec<i32>>>) {
    let mut r;
    let mut c;
    for ide in 2..video.video_action_state_recorder.len() {
        let vas = &mut video.video_action_state_recorder[ide];
        if let Some(Event::Mouse(mouse_event)) = &vas.event {
            r = (mouse_event.y / video.cell_pixel_size as u16) as usize;
            c = (mouse_event.x / video.cell_pixel_size as u16) as usize;
            if vas.useful_level >= 2 {
                let p = vas
                    .prior_game_board
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .get_poss()[r][c];
                if p >= 0.51 {
                    vas.comments = format!(
                        "{}{}",
                        vas.comments,
                        format!("error: 危险的猜雷(猜对概率{:.3});", 1.0 - p)
                    );
                }
            }
        }
    }
}

pub fn analyse_jump_judge(video: &mut BaseVideo<Vec<Vec<i32>>>) {
    // 功能：检测左键或右键的跳判
    let mut r;
    let mut c;
    for ide in 2..video.video_action_state_recorder.len() {
        let vas = &mut video.video_action_state_recorder[ide];
        if let Some(Event::Mouse(mouse_event)) = &vas.event {
            r = (mouse_event.y / video.cell_pixel_size as u16) as usize;
            c = (mouse_event.x / video.cell_pixel_size as u16) as usize;
            if vas.useful_level >= 2 && mouse_event.mouse == "lr" {
                if !vas
                    .prior_game_board
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .get_basic_not_mine()
                    .contains(&(r, c))
                    && vas
                        .prior_game_board
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .get_enum_not_mine()
                        .contains(&(r, c))
                {
                    vas.comments = format!(
                        "{}{}",
                        vas.comments,
                        format!("feature: 高难度的判雷(左键);")
                    );
                }
            } else if vas.useful_level == 1 && mouse_event.mouse == "rc" {
                if !vas
                    .prior_game_board
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .get_basic_is_mine()
                    .contains(&(r, c))
                    && vas
                        .prior_game_board
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .get_enum_is_mine()
                        .contains(&(r, c))
                {
                    vas.comments = format!(
                        "{}{}",
                        vas.comments,
                        format!("feature: 高难度的判雷(标雷);")
                    );
                }
            }
        }
    }
}

// 猜雷时，假如周围5*5范围内有可判的，引发可以判雷时选择猜雷
pub fn analyse_needless_guess(video: &mut BaseVideo<Vec<Vec<i32>>>) {
    let mut r;
    let mut c;
    'outer: for ide in 2..video.video_action_state_recorder.len() {
        let vas = &mut video.video_action_state_recorder[ide];
        if let Some(Event::Mouse(mouse_event)) = &vas.event {
            if vas.useful_level >= 2 && mouse_event.mouse == "lr" {
                r = (mouse_event.y / video.cell_pixel_size as u16) as usize;
                c = (mouse_event.x / video.cell_pixel_size as u16) as usize;

                if vas
                    .prior_game_board
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .get_poss()[r][c]
                    > 0.0
                {
                    for m in max(2, r) - 2..min(video.height, r + 3) {
                        for n in max(2, c) - 2..min(video.width, c + 3) {
                            if vas
                                .prior_game_board
                                .as_ref()
                                .unwrap()
                                .borrow_mut()
                                .get_basic_not_mine()
                                .contains(&(m, n))
                                || vas
                                    .prior_game_board
                                    .as_ref()
                                    .unwrap()
                                    .borrow_mut()
                                    .get_enum_not_mine()
                                    .contains(&(m, n))
                            {
                                vas.comments = format!(
                                    "{}{}",
                                    vas.comments,
                                    format!("warning: 可以判雷时选择猜雷;")
                                );
                                continue 'outer;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 检查鼠标轨迹是否弯曲
pub fn analyse_mouse_trace(video: &mut BaseVideo<Vec<Vec<i32>>>) {
    let Some(Event::Mouse(mut last_click_event)) =
        video.video_action_state_recorder[0].event.clone()
    else {
        panic!("expected mouse event");
    };
    let Some(Event::Mouse(mut last_move_event)) =
        video.video_action_state_recorder[0].event.clone()
    else {
        panic!("expected mouse event");
    };
    let mut comments = vec![];
    let mut click_last_id = 0;
    let mut path = 0.0;
    for ide in 0..video.video_action_state_recorder.len() {
        let vas = &mut video.video_action_state_recorder[ide];
        if let Some(Event::Mouse(mouse_event)) = &vas.event {
            let current_x = mouse_event.x as f64;
            let current_y = mouse_event.y as f64;
            path += ((last_move_event.x as f64 - current_x).powf(2.0)
                + (last_move_event.y as f64 - current_y).powf(2.0))
            .sqrt();
            last_move_event = mouse_event.clone();
            if mouse_event.mouse == "lr" || mouse_event.mouse == "rc" || mouse_event.mouse == "rr" {
                let path_straight = ((last_click_event.x as f64 - current_x).powf(2.0)
                    + (last_click_event.y as f64 - current_y).powf(2.0))
                .sqrt();
                let k = path / path_straight;
                if k > 7.0 {
                    comments.push((
                        click_last_id,
                        format!("error: 过于弯曲的鼠标轨迹({:.0}%);", k * 100.0),
                    ));
                } else if k > 3.5 {
                    comments.push((
                        click_last_id,
                        format!("warning: 弯曲的鼠标轨迹({:.0}%);", k * 100.0),
                    ));
                } else if k < 1.01 {
                    if k > 5.0 {
                        comments.push((click_last_id, "suspect: 笔直的鼠标轨迹;".to_owned()));
                    }
                }
                last_click_event = mouse_event.clone();
                click_last_id = ide;
                path = 0.0;
            }
        }
    }
    for comment in comments {
        video.video_action_state_recorder[comment.0].comments = video.video_action_state_recorder
            [comment.0]
            .comments
            .clone()
            + &comment.1;
    }
}

// bug
pub fn analyse_vision_transfer(video: &mut BaseVideo<Vec<Vec<i32>>>) {
    let Some(Event::Mouse(mut last_click_event)) =
        video.video_action_state_recorder[0].event.clone()
    else {
        panic!("expected mouse event");
    };
    let mut comments = vec![];
    let mut last_c = (last_click_event.y / video.cell_pixel_size as u16) as usize;
    let mut last_r = (last_click_event.x / video.cell_pixel_size as u16) as usize;

    let mut click_last_id = 0;
    for ide in 0..video.video_action_state_recorder.len() {
        let vas = &mut video.video_action_state_recorder[ide];
        if let Some(Event::Mouse(mouse_event)) = &vas.event {
            if vas.useful_level >= 2 {
                if ((last_click_event.x as f64 - mouse_event.x as f64).powf(2.0)
                    + (last_click_event.y as f64 - mouse_event.y as f64).powf(2.0))
                .sqrt()
                    / video.cell_pixel_size as f64
                    >= 6.0
                {
                    let mut flag = false;
                    for &(xxx, yyy) in vas
                        .prior_game_board
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .get_basic_not_mine()
                    {
                        if xxx <= last_c + 3
                            && xxx + 3 >= last_c
                            && yyy <= last_r + 3
                            && yyy + 3 >= last_r
                        {
                            flag = true;
                        }
                    }
                    for &(xxx, yyy) in vas
                        .prior_game_board
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .get_enum_not_mine()
                    {
                        if xxx <= last_c + 3
                            && xxx + 3 >= last_c
                            && yyy <= last_r + 3
                            && yyy + 3 >= last_r
                        {
                            flag = true;
                        }
                    }
                    if flag {
                        comments.push((click_last_id, "warning: 可以判雷时视野的转移;"));
                    }
                }
                last_click_event = mouse_event.clone();
                last_c = (last_click_event.y / video.cell_pixel_size as u16) as usize;
                last_r = (last_click_event.x / video.cell_pixel_size as u16) as usize;
                click_last_id = ide;
            }
        }
    }
    for comment in comments {
        video.video_action_state_recorder[comment.0].comments = video.video_action_state_recorder
            [comment.0]
            .comments
            .clone()
            + &comment.1;
    }
}

/// 计算回放的录像的各个时刻的pluck参数
pub fn analyse_pluck(video: &mut BaseVideo<Vec<Vec<i32>>>) {
    let mut pluck = 0.0;
    let mut has_begin = false;
    for vas in video.video_action_state_recorder.iter_mut() {
        if let Some(Event::Mouse(mouse_event)) = &vas.event {
            if vas.useful_level == 2 {
                // 有效的左键
                if !has_begin {
                    has_begin = true;
                    vas.key_dynamic_params.pluck = 0.0;
                    continue;
                }
                let r = (mouse_event.y / video.cell_pixel_size as u16) as usize;
                let c = (mouse_event.x / video.cell_pixel_size as u16) as usize;
                // 安全的概率
                let p = 1.0
                    - vas
                        .prior_game_board
                        .as_ref()
                        .unwrap()
                        .borrow_mut()
                        .get_poss()[r][c];
                if p <= 0.0 || pluck == f64::MAX {
                    pluck = f64::MAX;
                } else if p < 1.0 {
                    pluck -= p.log10();
                }
            } else if vas.useful_level == 3 {
                // 有效的双键
                let r = (mouse_event.y / video.cell_pixel_size as u16) as usize;
                let c = (mouse_event.x / video.cell_pixel_size as u16) as usize;
                let mut game_board_clone = vas
                    .prior_game_board
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .game_board
                    .clone();
                let mut chording_cells = vec![];
                for m in max(1, r) - 1..min(video.height, r + 2) {
                    for n in max(1, c) - 1..min(video.width, c + 2) {
                        if game_board_clone[m][n] == 10 {
                            chording_cells.push((m, n));
                        }
                    }
                }
                let _ = mark_board(&mut game_board_clone, true).unwrap();
                // 安全的概率
                let p = cal_probability_cells_not_mine(
                    &game_board_clone,
                    video.mine_num as f64,
                    &chording_cells,
                );
                if p <= 0.0 || pluck == f64::MAX {
                    pluck = f64::MAX;
                } else if p > 0.0 {
                    pluck -= p.log10();
                }
            } else if vas.useful_level == 4 {
                pluck = f64::MAX;
            }

            if has_begin {
                vas.key_dynamic_params.pluck = pluck;
            } else {
                vas.key_dynamic_params.pluck = 0.0;
            }
        }
    }
    video.video_analyse_params.pluck = pluck;
}

#[derive(Debug, PartialEq)]
pub enum SuperFLState {
    NotStart,   // 还没开始
    StartNow,   // 此处开始
    StartNotOk, // 开始了，但还没满足数量
    IsOk,       // 满足数量了，延续
    Finish,     // 检测到，结束
}
pub fn analyse_super_fl_local(video: &mut BaseVideo<Vec<Vec<i32>>>) {
    let mut comments = vec![];
    let event_min_num = 5;
    let euclidean_distance = 16;
    let mut anchor = 0;
    let mut counter = 0; //正在标雷、双击超过event_min_num总次数
    let mut state = SuperFLState::NotStart;
    let mut last_rc_num = 0; // 最后有几个右键
                             // let mut last_ide = 0;
    let Some(Event::Mouse(mut last_event)) = video.video_action_state_recorder[0].event.clone()
    else {
        panic!("expected mouse event");
    };
    let mut last_event_mouse_state = video.video_action_state_recorder[0].mouse_state;

    for ide in 1..video.video_action_state_recorder.len() {
        let vas = &mut video.video_action_state_recorder[ide];
        if let Some(Event::Mouse(mouse_event)) = &vas.event {
            if mouse_event.mouse == "mv" {
                continue;
            }
            let x = mouse_event.y as usize / video.cell_pixel_size as usize;
            let y = mouse_event.x as usize / video.cell_pixel_size as usize;
            let r_1 = last_event.y as usize / video.cell_pixel_size as usize;
            let c_1 = last_event.x as usize / video.cell_pixel_size as usize;
            // if mouse_event.mouse == "rc" || mouse_event.mouse == "rc"{
            //     println!("{:?}, {:?}, {:?}, {:?}", vas.time, vas.mouse_state, x, y);
                // println!("---{:?}", video.video_action_state_recorder[ide].useful_level);
            // }

            if mouse_event.mouse == "rc"
                && vas.useful_level == 1
                && vas.prior_game_board.as_ref().unwrap().borrow().game_board[x][y] == 10
            {
                // 正确的标雷
                match state {
                    SuperFLState::NotStart => {
                        state = SuperFLState::StartNow;
                        counter = 1;
                        last_rc_num = 1;
                        anchor = ide;
                        // println!("666");
                    }
                    SuperFLState::StartNow => {
                        state = SuperFLState::StartNotOk;
                        counter += 1;
                        last_rc_num += 1;
                    }
                    SuperFLState::StartNotOk | SuperFLState::IsOk => {
                        counter += 1;
                        last_rc_num += 1;
                    }
                    _ => {}
                }
            } else if vas.useful_level == 3 {
                // 正确的双击
                if !is_good_chording(
                    &vas.prior_game_board.as_ref().unwrap().borrow().game_board,
                    (x, y),
                ) {
                    match state {
                        SuperFLState::IsOk => {
                            counter -= last_rc_num;
                            state = SuperFLState::Finish;
                        }
                        _ => {
                            state = SuperFLState::NotStart;
                            counter = 0;
                            last_rc_num = 0;
                        }
                    }
                } else {
                    match state {
                        SuperFLState::StartNow => {
                            state = SuperFLState::StartNotOk;
                            counter += 1;
                            last_rc_num = 0;
                        }
                        SuperFLState::StartNotOk | SuperFLState::IsOk => {
                            counter += 1;
                            last_rc_num = 0;
                        }
                        _ => {}
                    }
                }
            } else if mouse_event.mouse == "lr"
                && (last_event_mouse_state == MouseState::DownUp
                    || last_event_mouse_state == MouseState::Chording)
                || mouse_event.mouse == "rr" && last_event_mouse_state == MouseState::Chording
            {
                // 左键或错误的右键或错误的双键
                match state {
                    SuperFLState::IsOk => {
                        counter -= last_rc_num;
                        state = SuperFLState::Finish;
                    }
                    _ => {
                        state = SuperFLState::NotStart;
                        counter = 0;
                        last_rc_num = 0;
                    }
                }
            }
            if (x as i32 - r_1 as i32) * (x as i32 - r_1 as i32)
                + (y as i32 - c_1 as i32) * (y as i32 - c_1 as i32)
                > euclidean_distance
            {
                match state {
                    SuperFLState::StartNotOk => {
                        state = SuperFLState::NotStart;
                        counter = 0;
                        last_rc_num = 0;
                    }
                    SuperFLState::IsOk => {
                        counter -= last_rc_num;
                        state = SuperFLState::Finish;
                    }
                    _ => {}
                }
            }
            if counter - last_rc_num >= event_min_num {
                match state {
                    SuperFLState::StartNow | SuperFLState::StartNotOk => {
                        state = SuperFLState::IsOk;
                    }
                    _ => {}
                }
            }
            match state {
                SuperFLState::Finish => {
                    comments.push((
                        anchor,
                        format!("feature: 教科书式的FL局部(步数{});", counter),
                    ));

                    // video.video_action_state_recorder[anchor].comments = format!(
                    //     "{}{}",
                    //     video.video_action_state_recorder[anchor].comments,
                    //     format!("feature: 教科书式的FL局部(步数{});", counter)
                    // );
                    state = SuperFLState::NotStart;
                }
                _ => {}
            }
            last_event = mouse_event.clone();
            last_event_mouse_state = vas.mouse_state;
            // println!("{:?}", video.video_action_state_recorder[last_ide].mouse_state);
        }
    }
    for comment in comments {
        video.video_action_state_recorder[comment.0].comments = video.video_action_state_recorder
            [comment.0]
            .comments
            .clone()
            + &comment.1;
    }
}
