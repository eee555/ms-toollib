use crate::safe_board;
use getrandom::getrandom;

// 本模块实现了三种 ZiNi 计算方法，均为原版算法。可用于大型局面，例如255*255以下。
// 帖子在[此处](https://minesweepergame.com/forum/viewtopic.php?f=15&t=70)
// 
// 源码下载链接在[此处](https://minesweepergame.com/forum/download/file.php?id=120&sid=1248a41671e7701e8c9b340ffb13e55d)
//
// ZiNi 是一种贪心蒙特卡洛算法，估计扫完一个局面所需的最少点击次数（左键 + 右键 + 双击）。由 Elmar Zimmermann 和 Christoph Nikolaus 提出，取二人姓名首字母得名。PTTACGfans 做的是增量改进（两步式、混合式、路径优化）。
// PTTACGfans算法开源在[此处](https://github.com/PTTACGfans/Minesweeper-ZiNi-Calculator)
// 文档在[此处](https://docs.google.com/document/d/1Ve1gfaxZcgabvkAusIDzPVK0RMSpCdmgD_8TruhE28g/edit?tab=t.0#heading=h.tujz7cf074cp)
// 

struct Zcell {
    mine: bool,
    opening: usize,
    opening2: usize,
    number: i32,
    opened: bool,
    flagged: bool,
    premium: i32,
    rb: usize, re: usize,
    cb: usize, ce: usize,
}

/// Greedy ZiNi（确定性算法，贪心算法，默认的算法）。  
/// 每轮选择 premium 最高的未打开格，对其标雷并 chord，重复至游戏结束。  
/// `loop_count` 仅用于保持签名兼容，内部未使用。
pub fn cal_zini<T>(board: &T) -> usize
where
    T: std::ops::Index<usize> + safe_board::BoardSize,
    T::Output: std::ops::Index<usize, Output = i32>,
{
    let height = board.get_row();
    let width = board.get_column();
    let mut vboard = vec![vec![0i32; width]; height];
    for r in 0..height {
        for c in 0..width {
            vboard[r][c] = board[r][c];
        }
    }
    let mut cells = build_cells_ref(&vboard, height, width);
    initboard(&mut cells, width, height);
    zinialg(&mut cells, width, height, false, false)
}

/// Human ZiNi（模拟人工操作）。  
/// 先点开所有 opening，再在已打开格上按 premium 做标雷/chord。
pub fn cal_hzini<T>(board: &T) -> usize
where
    T: std::ops::Index<usize> + safe_board::BoardSize,
    T::Output: std::ops::Index<usize, Output = i32>,
{
    let height = board.get_row();
    let width = board.get_column();
    let mut vboard = vec![vec![0i32; width]; height];
    for r in 0..height {
        for c in 0..width {
            vboard[r][c] = board[r][c];
        }
    }
    let mut cells = build_cells_ref(&vboard, height, width);
    initboard(&mut cells, width, height);
    zinialg(&mut cells, width, height, true, true)
}

fn rand_range(limit: usize) -> usize {
    if limit <= 1 {
        return 0;
    }
    let mut buf = [0u8; 4];
    loop {
        getrandom(&mut buf).unwrap();
        let t = u32::from_le_bytes(buf) as usize;
        let max = usize::MAX - (usize::MAX % limit);
        if t < max {
            return t % limit;
        }
    }
}

/// Random ZiNi（随机化算法）。  
/// 多轮迭代，每轮在 premium 最高值对应的格子中随机选择，返回所有轮次的最小值。
pub fn cal_rzini<T>(board: &T, n_iter: usize) -> usize
where
    T: std::ops::Index<usize> + safe_board::BoardSize,
    T::Output: std::ops::Index<usize, Output = i32>,
{
    let height = board.get_row();
    let width = board.get_column();
    let mut vboard = vec![vec![0i32; width]; height];
    for r in 0..height {
        for c in 0..width {
            vboard[r][c] = board[r][c];
        }
    }
    let mut min_zini = usize::MAX;
    for _i in 0..n_iter {
        let mut cells = build_cells_ref(&vboard, height, width);
        initboard(&mut cells, width, height);
        let z = zinialg_rng(&mut cells, width, height, false, false);
        if z < min_zini {
            min_zini = z;
        }
    }
    min_zini
}

fn build_cells_ref(board: &[Vec<i32>], height: usize, width: usize) -> Vec<Zcell> {
    let size = width * height;
    let mut cells: Vec<Zcell> = Vec::with_capacity(size);
    for c in 0..width {
        for r in 0..height {
            let mine = board[r][c] == -1;
            let rb = if r > 0 { r - 1 } else { r };
            let re = if r < height - 1 { r + 1 } else { r };
            let cb = if c > 0 { c - 1 } else { c };
            let ce = if c < width - 1 { c + 1 } else { c };
            cells.push(Zcell {
                mine,
                opening: 0,
                opening2: 0,
                number: 0,
                opened: false,
                flagged: false,
                premium: 0,
                rb, re, cb, ce,
            });
        }
    }
    cells
}

fn getnumber(cells: &[Zcell], height: usize, index: usize) -> i32 {
    let mut res = 0i32;
    let rb = cells[index].rb;
    let re = cells[index].re;
    let cb = cells[index].cb;
    let ce = cells[index].ce;
    for rr in rb..=re {
        for cc in cb..=ce {
            let i = cc * height + rr;
            if cells[i].mine {
                res += 1;
            }
        }
    }
    res
}

fn getadj3bv(cells: &[Zcell], height: usize, index: usize) -> i32 {
    if cells[index].number == 0 {
        return 1;
    }
    let mut res = 0i32;
    let rb = cells[index].rb;
    let re = cells[index].re;
    let cb = cells[index].cb;
    let ce = cells[index].ce;
    for rr in rb..=re {
        for cc in cb..=ce {
            let i = cc * height + rr;
            if !cells[i].mine && cells[i].opening == 0 {
                res += 1;
            }
        }
    }
    if cells[index].opening != 0 {
        res += 1;
    }
    if cells[index].opening2 != 0 {
        res += 1;
    }
    res
}

fn setopeningborder(cells: &mut [Zcell], op_id: usize, index: usize) {
    if cells[index].opening == 0 {
        cells[index].opening = op_id;
    } else if cells[index].opening != op_id && cells[index].opening2 == 0 {
        cells[index].opening2 = op_id;
    }
}

fn process_opening(cells: &mut [Zcell], height: usize, op_id: usize, index: usize) {
    cells[index].opening = op_id;
    let rb = cells[index].rb;
    let re = cells[index].re;
    let cb = cells[index].cb;
    let ce = cells[index].ce;
    for rr in rb..=re {
        for cc in cb..=ce {
            let i = cc * height + rr;
            if cells[i].number != 0 {
                setopeningborder(cells, op_id, i);
            } else if cells[i].opening == 0 {
                process_opening(cells, height, op_id, i);
            }
        }
    }
}

fn initboard(cells: &mut [Zcell], width: usize, height: usize) {
    let size = width * height;
    for i in 0..size {
        cells[i].number = getnumber(cells, height, i);
        cells[i].premium = -cells[i].number - 2;
    }
    let mut openings = 0usize;
    for i in 0..size {
        if !cells[i].mine && cells[i].number == 0 && cells[i].opening == 0 {
            openings += 1;
            process_opening(cells, height, openings, i);
        }
    }
    for i in 0..size {
        cells[i].premium += getadj3bv(cells, height, i);
    }
}

fn open(cells: &mut [Zcell], height: usize, closed_cells: &mut usize, index: usize) {
    cells[index].opened = true;
    cells[index].premium += 1;
    if cells[index].opening == 0 {
        let rb = cells[index].rb;
        let re = cells[index].re;
        let cb = cells[index].cb;
        let ce = cells[index].ce;
        for rr in rb..=re {
            for cc in cb..=ce {
                let i = cc * height + rr;
                cells[i].premium -= 1;
            }
        }
    }
    *closed_cells -= 1;
}

fn reveal(cells: &mut [Zcell], height: usize, closed_cells: &mut usize, index: usize) {
    if cells[index].opened || cells[index].flagged {
        return;
    }
    if cells[index].number != 0 {
        open(cells, height, closed_cells, index);
    } else {
        let op = cells[index].opening;
        let size = cells.len();
        for i in 0..size {
            if cells[i].opening2 == op || cells[i].opening == op {
                if !cells[i].opened {
                    open(cells, height, closed_cells, i);
                }
                cells[i].premium -= 1;
            }
        }
    }
}

fn click(cells: &mut [Zcell], height: usize, zini: &mut usize, closed_cells: &mut usize, index: usize) {
    reveal(cells, height, closed_cells, index);
    *zini += 1;
}

fn flag(cells: &mut [Zcell], height: usize, zini: &mut usize, index: usize) {
    if cells[index].flagged {
        return;
    }
    *zini += 1;
    cells[index].flagged = true;
    let rb = cells[index].rb;
    let re = cells[index].re;
    let cb = cells[index].cb;
    let ce = cells[index].ce;
    for rr in rb..=re {
        for cc in cb..=ce {
            let i = cc * height + rr;
            cells[i].premium += 1;
        }
    }
}

fn flagaround(cells: &mut [Zcell], height: usize, zini: &mut usize, index: usize) {
    let rb = cells[index].rb;
    let re = cells[index].re;
    let cb = cells[index].cb;
    let ce = cells[index].ce;
    for rr in rb..=re {
        for cc in cb..=ce {
            let i = cc * height + rr;
            if cells[i].mine {
                flag(cells, height, zini, i);
            }
        }
    }
}

fn chord(cells: &mut [Zcell], height: usize, zini: &mut usize, closed_cells: &mut usize, index: usize) {
    *zini += 1;
    let rb = cells[index].rb;
    let re = cells[index].re;
    let cb = cells[index].cb;
    let ce = cells[index].ce;
    for rr in rb..=re {
        for cc in cb..=ce {
            let i = cc * height + rr;
            reveal(cells, height, closed_cells, i);
        }
    }
}

fn hitopenings(cells: &mut [Zcell], height: usize, zini: &mut usize, closed_cells: &mut usize) {
    let size = cells.len();
    for j in 0..size {
        if !cells[j].mine && cells[j].number == 0 && !cells[j].opened {
            click(cells, height, zini, closed_cells, j);
        }
    }
}

fn apply_zini(
    cells: &mut [Zcell],
    height: usize,
    zini: &mut usize,
    closed_cells: &mut usize,
    human: bool,
) -> Option<usize> {
    let size = cells.len();
    let mut maxp = -1i32;
    let mut curi = None;

    for i in 0..size {
        if cells[i].premium > maxp && !cells[i].mine && (cells[i].opened || !human) {
            maxp = cells[i].premium;
            curi = Some(i);
        }
    }

    if let Some(idx) = curi {
        if !cells[idx].opened {
            click(cells, height, zini, closed_cells, idx);
        }
        flagaround(cells, height, zini, idx);
        chord(cells, height, zini, closed_cells, idx);
        Some(idx)
    } else {
        let mut fb = None;
        for i in 0..size {
            if !cells[i].opened && !cells[i].mine &&
                (cells[i].number == 0 || cells[i].opening == 0)
            {
                fb = Some(i);
                break;
            }
        }
        if let Some(idx) = fb {
            click(cells, height, zini, closed_cells, idx);
            fb
        } else {
            None
        }
    }
}

fn apply_zini_rng(
    cells: &mut [Zcell],
    height: usize,
    zini: &mut usize,
    closed_cells: &mut usize,
    human: bool,
    maxpr: &mut Vec<usize>,
) -> Option<usize> {
    let size = cells.len();
    let mut maxp = -1i32;
    let mut curi_len = 0usize;

    for i in 0..size {
        if cells[i].mine { continue; }
        if human && !cells[i].opened { continue; }
        if cells[i].premium > maxp {
            maxp = cells[i].premium;
            curi_len = 1;
            maxpr[0] = i;
        } else if maxp >= 0 && cells[i].premium == maxp {
            maxpr[curi_len] = i;
            curi_len += 1;
        }
    }

    if maxp >= 0 {
        let idx = maxpr[rand_range(curi_len)];
        if !cells[idx].opened {
            click(cells, height, zini, closed_cells, idx);
        }
        flagaround(cells, height, zini, idx);
        chord(cells, height, zini, closed_cells, idx);
        Some(idx)
    } else {
        let mut fb_count = 0usize;
        for i in 0..size {
            if !cells[i].opened && !cells[i].mine &&
                (cells[i].number == 0 || cells[i].opening == 0)
            {
                maxpr[fb_count] = i;
                fb_count += 1;
            }
        }
        if fb_count > 0 {
            let pick = maxpr[rand_range(fb_count)];
            click(cells, height, zini, closed_cells, pick);
            Some(pick)
        } else {
            None
        }
    }
}

fn zinialg(
    cells: &mut [Zcell],
    width: usize,
    height: usize,
    human: bool,
    hitops: bool,
) -> usize {
    let size = width * height;
    let mut mines = 0usize;
    for i in 0..size {
        if cells[i].mine {
            mines += 1;
        }
    }
    let mut closed_cells = size;
    let mut zini = 0;

    if hitops {
        hitopenings(cells, height, &mut zini, &mut closed_cells);
    }

    while closed_cells > mines {
        if apply_zini(cells, height, &mut zini, &mut closed_cells, human).is_none() {
            break;
        }
    }
    zini
}

fn zinialg_rng(
    cells: &mut [Zcell],
    width: usize,
    height: usize,
    human: bool,
    hitops: bool,
) -> usize {
    let size = width * height;
    let mut mines = 0usize;
    for i in 0..size {
        if cells[i].mine {
            mines += 1;
        }
    }
    let mut maxpr = vec![0usize; size];
    let mut closed_cells = size;
    let mut zini = 0;

    if hitops {
        hitopenings(cells, height, &mut zini, &mut closed_cells);
    }

    while closed_cells > mines {
        if apply_zini_rng(cells, height, &mut zini, &mut closed_cells, human, &mut maxpr).is_none() {
            break;
        }
    }
    zini
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{laymine, videos::NewSomeVideo};

    #[test]
    fn test_greedy_vs_reference() {
        let mut video = crate::AvfVideo::new("../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf");
        let r = video.parse();
        assert_eq!(r.unwrap(), ());
        video.data.analyse();
        let board = video.data.board.clone();
        let z = cal_zini(&board);
        assert_eq!(z, 96);
    }

    #[test]
    fn test_human_vs_reference() {
        let mut video = crate::AvfVideo::new("../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf");
        let r = video.parse();
        assert_eq!(r.unwrap(), ());
        video.data.analyse();
        let board = video.data.board.clone();
        let z = cal_hzini(&board);
        assert_eq!(z, 103);
    }

    #[test]
    fn test_random_min_le_greedy() {
        let mut video = crate::AvfVideo::new("../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf");
        let r = video.parse();
        assert_eq!(r.unwrap(), ());
        video.data.analyse();
        let board = video.data.board.clone();
        let z = cal_rzini(&board, 50);
        let greedy = cal_zini(&board);
        println!("Random min={} Greedy={}", z, greedy);
        assert!(z <= greedy, "Random min={} should be <= Greedy={}", z, greedy);
        assert!(z > 0);
    }

    #[test]
    fn test_empty_board() {
        let board = vec![
            vec![0, 0, 0],
            vec![0, 0, 0],
            vec![0, 0, 0],
        ];
        let z = cal_zini(&board);
        assert_eq!(z, 1);
    }

    #[test]
    fn test_big_board() {
        let board = laymine(300, 100, 6000, 0, 0);
        let z = cal_zini(&board);
        println!("Big board ZiNi: {}", z);
    }
}
