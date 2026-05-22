use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::cmp::{max, min};
use crate::safe_board;

/// 计算局面的 ZiNi 值，PTTACGfans算法（估计最少点击次数）。
///
/// ZiNi 是一种贪心蒙特卡洛算法，估计扫完一个局面所需的最少点击次数（左键 + 右键 + 双击）。由 Elmar Zimmermann 和 Christoph Nikolaus 提出，取二人姓名首字母得名。PTTACGfans 做的是增量改进（两步式、混合式、路径优化）。
/// PTTACGfans算法开源在[此处](https://github.com/PTTACGfans/Minesweeper-ZiNi-Calculator)
/// 文档在[此处](https://docs.google.com/document/d/1Ve1gfaxZcgabvkAusIDzPVK0RMSpCdmgD_8TruhE28g/edit?tab=t.0#heading=h.tujz7cf074cp)
/// 
/// - `board`: 真实局面，`board[row][col]`，`-1` 为雷，`0~8` 为数字
/// - `loop_count`: 蒙特卡洛模拟次数。根据原文档，推荐值100
///
/// 返回找到的最小 ZiNi 值。
pub fn cal_zini<T>(board: &T, loop_count: usize) -> usize
where
    T: std::ops::Index<usize> + safe_board::BoardSize,
    T::Output: std::ops::Index<usize, Output = i32>,
{
    // 先把传入的任意实现了 BoardSize/Index 的局面转换为 Vec<Vec<i32>>
    let row = board.get_row();
    let col = board.get_column();
    let mut vboard: Vec<Vec<i32>> = vec![vec![0i32; col]; row];
    for r in 0..row {
        for c in 0..col {
            vboard[r][c] = board[r][c];
        }
    }

    let (opening_count, groups) = calculate_groups(&vboard);
    let seed = board_to_seed(&vboard);
    let mut best_zini = usize::MAX;

    for i in 0..loop_count {
        let two_step = i % 2 == 1 && loop_count > 100;
        let wide_candidate = i % 100 == 99 && loop_count > 100;

        let mut rng: StdRng = SeedableRng::seed_from_u64(seed.wrapping_add(i as u64));
        let z = random_zini(&vboard, &groups, opening_count, two_step, wide_candidate, &mut rng);
        if z < best_zini {
            best_zini = z;
        }
    }
    best_zini
}

// 将局面用作随机数种子，获得确定性
fn board_to_seed(board: &Vec<Vec<i32>>) -> u64 {
    let mut seed: u64 = 0;
    for row in board.iter() {
        for &v in row.iter() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(v as u64);
        }
    }
    seed
}

fn calculate_groups(board: &Vec<Vec<i32>>) -> (usize, Vec<Vec<usize>>) {
    let row = board.len();
    let col = board[0].len();
    let mut groups = vec![vec![0usize; col]; row];
    let mut opening_count = 0;

    // 第1步：为 0 格做泛洪分组（数字 0 代表空，相邻的 0 属同一组）
    for r in 0..row {
        for c in 0..col {
            if groups[r][c] == 0 && board[r][c] == 0 {
                opening_count += 1;
                flood_fill_zero(board, &mut groups, r, c, opening_count);
            }
        }
    }

    // 第2步：将与 0 相邻的数字格子归到对应的空组
    for r in 0..row {
        for c in 0..col {
            if groups[r][c] != 0 || board[r][c] == -1 {
                continue;
            }
            'outer: for dr in max(1, r) - 1..min(row, r + 2) {
                for dc in max(1, c) - 1..min(col, c + 2) {
                    if groups[dr][dc] != 0 && board[dr][dc] == 0 {
                        groups[r][c] = groups[dr][dc];
                        break 'outer;
                    }
                }
            }
        }
    }

    // 第3步：剩余非雷格子各取一个新组号
    let mut next_group = opening_count;
    for r in 0..row {
        for c in 0..col {
            if groups[r][c] == 0 && board[r][c] != -1 {
                next_group += 1;
                groups[r][c] = next_group;
            }
        }
    }

    (opening_count, groups)
}

fn flood_fill_zero(
    board: &Vec<Vec<i32>>,
    groups: &mut Vec<Vec<usize>>,
    r: usize,
    c: usize,
    gid: usize,
) {
    let row = board.len();
    let col = board[0].len();
    groups[r][c] = gid;
    for dr in max(1, r) - 1..min(row, r + 2) {
        for dc in max(1, c) - 1..min(col, c + 2) {
            if groups[dr][dc] == 0 && board[dr][dc] == 0 {
                flood_fill_zero(board, groups, dr, dc, gid);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 核心模拟
// ---------------------------------------------------------------------------

fn random_zini(
    board: &Vec<Vec<i32>>,
    groups: &Vec<Vec<usize>>,
    opening_count: usize,
    two_step: bool,
    wide_candidate: bool,
    rng: &mut StdRng,
) -> usize {
    let row = board.len();
    let col = board[0].len();
    let mut is_open = vec![vec![false; col]; row];
    let mut all_open = vec![vec![false; col]; row];
    let mut zini = 0;

    loop {
        let mut candidates: Vec<(usize, usize)> = Vec::new();
        let mut max_premium: isize = -10;

        for r in 0..row {
            for c in 0..col {
                if all_open[r][c] || board[r][c] == -1 {
                    continue;
                }

                let (opt_prem, _, have_unopen) = calc_premium(
                    board,
                    groups,
                    opening_count,
                    &is_open,
                    r,
                    c,
                    two_step,
                    wide_candidate,
                );

                if !have_unopen {
                    all_open[r][c] = true;
                }

                if let Some(prem) = opt_prem {
                    if prem > max_premium {
                        max_premium = prem;
                        candidates.clear();
                        candidates.push((r, c));
                    } else if prem == max_premium {
                        candidates.push((r, c));
                    }
                }
            }
        }

        if candidates.is_empty() {
            break;
        }

        let idx = rng.gen_range(0..candidates.len());
        let (cr, cc) = candidates[idx];

        if max_premium >= 0 {
            // ---- premium >= 0：双击 ----
            if !is_open[cr][cc] {
                zini += 1;
                // 找周围已开格子能否通过双击打开此格
                let chord_info = best_chord_from_neighbor(board, groups, opening_count, &is_open, cr, cc);

                if let Some((br, bc)) = chord_info {
                    // 从邻居双击
                    chord_at(&mut is_open, board, groups, br, bc, &mut zini);
                    if all_non_mine_open(&is_open, board) {
                        break;
                    }
                    continue;
                } else {
                    // 直接左键
                    if board[cr][cc] == 0 {
                        open_opening(&mut is_open, board, groups, cr, cc);
                    } else {
                        is_open[cr][cc] = true;
                    }
                }
            }

            // 双击当前格
            chord_at(&mut is_open, board, groups, cr, cc, &mut zini);
        } else {
            // ---- premium < 0：随机左键 ----
            let mut unopened: Vec<(usize, usize)> = Vec::new();
            for r in 0..row {
                for c in 0..col {
                    if board[r][c] != -1 && !is_open[r][c] {
                        if board[r][c] == 0 {
                            unopened.push((r, c));
                        } else if groups[r][c] > opening_count {
                            unopened.push((r, c));
                        }
                    }
                }
            }

            if unopened.is_empty() {
                break;
            }

            let pick = rng.gen_range(0..unopened.len());
            let (pr, pc) = unopened[pick];
            zini += 1;
            if board[pr][pc] == 0 {
                open_opening(&mut is_open, board, groups, pr, pc);
            } else {
                is_open[pr][pc] = true;
            }
        }

        if all_non_mine_open(&is_open, board) {
            break;
        }
    }

    zini
}

/// 在 (r,c) 执行双击：标出周围雷、打开非雷格子。
fn chord_at(
    is_open: &mut Vec<Vec<bool>>,
    board: &Vec<Vec<i32>>,
    groups: &Vec<Vec<usize>>,
    r: usize,
    c: usize,
    zini: &mut usize,
) {
    let row = board.len();
    let col = board[0].len();
    let mut flag_count: usize = 0;
    for dr in max(1, r) - 1..min(row, r + 2) {
        for dc in max(1, c) - 1..min(col, c + 2) {
            if is_open[dr][dc] {
                continue;
            }
            if board[dr][dc] == -1 {
                flag_count += 1;
                is_open[dr][dc] = true;
            } else if board[dr][dc] == 0 {
                open_opening(is_open, board, groups, dr, dc);
            } else {
                is_open[dr][dc] = true;
            }
        }
    }
    *zini += 1 + flag_count;
}

/// 判断是否可以从 (cr,cc) 的已开邻居双击来打开 (cr,cc)。
/// 返回最优的邻居坐标。
fn best_chord_from_neighbor(
    board: &Vec<Vec<i32>>,
    groups: &Vec<Vec<usize>>,
    opening_count: usize,
    is_open: &Vec<Vec<bool>>,
    cr: usize,
    cc: usize,
) -> Option<(usize, usize)> {
    let row = board.len();
    let col = board[0].len();
    let mut best: Option<(usize, usize, isize, isize)> = None;

    for dr in max(1, cr) - 1..min(row, cr + 2) {
        for dc in max(1, cc) - 1..min(col, cc + 2) {
            if dr == cr && dc == cc {
                continue;
            }
            if !is_open[dr][dc] || board[dr][dc] == -1 {
                continue;
            }

            let mut extra_cell: isize = 0;
            let mut extra_3bv: isize = 0;
            for nr in max(1, dr) - 1..min(row, dr + 2) {
                for nc in max(1, dc) - 1..min(col, dc + 2) {
                    if is_open[nr][nc] {
                        continue;
                    }
                    // 不在 cr,cc 的 3x3 内
                    if nr < max(1, cr) - 1 || nr >= min(row, cr + 2)
                        || nc < max(1, cc) - 1 || nc >= min(col, cc + 2)
                    {
                        if board[nr][nc] == -1 {
                            extra_cell -= 1;
                            extra_3bv -= 1;
                        } else {
                            extra_cell += 1;
                            if groups[nr][nc] > opening_count {
                                extra_3bv += 1;
                            }
                        }
                    }
                }
            }

            best = match best {
                None => Some((dr, dc, extra_3bv, extra_cell)),
                Some((_, _, eb, ec))
                    if extra_3bv > eb || (extra_3bv == eb && extra_cell > ec) =>
                {
                    Some((dr, dc, extra_3bv, extra_cell))
                }
                Some(v) => Some(v),
            };
        }
    }

    best.map(|(r, c, _, _)| (r, c))
}

/// 打开整个空域（与 (r,c) 同组的所有 0 格 + 周围一圈）。
fn open_opening(
    is_open: &mut Vec<Vec<bool>>,
    board: &Vec<Vec<i32>>,
    groups: &Vec<Vec<usize>>,
    r: usize,
    c: usize,
) {
    let row = board.len();
    let col = board[0].len();
    let gid = groups[r][c];
    for gr in 0..row {
        for gc in 0..col {
            if board[gr][gc] == 0 && groups[gr][gc] == gid {
                is_open[gr][gc] = true;
                for dr in max(1, gr) - 1..min(row, gr + 2) {
                    for dc in max(1, gc) - 1..min(col, gc + 2) {
                        is_open[dr][dc] = true;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 优先值（premium）计算
// ---------------------------------------------------------------------------

fn calc_premium(
    board: &Vec<Vec<i32>>,
    groups: &Vec<Vec<usize>>,
    opening_count: usize,
    is_open: &Vec<Vec<bool>>,
    r: usize,
    c: usize,
    two_step: bool,
    wide_candidate: bool,
) -> (Option<isize>, isize, bool) {
    let row = board.len();
    let col = board[0].len();
    let mut premium: isize = 0;
    let mut unflagged: isize = 0;
    let mut temp_group: Vec<usize> = Vec::new();
    let mut have_unopen = false;

    for dr in max(1, r) - 1..min(row, r + 2) {
        for dc in max(1, c) - 1..min(col, c + 2) {
            if is_open[dr][dc] {
                continue;
            }
            have_unopen = true;
            if board[dr][dc] != -1 {
                if groups[dr][dc] <= opening_count {
                    if board[dr][dc] == 0 && !temp_group.contains(&groups[dr][dc]) {
                        temp_group.push(groups[dr][dc]);
                        premium += 1;
                    }
                } else {
                    premium += 1;
                }
            } else {
                premium -= 1;
                unflagged += 1;
            }
        }
    }

    if !is_open[r][c] {
        premium -= 1;
    }
    premium -= 1;

    // 移除无用的双击候选
    if premium <= 0 && is_open[r][c] && unflagged == 0 {
        return (None, premium, have_unopen);
    }

    let mut max_adjacent_premium: isize = 0;
    let mut max_adjacent_premium_related: isize = 0;

    if (premium > 0 || wide_candidate) && two_step {
        let (ar_min_r, ar_max_r, ar_min_c, ar_max_c) = if wide_candidate {
            (
                max(1, r) - 2,
                min(row, r + 3),
                max(1, c) - 2,
                min(col, c + 3),
            )
        } else {
            (
                max(1, r) - 1,
                min(row, r + 2),
                max(1, c) - 1,
                min(col, c + 2),
            )
        };

        for ar in ar_min_r..ar_max_r {
            for ac in ar_min_c..ar_max_c {
                if board[ar][ac] == -1 || board[ar][ac] == 0 {
                    continue;
                }

                let is_adjacent_closed = !is_open[ar][ac];
                let is_far = ar < max(1, r) - 1
                    || ar >= min(row, r + 2)
                    || ac < max(1, c) - 1
                    || ac >= min(col, c + 2);

                let mut adjacent_premium = if is_adjacent_closed {
                    if is_far {
                        -2
                    } else {
                        -1
                    }
                } else {
                    -1
                };

                let adjacent_premium_related = if is_adjacent_closed && !is_far {
                    1
                } else {
                    0
                };

                let mut temp_group2 = temp_group.clone();

                for br in max(1, ar) - 1..min(row, ar + 2) {
                    for bc in max(1, ac) - 1..min(col, ac + 2) {
                        if is_open[br][bc] {
                            continue;
                        }

                        let in_overlap = !(br < max(1, r) - 1
                            || br >= min(row, r + 2)
                            || bc < max(1, c) - 1
                            || bc >= min(col, c + 2));

                        if in_overlap {
                            if board[br][bc] == -1 {
                                adjacent_premium += 1;
                            }
                            if board[br][bc] != -1 && groups[br][bc] > opening_count {
                                adjacent_premium -= 1;
                            }
                        } else {
                            if board[br][bc] != -1 {
                                if groups[br][bc] <= opening_count {
                                    if board[br][bc] == 0
                                        && !temp_group2.contains(&groups[br][bc])
                                    {
                                        temp_group2.push(groups[br][bc]);
                                        adjacent_premium += 1;
                                    }
                                } else {
                                    adjacent_premium += 1;
                                }
                            } else {
                                adjacent_premium -= 1;
                            }
                        }
                    }
                }

                if adjacent_premium > max_adjacent_premium {
                    max_adjacent_premium = adjacent_premium;
                    max_adjacent_premium_related = adjacent_premium_related;
                }
            }
        }
    }

    if max_adjacent_premium > 0
        && (max_adjacent_premium - max_adjacent_premium_related) > premium
    {
        return (None, premium, have_unopen);
    }
    premium += max_adjacent_premium;

    (Some(premium), premium, have_unopen)
}

// ---------------------------------------------------------------------------
// 辅助
// ---------------------------------------------------------------------------

fn all_non_mine_open(is_open: &Vec<Vec<bool>>, board: &Vec<Vec<i32>>) -> bool {
    for r in 0..board.len() {
        for c in 0..board[0].len() {
            if board[r][c] != -1 && !is_open[r][c] {
                return false;
            }
        }
    }
    true
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cal_bbbv;

    #[test]
    fn test_cal_zini_empty() {
        let board = vec![
            vec![0, 0, 0],
            vec![0, 0, 0],
            vec![0, 0, 0],
        ];
        let z = cal_zini(&board, 10);
        // 3x3 全是 0，只需要 1 左键
        assert_eq!(z, 1);
    }

    #[test]
    fn test_cal_zini_one_mine() {
        let board = vec![
            vec![0, 0, 0],
            vec![0, 1, 1],
            vec![0, 1, -1],
        ];
        let z = cal_zini(&board, 50);
        assert!(z > 0);
        assert!(z <= 5);
    }

    #[test]
    fn test_cal_zini_vs_bbbv() {
        // ZiNi 值通常接近 3BV（但会略小，因为允许双击）
        let board = crate::laymine_op(16, 30, 99, 0, 0);
        let bbbv = cal_bbbv(&board);
        let z = cal_zini(&board, 20);
        println!("高级 3BV={} ZiNi={}", bbbv, z);
        assert!(z <= bbbv || bbbv == 0);
    }

    #[test]
    fn test_cal_zini_split_opening() {
        // 两个零不连通，中间有数字隔开
        let board = vec![
            vec![0, 0, 1, -1, 1, 0, 0],
            vec![0, 0, 1, 1, 1, 0, 0],
            vec![1, 1, 1, 1, 1, 1, 1],
            vec![-1, 2, -1, 1, 2, -1, 1],
            vec![1, 2, 1, 1, 2, 1, 1],
            vec![0, 1, 1, 1, 1, 1, 1],
            vec![0, 0, 1, 1, 1, 0, 0],
        ];
        let z = cal_zini(&board, 50);
        println!("分割空局面 ZiNi = {}", z);
        assert!(z >= 2);
    }

    #[test]
    fn test_zini_with_islands() {
        // 两处分离的数字孤岛，需要分别处理
        let board = vec![
            vec![-1, -1, -1, -1, -1, -1, -1],
            vec![-1, 1,  1,  1, -1, 1,  -1],
            vec![-1, 1,  2,  1, -1, 1,  -1],
            vec![-1, 1,  1,  1, -1, 1,  -1],
            vec![-1, -1, -1, -1, -1, -1, -1],
        ];
        let bbbv = crate::cal_bbbv(&board);
        let z = cal_zini(&board, 50);
        println!("双孤岛 3BV={} ZiNi={}", bbbv, z);
        // 两处孤岛：每处先左键点一个数字再双击，共 2*(1+1)=4，
        // 但还可能点中间，所以保守断言
        assert!(z >= 2);
    }
}
