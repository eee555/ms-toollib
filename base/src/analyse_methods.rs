use crate::algorithms::{solve_direct, solve_enumerate, solve_minus};
use crate::board;
use crate::utils::{refresh_matrix, refresh_matrixs};
use board::{AvfVideo, VideoEvent};
// 录像的事件分析。参与分析的录像必须已经计算出对应的数据。
// error: 高风险的猜雷（猜对概率0.05）√
// feature: 高难度的判雷√
// warning: 可以判雷时视野的转移
// feature: 双线操作
// feature: 破空（成功率0.98）
// error: 过于弯曲的鼠标轨迹(500%)√
// warning：弯曲的鼠标轨迹(200%)√
// warning: 可以判雷时选择猜雷√
// warning: 没有作用的操作
// suspect: 点击速度过快(0.01)
// suspect: 鼠标移动过快(2)
// suspect: 笔直的鼠标轨迹(101%)√
pub fn analyse_high_risk_guess(video: &mut AvfVideo) {
    let mut x = (video.events[video.events.len() - 1].y / 16) as usize;
    let mut y = (video.events[video.events.len() - 1].x / 16) as usize;
    let mut id = video.events.len() - 1;
    for ide in (0..video.events.len() - 1).rev() {
        if video.events[ide].useful_level >= 2 {
            let p = video.events[ide].posteriori_game_board.get_poss()[x][y];
            if p >= 0.51 {
                video.events[id].comments = format!(
                    "{}{}",
                    video.events[id].comments,
                    format!("error: 危险的猜雷(猜对概率{:.3});", 1.0 - p)
                );
                // println!(
                //     "{:?} => {:?}",
                //     video.events[id].time, video.events[id].comments
                // );
            }
            x = (video.events[ide].y / 16) as usize;
            y = (video.events[ide].x / 16) as usize;
            id = ide;
        }
    }
}

pub fn analyse_jump_judge(video: &mut AvfVideo) {
    // 功能：检测左键的跳判
    let mut id_last = 0;
    loop {
        if video.events[id_last].mouse != "lr" {
            id_last += 1;
        } else {
            break;
        }
    }
    // let mut tb = video.events[id_last].posteriori_game_board.game_board_marked.clone();
    let mut x;
    let mut y;
    for ide in 0..video.events.len() {
        x = (video.events[ide].y / 16) as usize;
        y = (video.events[ide].x / 16) as usize;
        if video.events[ide].useful_level >= 2 && video.events[ide].mouse == "lr" {
            if !video.events[id_last]
                .posteriori_game_board
                .get_basic_not_mine()
                .contains(&(x, y))
                && video.events[id_last]
                    .posteriori_game_board
                    .get_enum_not_mine()
                    .contains(&(x, y))
            {
                video.events[ide].comments = format!(
                    "{}{}",
                    video.events[ide].comments,
                    format!("feature: 高难度的判雷(左键);")
                );
                // println!(
                //     "{:?} => {:?}",
                //     video.events[ide].time, video.events[ide].comments
                // );
            }
        } else if video.events[ide].useful_level == 1 && video.events[ide].mouse == "rc" {
            if !video.events[id_last]
                .posteriori_game_board
                .get_basic_is_mine()
                .contains(&(x, y))
                && video.events[id_last]
                    .posteriori_game_board
                    .get_enum_is_mine()
                    .contains(&(x, y))
            {
                video.events[ide].comments = format!(
                    "{}{}",
                    video.events[ide].comments,
                    format!("feature: 高难度的判雷(标雷);")
                );
                // println!(
                //     "{:?} => {:?}",
                //     video.events[ide].time, video.events[ide].comments
                // );
            }
        }

        if video.events[ide].useful_level >= 2 {
            id_last = ide;
            // tb = video.events[id_last].posteriori_game_board.clone();
        }
    }
}

pub fn analyse_needless_guess(video: &mut AvfVideo) {
    let mut id_last = 0;
    loop {
        if video.events[id_last].mouse != "lr" {
            id_last += 1;
        } else {
            break;
        }
    }
    // let mut tb = video.events[id_last].posteriori_game_board.clone();
    let mut x;
    let mut y;
    for ide in 0..video.events.len() {
        if video.events[ide].useful_level >= 2 && video.events[ide].mouse == "lr" {
            x = (video.events[ide].y / 16) as usize;
            y = (video.events[ide].x / 16) as usize;

            if video.events[id_last].posteriori_game_board.get_poss()[x][y] > 0.0
                && !video.events[id_last]
                    .posteriori_game_board
                    .get_basic_not_mine()
                    .contains(&(x, y))
                && !video.events[id_last]
                    .posteriori_game_board
                    .get_enum_not_mine()
                    .contains(&(x, y))
            {
                video.events[ide].comments = format!(
                    "{}{}",
                    video.events[ide].comments,
                    format!("warning: 可以判雷时选择猜雷;")
                );
                // println!(
                //     "{:?} => {:?}",
                //     video.events[ide].time, video.events[ide].comments
                // );
            }
        }
        if video.events[ide].useful_level >= 2 {
            id_last = ide;
            // tb = video.events[id_last].posteriori_game_board.clone();
        }
    }
}

pub fn analyse_mouse_trace(video: &mut AvfVideo) {
    let mut click_last = (video.events[0].x as f64, video.events[0].y as f64);
    let mut click_last_id = 0;
    let mut move_last = (video.events[0].x as f64, video.events[0].y as f64);
    let mut path = 0.0;
    for ide in 0..video.events.len() {
        let current_x = video.events[ide].x as f64;
        let current_y = video.events[ide].y as f64;
        path += ((move_last.0 - current_x).powf(2.0) + (move_last.1 - current_y).powf(2.0)).sqrt();
        move_last = (current_x, current_y);
        if video.events[ide].mouse == "lr"
            || video.events[ide].mouse == "rc"
            || video.events[ide].mouse == "rr"
        {
            let path_straight = ((click_last.0 - current_x).powf(2.0)
                + (click_last.1 - current_y).powf(2.0))
            .sqrt();
            let k = path / path_straight;
            if k > 7.0 {
                video.events[click_last_id].comments = format!(
                    "{}{}",
                    video.events[click_last_id].comments,
                    format!("error: 过于弯曲的鼠标轨迹({:.0}%);", k * 100.0)
                );
                // println!(
                //     "{:?} => {:?}",
                //     video.events[click_last_id].time, video.events[click_last_id].comments
                // );
            } else if k > 3.5 {
                video.events[click_last_id].comments = format!(
                    "{}{}",
                    video.events[click_last_id].comments,
                    format!("warning: 弯曲的鼠标轨迹({:.0}%);", k * 100.0)
                );
                // println!(
                //     "{:?} => {:?}",
                //     video.events[click_last_id].time, video.events[click_last_id].comments
                // );
            } else if k < 1.01 {
                if k > 5.0 {
                    video.events[click_last_id].comments = format!(
                        "{}{}",
                        video.events[click_last_id].comments,
                        format!("suspect: 笔直的鼠标轨迹;")
                    );
                    // println!(
                    //     "{:?} => {:?}",
                    //     video.events[click_last_id].time, video.events[click_last_id].comments
                    // );
                }
            }
            click_last = (video.events[ide].x as f64, video.events[ide].y as f64);
            click_last_id = ide;
            path = 0.0;
        }
    }
}

pub fn analyse_vision_transfer(video: &mut AvfVideo) {
    let mut click_last = (video.events[0].y as f64, video.events[0].x as f64);
    let mut l_x = (video.events[0].y / 16) as usize;
    let mut l_y = (video.events[0].x / 16) as usize;
    let mut click_last_id = 0;
    for ide in 0..video.events.len() {
        if video.events[ide].useful_level >= 2 {
            // let xx = (video.events[ide].y / 16) as usize;
            // let yy = (video.events[ide].x / 16) as usize;
            let click_current = (video.events[ide].y as f64, video.events[ide].x as f64);
            if ((click_last.0 - click_current.0).powf(2.0)
                + (click_last.1 - click_current.1).powf(2.0))
            .sqrt()
                >= 112.0
            {
                let mut flag = false;
                for &(xxx, yyy) in video.events[click_last_id]
                    .posteriori_game_board
                    .get_basic_not_mine()
                {
                    if xxx <= l_x + 3 && xxx + 3 >= l_x && yyy <= l_y + 3 && yyy + 3 >= l_y {
                        flag = true;
                    }
                }
                for &(xxx, yyy) in video.events[click_last_id]
                    .posteriori_game_board
                    .get_enum_not_mine()
                {
                    if xxx <= l_x + 3 && xxx + 3 >= l_x && yyy <= l_y + 3 && yyy + 3 >= l_y {
                        flag = true;
                    }
                }
                if flag {
                    video.events[click_last_id].comments = format!(
                        "{}{}",
                        video.events[click_last_id].comments,
                        format!("warning: 可以判雷时视野的转移;")
                    );
                    // println!(
                    //     "{:?} => {:?}",
                    //     video.events[click_last_id].time, video.events[click_last_id].comments
                    // );
                }
            }
            click_last = click_current;
            l_x = (video.events[ide].y / 16) as usize;
            l_y = (video.events[ide].x / 16) as usize;
            click_last_id = ide;
        }
    }
}

pub fn analyse_survive_poss(video: &mut AvfVideo) {
    // 计算扫开这局的后验开率
    let mut s_poss = 1.0;
    let mut message = "luck: ".to_string();
    let mut click_last_id = 0;
    for ide in 2..video.events.len() {
        if video.events[ide].mouse == "lr" && video.events[ide].useful_level > 0 {
            let l_x = (video.events[ide].y / 16) as usize;
            let l_y = (video.events[ide].x / 16) as usize;
            let p = video.events[click_last_id].posteriori_game_board.get_poss()[l_x][l_y];
            if p > 0.0 && p < 1.0 {
                s_poss *= 1.0 - p;
                message.push_str(&format!("{:.3} * ", 1.0 - p));
                // println!("{:?} ==> {:?}", video.events[ide].time, 1.0 - p);
            }
            click_last_id = ide;
        }
    }
    message.pop();
    message.pop();
    message.push_str("= ");
    message.push_str(&format!("{:.6};", s_poss));
    video.events.last_mut().unwrap().comments = message;
}
