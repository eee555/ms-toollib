use super::{BaseVideo, Event, VideoActionStateRecorder, VideoAnalyseParams};
use crate::algorithms::{cal_probability_cells_not_mine, mark_board};

/// 同时维护加权累计值和归一化值，避免用已下溢的Luck做除法。
struct Accumulator {
    params: VideoAnalyseParams,
    failure_pluck: f64,
}

impl Accumulator {
    fn new(failure_pluck: f64) -> Self {
        Self {
            params: VideoAnalyseParams {
                pluck: 0.0,
                expected_bbbv_solved_per_attempt: 0.0,
                expected_bbbv_solved_per_win: 0.0,
                expected_time_per_attempt: 0.0,
                expected_time_per_win: 0.0,
            },
            failure_pluck,
        }
    }

    fn advance_time(&mut self, seconds: f64) {
        self.params.expected_time_per_attempt += self.params.get_luck() * seconds;
        self.params.expected_time_per_win += seconds;
    }

    fn advance_action(&mut self, safety: f64, solved: usize) {
        if safety <= 0.0 || self.params.pluck == self.failure_pluck {
            self.params.pluck = self.failure_pluck;
            self.params.expected_bbbv_solved_per_win = f64::INFINITY;
            self.params.expected_time_per_win = f64::INFINITY;
            return;
        }
        if safety != 1.0 {
            self.params.pluck -= safety.log10();
        }
        self.params.expected_bbbv_solved_per_attempt += self.params.get_luck() * solved as f64;
        self.params.expected_bbbv_solved_per_win =
            self.params.expected_bbbv_solved_per_win / safety + solved as f64;
        self.params.expected_time_per_win /= safety;
    }
}

fn safety_probability(
    record: &VideoActionStateRecorder,
    height: usize,
    width: usize,
    cell_pixel_size: u8,
    mine_num: usize,
) -> f64 {
    let Some(Event::Mouse(mouse)) = &record.event else {
        return 1.0;
    };
    if record.useful_level == 4 {
        return 0.0;
    }
    if !matches!(record.useful_level, 2 | 3) {
        return 1.0;
    }
    let r = mouse.y as usize / cell_pixel_size as usize;
    let c = mouse.x as usize / cell_pixel_size as usize;
    let prior = record.prior_game_board.as_ref().unwrap();
    if record.useful_level == 2 {
        // 沿用pluck的首击豁免及单格概率计算。
        let safety = 1.0 - prior.borrow_mut().get_poss()[r][c];
        return if safety < 1.0 { safety } else { 1.0 };
    }
    let mut board = prior.borrow().game_board.clone();
    let mut cells = Vec::new();
    for row in r.saturating_sub(1)..(r + 2).min(height) {
        for column in c.saturating_sub(1)..(c + 2).min(width) {
            if board[row][column] == 10 {
                cells.push((row, column));
            }
        }
    }
    mark_board(&mut board, true).unwrap();
    cal_probability_cells_not_mine(&board, mine_num as f64, &cells)
}

/// 共用录像预计算与游戏结束后的按需计算；保留两条旧路径的失败pluck标记。
pub(super) fn analyse<T>(video: &mut BaseVideo<T>, failure_pluck: f64) {
    let mut accumulator = Accumulator::new(failure_pluck);
    let mut start_time = None;
    let mut previous_time = 0.0;
    let mut previous_solved = 0;
    let duration = video.game_dynamic_params.rtime.max(0.0);
    for record in &mut video.video_action_state_recorder {
        let is_mouse = matches!(&record.event, Some(Event::Mouse(_)));
        let first_click = start_time.is_none() && is_mouse && record.useful_level == 2;
        if first_click || (start_time.is_none() && is_mouse && record.useful_level == 4) {
            start_time = Some(record.time);
        }
        if let Some(start) = start_time {
            let elapsed = (record.time - start).clamp(0.0, duration);
            // 等待、移动和无效操作也耗时；本次风险在这段时间支付后才发生。
            accumulator.advance_time((elapsed - previous_time).max(0.0));
            previous_time = previous_time.max(elapsed);
            let solved = if is_mouse {
                record
                    .key_dynamic_params
                    .bbbv_solved
                    .saturating_sub(previous_solved)
            } else {
                0
            };
            if is_mouse {
                previous_solved = record.key_dynamic_params.bbbv_solved;
            }
            let safety = if first_click {
                1.0
            } else {
                safety_probability(
                    record,
                    video.height,
                    video.width,
                    video.cell_pixel_size,
                    video.mine_num,
                )
            };
            accumulator.advance_action(safety, solved);
        }
        record
            .key_dynamic_params
            .set_pluck_metrics(accumulator.params);
    }
    if start_time.is_some() {
        // 点脸等结束方式可能没有末尾鼠标事件，仍计入实际游戏结束前的时间。
        accumulator.advance_time((duration - previous_time).max(0.0));
    }
    video.video_analyse_params = accumulator.params;
}

#[cfg(test)]
mod tests {
    use super::Accumulator;

    #[test]
    fn time_is_paid_before_a_risky_action() {
        let mut accumulator = Accumulator::new(f64::MAX);
        accumulator.advance_action(1.0, 1);
        accumulator.advance_time(10.0);
        accumulator.advance_action(0.5, 1);
        assert_eq!(accumulator.params.expected_bbbv_solved_per_attempt, 1.5);
        assert_eq!(accumulator.params.expected_bbbv_solved_per_win, 3.0);
        assert_eq!(accumulator.params.expected_time_per_attempt, 10.0);
        assert_eq!(accumulator.params.expected_time_per_win, 20.0);
    }

    #[test]
    fn normalization_does_not_divide_an_underflowed_luck() {
        let mut accumulator = Accumulator::new(f64::MAX);
        accumulator.advance_action(1e-200, 0);
        accumulator.advance_action(1e-200, 0);
        accumulator.advance_action(1.0, 2);
        assert_eq!(accumulator.params.get_luck(), 0.0);
        assert_eq!(accumulator.params.expected_bbbv_solved_per_attempt, 0.0);
        assert_eq!(accumulator.params.expected_bbbv_solved_per_win, 2.0);
        assert_eq!(accumulator.params.expected_time_per_win, 0.0);
    }
}
