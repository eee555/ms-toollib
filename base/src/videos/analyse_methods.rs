use crate::algorithms::{cal_probability_cells_not_mine, mark_board};
use crate::utils::is_good_chording;
use crate::videos::base_video::BaseVideo;
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
    let mut x;
    let mut y;
    for ide in 2..video.video_action_state_recorder.len() {
        x = (video.video_action_state_recorder[ide].y / video.cell_pixel_size as u16) as usize;
        y = (video.video_action_state_recorder[ide].x / video.cell_pixel_size as u16) as usize;
        if video.video_action_state_recorder[ide].useful_level >= 2 {
            let p = video.video_action_state_recorder[ide]
                .prior_game_board
                .as_ref()
                .unwrap()
                .borrow_mut()
                .get_poss()[x][y];
            if p >= 0.51 {
                video.video_action_state_recorder[ide].comments = format!(
                    "{}{}",
                    video.video_action_state_recorder[ide].comments,
                    format!("error: 危险的猜雷(猜对概率{:.3});", 1.0 - p)
                );
            }
        }
    }
}

pub fn analyse_jump_judge(video: &mut BaseVideo<Vec<Vec<i32>>>) {
    // 功能：检测左键或右键的跳判
    let mut x;
    let mut y;
    for ide in 2..video.video_action_state_recorder.len() {
        x = (video.video_action_state_recorder[ide].y / video.cell_pixel_size as u16) as usize;
        y = (video.video_action_state_recorder[ide].x / video.cell_pixel_size as u16) as usize;
        if video.video_action_state_recorder[ide].useful_level >= 2
            && video.video_action_state_recorder[ide].mouse == "lr"
        {
            if !video.video_action_state_recorder[ide]
                .prior_game_board
                .as_ref()
                .unwrap()
                .borrow_mut()
                .get_basic_not_mine()
                .contains(&(x, y))
                && video.video_action_state_recorder[ide]
                    .prior_game_board
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .get_enum_not_mine()
                    .contains(&(x, y))
            {
                video.video_action_state_recorder[ide].comments = format!(
                    "{}{}",
                    video.video_action_state_recorder[ide].comments,
                    format!("feature: 高难度的判雷(左键);")
                );
            }
        } else if video.video_action_state_recorder[ide].useful_level == 1
            && video.video_action_state_recorder[ide].mouse == "rc"
        {
            if !video.video_action_state_recorder[ide]
                .prior_game_board
                .as_ref()
                .unwrap()
                .borrow_mut()
                .get_basic_is_mine()
                .contains(&(x, y))
                && video.video_action_state_recorder[ide]
                    .prior_game_board
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .get_enum_is_mine()
                    .contains(&(x, y))
            {
                video.video_action_state_recorder[ide].comments = format!(
                    "{}{}",
                    video.video_action_state_recorder[ide].comments,
                    format!("feature: 高难度的判雷(标雷);")
                );
            }
        }
    }
}

// 猜雷时，假如周围5*5范围内有可判的，引发可以判雷时选择猜雷
pub fn analyse_needless_guess(video: &mut BaseVideo<Vec<Vec<i32>>>) {
    let mut x;
    let mut y;
    'outer: for ide in 2..video.video_action_state_recorder.len() {
        if video.video_action_state_recorder[ide].useful_level >= 2
            && video.video_action_state_recorder[ide].mouse == "lr"
        {
            x = (video.video_action_state_recorder[ide].y / video.cell_pixel_size as u16) as usize;
            y = (video.video_action_state_recorder[ide].x / video.cell_pixel_size as u16) as usize;

            if video.video_action_state_recorder[ide]
                .prior_game_board
                .as_ref()
                .unwrap()
                .borrow_mut()
                .get_poss()[x][y]
                > 0.0
            {
                for m in max(2, x) - 2..min(video.height, x + 3) {
                    for n in max(2, y) - 2..min(video.width, y + 3) {
                        if video.video_action_state_recorder[ide]
                            .prior_game_board
                            .as_ref()
                            .unwrap()
                            .borrow_mut()
                            .get_basic_not_mine()
                            .contains(&(m, n))
                            || video.video_action_state_recorder[ide]
                                .prior_game_board
                                .as_ref()
                                .unwrap()
                                .borrow_mut()
                                .get_enum_not_mine()
                                .contains(&(m, n))
                        {
                            video.video_action_state_recorder[ide].comments = format!(
                                "{}{}",
                                video.video_action_state_recorder[ide].comments,
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

pub fn analyse_mouse_trace(video: &mut BaseVideo<Vec<Vec<i32>>>) {
    let mut click_last = (
        video.video_action_state_recorder[0].x as f64,
        video.video_action_state_recorder[0].y as f64,
    );
    let mut click_last_id = 0;
    let mut move_last = (
        video.video_action_state_recorder[0].x as f64,
        video.video_action_state_recorder[0].y as f64,
    );
    let mut path = 0.0;
    for ide in 0..video.video_action_state_recorder.len() {
        let current_x = video.video_action_state_recorder[ide].x as f64;
        let current_y = video.video_action_state_recorder[ide].y as f64;
        path += ((move_last.0 - current_x).powf(2.0) + (move_last.1 - current_y).powf(2.0)).sqrt();
        move_last = (current_x, current_y);
        if video.video_action_state_recorder[ide].mouse == "lr"
            || video.video_action_state_recorder[ide].mouse == "rc"
            || video.video_action_state_recorder[ide].mouse == "rr"
        {
            let path_straight = ((click_last.0 - current_x).powf(2.0)
                + (click_last.1 - current_y).powf(2.0))
            .sqrt();
            let k = path / path_straight;
            if k > 7.0 {
                video.video_action_state_recorder[click_last_id].comments = format!(
                    "{}{}",
                    video.video_action_state_recorder[click_last_id].comments,
                    format!("error: 过于弯曲的鼠标轨迹({:.0}%);", k * 100.0)
                );
                // println!(
                //     "{:?} => {:?}",
                //     video.video_action_state_recorder[click_last_id].time, video.video_action_state_recorder[click_last_id].comments
                // );
            } else if k > 3.5 {
                video.video_action_state_recorder[click_last_id].comments = format!(
                    "{}{}",
                    video.video_action_state_recorder[click_last_id].comments,
                    format!("warning: 弯曲的鼠标轨迹({:.0}%);", k * 100.0)
                );
                // println!(
                //     "{:?} => {:?}",
                //     video.video_action_state_recorder[click_last_id].time, video.video_action_state_recorder[click_last_id].comments
                // );
            } else if k < 1.01 {
                if k > 5.0 {
                    video.video_action_state_recorder[click_last_id].comments = format!(
                        "{}{}",
                        video.video_action_state_recorder[click_last_id].comments,
                        format!("suspect: 笔直的鼠标轨迹;")
                    );
                    // println!(
                    //     "{:?} => {:?}",
                    //     video.video_action_state_recorder[click_last_id].time, video.video_action_state_recorder[click_last_id].comments
                    // );
                }
            }
            click_last = (
                video.video_action_state_recorder[ide].x as f64,
                video.video_action_state_recorder[ide].y as f64,
            );
            click_last_id = ide;
            path = 0.0;
        }
    }
}

// bug
pub fn analyse_vision_transfer(video: &mut BaseVideo<Vec<Vec<i32>>>) {
    let mut click_last = (
        video.video_action_state_recorder[0].y as f64,
        video.video_action_state_recorder[0].x as f64,
    );
    let mut l_x = (video.video_action_state_recorder[0].y / video.cell_pixel_size as u16) as usize;
    let mut l_y = (video.video_action_state_recorder[0].x / video.cell_pixel_size as u16) as usize;
    let mut click_last_id = 0;
    for ide in 0..video.video_action_state_recorder.len() {
        if video.video_action_state_recorder[ide].useful_level >= 2 {
            // let xx = (video.video_action_state_recorder[ide].y / video.cell_pixel_size) as usize;
            // let yy = (video.video_action_state_recorder[ide].x / video.cell_pixel_size) as usize;
            let click_current = (
                video.video_action_state_recorder[ide].y as f64,
                video.video_action_state_recorder[ide].x as f64,
            );
            if ((click_last.0 - click_current.0).powf(2.0)
                + (click_last.1 - click_current.1).powf(2.0))
            .sqrt()
                / video.cell_pixel_size as f64
                >= 6.0
            {
                let mut flag = false;
                for &(xxx, yyy) in video.video_action_state_recorder[ide]
                    .prior_game_board
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .get_basic_not_mine()
                {
                    if xxx <= l_x + 3 && xxx + 3 >= l_x && yyy <= l_y + 3 && yyy + 3 >= l_y {
                        flag = true;
                    }
                }
                for &(xxx, yyy) in video.video_action_state_recorder[ide]
                    .prior_game_board
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .get_enum_not_mine()
                {
                    if xxx <= l_x + 3 && xxx + 3 >= l_x && yyy <= l_y + 3 && yyy + 3 >= l_y {
                        flag = true;
                    }
                }
                if flag {
                    video.video_action_state_recorder[click_last_id].comments = format!(
                        "{}{}",
                        video.video_action_state_recorder[click_last_id].comments,
                        format!("warning: 可以判雷时视野的转移;")
                    );
                    // println!(
                    //     "{:?} => {:?}",
                    //     video.video_action_state_recorder[click_last_id].time, video.video_action_state_recorder[click_last_id].comments
                    // );
                }
            }
            click_last = click_current;
            l_x =
                (video.video_action_state_recorder[ide].y / video.cell_pixel_size as u16) as usize;
            l_y =
                (video.video_action_state_recorder[ide].x / video.cell_pixel_size as u16) as usize;
            click_last_id = ide;
        }
    }
}

/// 计算扫开这局的后验开率
/// 不计算comment，修改pluck参数
pub fn analyse_survive_poss(video: &mut BaseVideo<Vec<Vec<i32>>>) {
    let mut pluck = 0.0;
    let mut has_begin = false;
    for vas in video.video_action_state_recorder.iter_mut() {
        if vas.useful_level == 2 {
            // 有效的左键
            if !has_begin {
                has_begin = true;
                vas.key_dynamic_params.pluck = Some(0.0);
                continue;
            }
            let x = (vas.y / video.cell_pixel_size as u16) as usize;
            let y = (vas.x / video.cell_pixel_size as u16) as usize;
            // 安全的概率
            let p = 1.0
                - vas
                    .prior_game_board
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .get_poss()[x][y];
            if p <= 0.0 || pluck == f64::MAX {
                pluck = f64::MAX;
            } else if p < 1.0 {
                pluck -= p.log10();
            }
        } else if vas.useful_level == 3 {
            // 有效的双键
            let x = (vas.y / video.cell_pixel_size as u16) as usize;
            let y = (vas.x / video.cell_pixel_size as u16) as usize;
            let mut game_board_clone = vas
                .prior_game_board
                .as_ref()
                .unwrap()
                .borrow_mut()
                .game_board
                .clone();
            let _ = mark_board(&mut game_board_clone, true).unwrap();
            let mut chording_cells = vec![];
            for m in max(1, x) - 1..min(video.height, x + 2) {
                for n in max(1, y) - 1..min(video.width, y + 2) {
                    if game_board_clone[m][n] == 10 {
                        chording_cells.push((m, n));
                    }
                }
            }
            // 安全的概率
            let p = cal_probability_cells_not_mine(
                &game_board_clone,
                video.mine_num as f64,
                &chording_cells,
            );
            if p >= 1.0 || pluck == f64::MAX {
                pluck = f64::MAX;
            } else if p > 0.0 {
                pluck -= p.log10();
            }
        }

        if has_begin {
            vas.key_dynamic_params.pluck = Some(pluck);
        } else {
            vas.key_dynamic_params.pluck = Some(0.0);
        }
    }
    video.video_analyse_params.pluck = Some(pluck);
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
    let event_min_num = 5;
    let euclidean_distance = 16;
    let mut anchor = 0;
    let mut counter = 0; //正在标雷、双击超过event_min_num总次数
    let mut state = SuperFLState::NotStart;
    let mut last_rc_num = 0; // 最后有几个右键
    let mut last_ide = 0;
    for ide in 1..video.video_action_state_recorder.len() {
        if video.video_action_state_recorder[ide].mouse == "mv" {
            continue;
        }
        let x = video.video_action_state_recorder[ide].y as usize / video.cell_pixel_size as usize;
        let y = video.video_action_state_recorder[ide].x as usize / video.cell_pixel_size as usize;
        let x_1 =
            video.video_action_state_recorder[last_ide].y as usize / video.cell_pixel_size as usize;
        let y_1 =
            video.video_action_state_recorder[last_ide].x as usize / video.cell_pixel_size as usize;
        // if video.video_action_state_recorder[ide].mouse == "lr" || video.video_action_state_recorder[ide].mouse == "rr"{
        //     println!("{:?}+++{:?}", video.video_action_state_recorder[last_ide].time, video.video_action_state_recorder[last_ide].mouse_state);
        //     // println!("---{:?}", video.video_action_state_recorder[ide].useful_level);
        // }

        if video.video_action_state_recorder[ide].mouse == "rc"
            && video.video_action_state_recorder[ide]
                .prior_game_board
                .as_ref()
                .unwrap()
                .borrow()
                .game_board[x][y]
                == 10
            && video.video_action_state_recorder[ide].useful_level == 1
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
        } else if video.video_action_state_recorder[ide].useful_level == 3 {
            // 正确的双击
            if !is_good_chording(
                &video.video_action_state_recorder[ide]
                    .prior_game_board
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .game_board,
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
        } else if video.video_action_state_recorder[ide].mouse == "lr"
            && (video.video_action_state_recorder[last_ide].mouse_state == MouseState::DownUp
                || video.video_action_state_recorder[last_ide].mouse_state == MouseState::Chording)
            || video.video_action_state_recorder[ide].mouse == "rr"
                && video.video_action_state_recorder[last_ide].mouse_state == MouseState::Chording
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
        if (x as i32 - x_1 as i32) * (x as i32 - x_1 as i32)
            + (y as i32 - y_1 as i32) * (y as i32 - y_1 as i32)
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
                video.video_action_state_recorder[anchor].comments = format!(
                    "{}{}",
                    video.video_action_state_recorder[anchor].comments,
                    format!("feature: 教科书式的FL局部(步数{});", counter)
                );
                state = SuperFLState::NotStart;
            }
            _ => {}
        }
        last_ide = ide;
        // println!("{:?}", video.video_action_state_recorder[last_ide].mouse_state);
    }
}
