use crate::algorithms::{mark_board, solve_direct, solve_enumerate, solve_minus};
use crate::utils::{
    cal_all_solution, cal_board_numbers, c, refresh_board, refresh_matrixs, unsolvable_structure,
};
use crate::ENUM_LIMIT;

/// 生成 [0, n) 的随机整数。py/rs 用 rand，js 用 getrandom。
pub fn rand_below(n: usize) -> usize {
    if n <= 1 {
        return 0;
    }
    #[cfg(any(feature = "py", feature = "rs"))]
    {
        use rand::Rng;
        return rand::thread_rng().gen_range(0..n);
    }
    #[cfg(not(any(feature = "py", feature = "rs")))]
    {
        crate::utils::get_random_int(n)
    }
}

fn rand_float() -> f64 {
    #[cfg(any(feature = "py", feature = "rs"))]
    {
        use rand::Rng;
        return rand::thread_rng().gen::<f64>();
    }
    #[cfg(not(any(feature = "py", feature = "rs")))]
    {
        // js 平台用 getrandom 模拟 [0, 1) 浮点数
        const BASE: usize = 1 << 30;
        crate::utils::get_random_int(BASE) as f64 / BASE as f64
    }
}

fn comb_f64(n: usize, k: usize) -> f64 {
    if k > n {
        return 0.0;
    }
    c(n, k).into()
}

/// 按权重随机选择一个元素，权重必须为非负数且至少有一个为正。
fn weighted_choice(items: &[(usize, f64)]) -> Option<usize> {
    let total: f64 = items.iter().map(|x| x.1).sum();
    if total <= 0.0 || !total.is_finite() {
        return None;
    }
    let mut threshold = rand_float() * total;
    for (item, weight) in items {
        if threshold < *weight {
            return Some(*item);
        }
        threshold -= *weight;
    }
    items.last().map(|x| x.0)
}

/// 从不重复的池子中随机取 amount 个元素。
fn sample_without_replacement(pool: &[(usize, usize)], amount: usize) -> Vec<(usize, usize)> {
    let mut pool = pool.to_vec();
    let amount = amount.min(pool.len());
    for i in 0..amount {
        let j = i + rand_below(pool.len() - i);
        pool.swap(i, j);
    }
    pool.truncate(amount);
    pool
}

/// 判断从当前的游戏局面出发，是否能够通过逻辑推演把非雷格全部打开。
/// - 复用了 `is_solvable` 的推演循环，但起点是任意合法的游戏局面。
pub fn is_solvable_from_game_board(board: &Vec<Vec<i32>>, game_board: &Vec<Vec<i32>>) -> bool {
    if board.is_empty() || game_board.is_empty() {
        return false;
    }
    let row = board.len();
    let column = board[0].len();
    if row >= 4 && column >= 4 && unsolvable_structure(board) {
        return false;
    }
    let mut board_of_game = game_board.clone();
    for x in 0..row {
        for y in 0..column {
            let v = board_of_game[x][y];
            if v > 10 {
                // 清除玩家的标雷、标非雷、标红等标记
                board_of_game[x][y] = 10;
            } else if v < 10 && v >= 0 {
                // 已打开的数字必须与实际雷局一致
                if v != board[x][y] {
                    return false;
                }
            }
        }
    }
    if is_all_opened(&board_of_game, board) {
        return true;
    }
    loop {
        let (mut a_mats, mut xs, mut bs, _, _) = refresh_matrixs(&board_of_game);
        if bs.is_empty() {
            return is_all_opened(&board_of_game, board);
        }
        let ans = match solve_direct(&mut a_mats, &mut xs, &mut bs, &mut board_of_game) {
            Ok(a) => a,
            Err(_) => return false,
        };
        let not_mine;
        if ans.0.is_empty() && ans.1.is_empty() {
            let ans = match solve_minus(&mut a_mats, &mut xs, &mut bs, &mut board_of_game) {
                Ok(a) => a,
                Err(_) => return false,
            };
            if ans.0.is_empty() && ans.1.is_empty() {
                let ans = solve_enumerate(&a_mats, &xs, &bs);
                if ans.0.is_empty() && ans.1.is_empty() {
                    return false;
                }
                not_mine = ans.0;
                for (o, p) in ans.1 {
                    board_of_game[o][p] = 11;
                }
            } else {
                not_mine = ans.0;
            }
        } else {
            not_mine = ans.0;
        }
        if not_mine.is_empty() {
            // 本轮只推出了雷，没有可以直接打开的格子；继续下一轮（雷被标出后约束会变强）
            if is_all_opened(&board_of_game, board) {
                return true;
            }
            continue;
        }
        refresh_board(board, &mut board_of_game, not_mine);
        if is_all_opened(&board_of_game, board) {
            return true;
        }
    }
}

/// 当前局面是否存在可以用逻辑证明一定不是雷的未打开格子。
/// 这就是“有解”的判据：只要能推出至少一个安全格，玩家就不必猜。
pub fn has_provable_safe_cell(game_board: &Vec<Vec<i32>>) -> bool {
    if game_board.is_empty() {
        return false;
    }
    let mut gb = game_board.clone();
    for row in gb.iter_mut() {
        for v in row.iter_mut() {
            if *v > 10 {
                *v = 10;
            }
        }
    }
    let (mut a_mats, mut xs, mut bs, _, _) = refresh_matrixs(&gb);
    if bs.is_empty() {
        return false;
    }
    if let Ok(ans) = solve_direct(&mut a_mats, &mut xs, &mut bs, &mut gb) {
        if !ans.0.is_empty() {
            return true;
        }
    }
    if let Ok(ans) = solve_minus(&mut a_mats, &mut xs, &mut bs, &mut gb) {
        if !ans.0.is_empty() {
            return true;
        }
    }
    let (not_mine, _) = solve_enumerate(&a_mats, &xs, &bs);
    !not_mine.is_empty()
}

fn is_all_opened(game_board: &Vec<Vec<i32>>, board: &Vec<Vec<i32>>) -> bool {
    let row = board.len();
    let column = board[0].len();
    for i in 0..row {
        for j in 0..column {
            if board[i][j] != -1 && game_board[i][j] >= 10 {
                return false;
            }
        }
    }
    true
}

/// 在与 `game_board` 一致的所有雷局中随机采样一个，满足 `must_not_mine` / `must_mine`。
/// - `require_solvable` 为 true 时，只接受从当前游戏局面起可以逻辑推完的雷局。
/// - 重排前后总雷数不变（取自输入 `board`）。
/// - 找不到满足条件的雷局时返回 `None`。
pub fn relayout(
    board: &Vec<Vec<i32>>,
    game_board: &Vec<Vec<i32>>,
    must_not_mine: &[(usize, usize)],
    must_mine: &[(usize, usize)],
    require_solvable: bool,
    max_trials: usize,
) -> Option<Vec<Vec<i32>>> {
    if board.is_empty() || game_board.is_empty() {
        return None;
    }
    for p in must_mine {
        if must_not_mine.contains(p) {
            return None;
        }
    }
    let row = board.len();
    let column = board[0].len();
    let mut total_mine_count = 0;
    for i in 0..row {
        for j in 0..column {
            if board[i][j] == -1 {
                total_mine_count += 1;
            }
        }
    }
    let trials = max_trials.max(1);
    for _ in 0..trials {
        if let Some(candidate) = relocate_once(
            game_board,
            must_not_mine,
            must_mine,
            total_mine_count,
            row,
            column,
        ) {
            if !require_solvable || is_solvable_from_game_board(&candidate, game_board) {
                return Some(candidate);
            }
        }
    }
    None
}

fn relocate_once(
    game_board: &Vec<Vec<i32>>,
    must_not_mine: &[(usize, usize)],
    must_mine: &[(usize, usize)],
    total_mine_count: usize,
    row: usize,
    column: usize,
) -> Option<Vec<Vec<i32>>> {
    // 复制游戏局面，清除玩家标记后做一次快速判雷
    let mut gb = game_board.clone();
    for x in 0..row {
        for y in 0..column {
            if gb[x][y] > 10 {
                gb[x][y] = 10;
            }
        }
    }
    mark_board(&mut gb, false).ok()?;

    // 必然雷、必然非雷
    let mut fixed_mines = vec![];
    for x in 0..row {
        for y in 0..column {
            if gb[x][y] == 11 {
                fixed_mines.push((x, y));
            }
        }
    }
    if total_mine_count < fixed_mines.len() {
        return None;
    }

    let mut need_mine = vec![];
    for &(x, y) in must_mine {
        match gb[x][y] {
            10 => need_mine.push((x, y)),
            11 => {} // 已经是必然雷，条件天然满足
            _ => return None,
        }
    }
    let mut need_not_mine = vec![];
    for &(x, y) in must_not_mine {
        match gb[x][y] {
            10 => need_not_mine.push((x, y)),
            12 => {} // 已经是必然非雷，条件天然满足
            _ => return None,
        }
    }

    let (a_mats, xs, bs, _, _) = refresh_matrixs(&gb);

    // 逐块枚举，按雷数分组
    let mut block_vars: Vec<Vec<(usize, usize)>> = vec![];
    let mut block_groups: Vec<std::collections::HashMap<usize, Vec<Vec<u8>>>> = vec![];
    let mut all_block_vars = std::collections::HashSet::new();
    for i in 0..a_mats.len() {
        if xs[i].len() > ENUM_LIMIT {
            return None;
        }
        let solutions = cal_all_solution(&a_mats[i], &bs[i]);
        if solutions.is_empty() {
            return None;
        }
        let mut filtered = vec![];
        for sol in solutions {
            let mut ok = true;
            for (k, coord) in xs[i].iter().enumerate() {
                if need_mine.contains(coord) {
                    if sol[k] != 1 {
                        ok = false;
                        break;
                    }
                } else if need_not_mine.contains(coord) && sol[k] != 0 {
                    ok = false;
                    break;
                }
            }
            if ok {
                filtered.push(sol);
            }
        }
        if filtered.is_empty() {
            return None;
        }
        let mut groups: std::collections::HashMap<usize, Vec<Vec<u8>>> =
            std::collections::HashMap::new();
        for sol in filtered {
            let count = sol.iter().filter(|&&v| v == 1).count();
            groups.entry(count).or_default().push(sol);
        }
        for coord in &xs[i] {
            all_block_vars.insert(*coord);
        }
        block_vars.push(xs[i].clone());
        block_groups.push(groups);
    }

    let must_mine_set: std::collections::HashSet<(usize, usize)> =
        must_mine.iter().cloned().collect();
    let must_not_mine_set: std::collections::HashSet<(usize, usize)> =
        must_not_mine.iter().cloned().collect();

    // 自由格：未打开、不在任何约束块内
    let mut free_positions = vec![];
    for x in 0..row {
        for y in 0..column {
            if gb[x][y] == 10 && !all_block_vars.contains(&(x, y)) {
                free_positions.push((x, y));
            }
        }
    }
    let fixed_mine_set: std::collections::HashSet<(usize, usize)> =
        fixed_mines.iter().cloned().collect();
    // 约束在自由格上的部分直接固定
    let mut required_free_mines = 0;
    for &p in &must_mine_set {
        if fixed_mine_set.contains(&p) {
            // 已经是必然雷，条件天然满足
            continue;
        }
        if !all_block_vars.contains(&p) {
            if !free_positions.contains(&p) {
                return None;
            }
            required_free_mines += 1;
        }
    }
    let mut free_pool = vec![];
    for &p in &free_positions {
        if must_mine_set.contains(&p) || must_not_mine_set.contains(&p) {
            continue;
        }
        free_pool.push(p);
    }

    let total_remaining = total_mine_count - fixed_mines.len();
    let block_num = block_groups.len();
    let mut block_count_maps: Vec<std::collections::HashMap<usize, usize>> = vec![];
    for groups in &block_groups {
        let mut map = std::collections::HashMap::new();
        for (&count, sols) in groups {
            map.insert(count, sols.len());
        }
        block_count_maps.push(map);
    }

    // 前缀 DP：各块雷数组合
    let mut dp = vec![0.0f64; total_remaining + 1];
    dp[0] = 1.0;
    for count_map in &block_count_maps {
        let mut next = vec![0.0f64; total_remaining + 1];
        for prefix in 0..=total_remaining {
            if dp[prefix] == 0.0 {
                continue;
            }
            for (&mine_count, &sol_count) in count_map {
                if prefix + mine_count <= total_remaining {
                    next[prefix + mine_count] += dp[prefix] * sol_count as f64;
                }
            }
        }
        dp = next;
    }

    // 结合自由格组合数，得到每种块雷数的完整解权重
    let mut weights: Vec<(usize, f64)> = vec![];
    for block_mines in 0..=total_remaining {
        if dp[block_mines] == 0.0 {
            continue;
        }
        if total_remaining < block_mines + required_free_mines {
            continue;
        }
        let free_mines = total_remaining - block_mines - required_free_mines;
        if free_mines > free_pool.len() {
            continue;
        }
        let w = dp[block_mines] * comb_f64(free_pool.len(), free_mines);
        if w > 0.0 && w.is_finite() {
            weights.push((block_mines, w));
        }
    }
    let chosen_block_mines = weighted_choice(&weights)?;

    // 后缀 DP：逐块采样
    let mut suffix: Vec<Vec<f64>> = vec![vec![0.0f64; total_remaining + 1]; block_num + 1];
    suffix[block_num][0] = 1.0;
    for i in (0..block_num).rev() {
        let mut cur = vec![0.0f64; total_remaining + 1];
        for (&mine_count, &sol_count) in &block_count_maps[i] {
            for s in 0..=total_remaining {
                if suffix[i + 1][s] == 0.0 {
                    continue;
                }
                if mine_count + s <= total_remaining {
                    cur[mine_count + s] += sol_count as f64 * suffix[i + 1][s];
                }
            }
        }
        suffix[i] = cur;
    }

    let mut result = vec![vec![0; column]; row];
    for &(x, y) in &fixed_mines {
        result[x][y] = -1;
    }

    let mut remaining = chosen_block_mines;
    for i in 0..block_num {
        let mut candidates = vec![];
        for (&mine_count, sols) in &block_groups[i] {
            if mine_count > remaining {
                continue;
            }
            let tail = remaining - mine_count;
            let sw = suffix[i + 1][tail];
            if sw == 0.0 {
                continue;
            }
            candidates.push((mine_count, sols.len() as f64 * sw));
        }
        let mine_count = weighted_choice(&candidates)?;
        let sols = block_groups[i].get(&mine_count)?;
        let sol = &sols[rand_below(sols.len())];
        for (k, &(x, y)) in block_vars[i].iter().enumerate() {
            if sol[k] == 1 {
                result[x][y] = -1;
            }
        }
        remaining -= mine_count;
    }
    if remaining != 0 {
        return None;
    }

    // 自由格中必须为雷的部分
    for &p in &must_mine_set {
        if !all_block_vars.contains(&p) {
            result[p.0][p.1] = -1;
        }
    }
    // 剩余自由雷
    if total_remaining < chosen_block_mines + required_free_mines {
        return None;
    }
    let free_mines = total_remaining - chosen_block_mines - required_free_mines;
    if free_mines > free_pool.len() {
        return None;
    }
    for &(x, y) in &sample_without_replacement(&free_pool, free_mines) {
        result[x][y] = -1;
    }

    cal_board_numbers(&mut result);

    // 校验：总雷数、已打开数字均一致
    let mut new_total = 0;
    for i in 0..row {
        for j in 0..column {
            if result[i][j] == -1 {
                new_total += 1;
            }
            let v = gb[i][j];
            if v < 10 && v >= 0 && result[i][j] != v {
                return None;
            }
        }
    }
    if new_total != total_mine_count {
        return None;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_mines(board: &Vec<Vec<i32>>) -> usize {
        board
            .iter()
            .map(|row| row.iter().filter(|&&v| v == -1).count())
            .sum()
    }

    fn opened_numbers_match(board: &Vec<Vec<i32>>, game_board: &Vec<Vec<i32>>) -> bool {
        for i in 0..board.len() {
            for j in 0..board[0].len() {
                let v = game_board[i][j];
                if (0..10).contains(&v) && v != board[i][j] {
                    return false;
                }
            }
        }
        true
    }

    /// 8x8 的复杂局面，只打开 (4,4)（数字 1）
    fn complex_8x8() -> (Vec<Vec<i32>>, Vec<Vec<i32>>) {
        let board = vec![
            vec![1, 1, 1, 1, 1, 2, 2, 2],
            vec![1, -1, 1, 2, -1, 3, -1, -1],
            vec![1, 1, 1, 3, -1, 5, 3, 3],
            vec![0, 0, 0, 2, -1, 3, -1, 1],
            vec![0, 0, 0, 1, 1, 3, 2, 2],
            vec![0, 0, 0, 0, 0, 2, -1, 2],
            vec![0, 1, 1, 1, 0, 2, -1, 2],
            vec![0, 1, -1, 1, 0, 1, 1, 1],
        ];
        let mut game_board = vec![vec![10; 8]; 8];
        game_board[4][4] = 1;
        (board, game_board)
    }

    #[test]
    fn relayout_is_consistent() {
        let (board, game_board) = complex_8x8();
        let total = count_mines(&board);
        let must_mine = vec![(3, 3)];
        let must_not_mine = vec![(3, 4)];
        let new_board = relayout(&board, &game_board, &must_not_mine, &must_mine, false, 20)
            .expect("应当能找到满足条件的雷局");
        assert_eq!(count_mines(&new_board), total);
        assert!(opened_numbers_match(&new_board, &game_board));
        assert_eq!(new_board[3][3], -1);
        assert_ne!(new_board[3][4], -1);
        // (4,4) 的约束是 8 邻格中恰好 1 雷
        let mut around = 0;
        for i in 3..6 {
            for j in 3..6 {
                if (i, j) != (4, 4) && new_board[i][j] == -1 {
                    around += 1;
                }
            }
        }
        assert_eq!(around, 1);
    }

    /// 5x5，中心 1 雷
    fn solvable_5x5() -> Vec<Vec<i32>> {
        vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 1, 1, 1, 0],
            vec![0, 1, -1, 1, 0],
            vec![0, 1, 1, 1, 0],
            vec![0, 0, 0, 0, 0],
        ]
    }

    #[test]
    fn solvability_check_works() {
        let board = solvable_5x5();
        let mut game_board = vec![vec![10; 5]; 5];
        refresh_board(&board, &mut game_board, vec![(0, 0)]);
        assert!(is_solvable_from_game_board(&board, &game_board));

        // 复杂的 8x8 只打开一个 1，无法推完
        let (board, game_board) = complex_8x8();
        assert!(!is_solvable_from_game_board(&board, &game_board));
    }
}
