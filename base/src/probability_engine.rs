use crate::big_number::BigNumber;
use std::cmp::{max, min};

const SMALL_COMB: [[usize; 9]; 9] = [
    [1, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 1, 0, 0, 0, 0, 0, 0],
    [1, 3, 3, 1, 0, 0, 0, 0, 0],
    [1, 4, 6, 4, 1, 0, 0, 0, 0],
    [1, 5, 10, 10, 5, 1, 0, 0, 0],
    [1, 6, 15, 20, 15, 6, 1, 0, 0],
    [1, 7, 21, 35, 35, 21, 7, 1, 0],
    [1, 8, 28, 56, 70, 56, 28, 8, 1],
];

fn comb_small(n: usize, k: usize) -> usize {
    if k > n || n > 8 {
        return 0;
    }
    SMALL_COMB[n][k]
}

fn comb_big(n: usize, k: usize) -> BigNumber {
    if k > n {
        return BigNumber { a: 0.0, b: 0 };
    }
    if k > n - k {
        return comb_big(n, n - k);
    }
    let mut r = BigNumber { a: 1.0, b: 0 };
    for i in 0..k {
        r = &r * ((n - i) as f64 / (i + 1) as f64);
    }
    r
}

fn comb_val(n: usize, k: usize) -> BigNumber {
    if n <= 8 && k <= 8 {
        let v = comb_small(n, k);
        BigNumber { a: v as f64, b: 0 }
    } else {
        comb_big(n, k)
    }
}

#[derive(Clone)]
struct BoxWitness {
    mines_to_find: usize,
    tiles: Vec<(usize, usize)>,
    boxes: Vec<usize>,
    processed: bool,
}

#[derive(Clone)]
struct Box {
    tiles: Vec<(usize, usize)>,
    witnesses: Vec<usize>,
    min_mines: usize,
    max_mines: usize,
    empty_tiles: usize,
    processed: bool,
}

#[derive(Clone)]
struct ProbabilityLine {
    mine_count: usize,
    solution_count: BigNumber,
    mine_box_count: Vec<BigNumber>,
    allocated_mines: Vec<usize>,
}

struct NextWitness {
    witness_idx: usize,
    old_boxes: Vec<usize>,
    new_boxes: Vec<usize>,
}

pub fn cal_probability_csp(
    board_of_game: &Vec<Vec<i32>>,
    minenum: f64,
) -> Result<(Vec<((usize, usize), f64)>, f64, [usize; 3], usize), usize> {
    let rows = board_of_game.len();
    let cols = board_of_game[0].len();
    let total_cells = rows * cols;

    let total_mines = if minenum < 1.0 {
        (total_cells as f64 * minenum) as usize
    } else {
        minenum as usize
    };

    let mut total_unopened = 0usize;
    for r in 0..rows {
        for c in 0..cols {
            if board_of_game[r][c] >= 10 {
                total_unopened += 1;
            }
        }
    }

    let total_mines = min(total_mines, total_unopened);

    // Build witnesses from number tiles
    let mut witnesses: Vec<BoxWitness> = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let val = board_of_game[r][c];
            if val < 0 || val > 8 {
                continue;
            }
            let mut adj_unknown = Vec::new();
            for dr in -1i32..=1 {
                for dc in -1i32..=1 {
                    if dr == 0 && dc == 0 { continue; }
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    if nr < 0 || nr >= rows as i32 || nc < 0 || nc >= cols as i32 { continue; }
                    let (nr, nc) = (nr as usize, nc as usize);
                    if board_of_game[nr][nc] >= 10 {
                        adj_unknown.push((nr, nc));
                    }
                }
            }
            if adj_unknown.is_empty() {
                continue;
            }
            let mines_to_find = val;
            if mines_to_find < 0 || mines_to_find > adj_unknown.len() as i32 {
                return Err(1);
            }
            witnesses.push(BoxWitness {
                mines_to_find: mines_to_find as usize,
                tiles: adj_unknown,
                boxes: Vec::new(),
                processed: false,
            });
        }
    }

    if witnesses.is_empty() {
        return fallback_pure_binomial(board_of_game, total_mines, total_unopened);
    }

    // Collect all witnessed tiles
    let mut witnessed_set: Vec<(usize, usize)> = Vec::new();
    for w in &witnesses {
        for &t in &w.tiles {
            if !witnessed_set.contains(&t) {
                witnessed_set.push(t);
            }
        }
    }

    let off_edge = total_unopened - witnessed_set.len();

    // Build boxes: group tiles by same witness adjacency pattern
    let mut boxes: Vec<Box> = Vec::new();
    for &tile in &witnessed_set {
        let adj_wits: Vec<usize> = witnesses.iter().enumerate()
            .filter(|(_, w)| w.tiles.contains(&tile))
            .map(|(i, _)| i)
            .collect();
        let count = adj_wits.len();

        let mut found = false;
        for b in boxes.iter_mut() {
            if b.witnesses.len() != count { continue; }
            if b.witnesses.iter().all(|wi| adj_wits.contains(wi)) {
                b.tiles.push(tile);
                found = true;
                break;
            }
        }
        if !found {
            let uid = boxes.len();
            for &wi in &adj_wits {
                witnesses[wi].boxes.push(uid);
            }
            boxes.push(Box {
                tiles: vec![tile],
                witnesses: adj_wits,
                min_mines: 0,
                max_mines: 0,
                empty_tiles: 0,
                processed: false,
            });
        }
    }

    // Calculate min/max mines for each box
    for b in boxes.iter_mut() {
        b.max_mines = b.tiles.len();
        b.min_mines = 0;
        for &wi in &b.witnesses {
            if witnesses[wi].mines_to_find < b.max_mines {
                b.max_mines = witnesses[wi].mines_to_find;
            }
            if witnesses[wi].boxes.len() == 1 {
                b.min_mines = witnesses[wi].mines_to_find;
            }
        }
    }

    let box_count = boxes.len();
    let mut held: Vec<ProbabilityLine> = Vec::new();
    let mut working = vec![ProbabilityLine {
        mine_count: 0,
        solution_count: BigNumber { a: 1.0, b: 0 },
        mine_box_count: vec![BigNumber { a: 0.0, b: 0 }; box_count],
        allocated_mines: vec![0; box_count],
    }];
    let mut mask = vec![false; box_count];
    let mut is_first_set = true;

    loop {
        // Find next witness to process
        let nw = if is_first_set {
            is_first_set = false;
            // First set: grab the first unprocessed witness directly
            match get_first_unprocessed(&witnesses) {
                Some(idx) => NextWitness {
                    witness_idx: idx,
                    old_boxes: Vec::new(),
                    new_boxes: witnesses[idx].boxes.clone(),
                },
                None => break,
            }
        } else {
            match get_boundary_witness(&witnesses, &mask) {
                Some(nw) => nw,
                None => {
                    // End of current independent set
                    store_independent_set(&mut held, &mut working, &boxes, &mask, total_unopened);
                    if witnesses.iter().all(|w| w.processed) {
                        break;
                    }
                    // Start new independent set
                    working = vec![ProbabilityLine {
                        mine_count: 0,
                        solution_count: BigNumber { a: 1.0, b: 0 },
                        mine_box_count: vec![BigNumber { a: 0.0, b: 0 }; box_count],
                        allocated_mines: vec![0; box_count],
                    }];
                    mask = vec![false; box_count];
                    match get_first_unprocessed(&witnesses) {
                        Some(idx) => NextWitness {
                            witness_idx: idx,
                            old_boxes: Vec::new(),
                            new_boxes: witnesses[idx].boxes.clone(),
                        },
                        None => break,
                    }
                }
            }
        };

        // Mark new boxes as processed
        for &b in &nw.new_boxes {
            mask[b] = true;
        }

        // Merge this witness into working probability lines
        let wi = nw.witness_idx;
        let mut new_working = Vec::new();
        for pl in &working {
            let placed: usize = nw.old_boxes.iter().map(|&b| pl.allocated_mines[b]).sum();
            if placed > witnesses[wi].mines_to_find {
                continue;
            }
            let missing = witnesses[wi].mines_to_find - placed;
            if missing == 0 {
                new_working.push(pl.clone());
            } else if nw.new_boxes.is_empty() {
                // Missing mines go to off-edge (just bump mine_count)
                let mut npl = pl.clone();
                npl.mine_count += missing;
                new_working.push(npl);
            } else {
                distribute_mines(pl, &nw, &boxes, missing, 0, &mut new_working);
            }
        }

        witnesses[wi].processed = true;
        for &b in &nw.new_boxes {
            boxes[b].processed = true;
        }

        working = if new_working.len() > 200 {
            let bound = find_boundary(&boxes, &mask);
            crunch(&new_working, &bound, &mask)
        } else {
            new_working
        };
    }

    // Final store if any working probs remain
    store_independent_set(&mut held, &mut working, &boxes, &mask, total_unopened);

    // After store_independent_set, working is consumed (taken) if small,
    // or left intact if large. Handle remaining.
    if !working.is_empty() {
        let all_boxes: Vec<bool> = vec![true; boxes.len()];
        filter_lines(&mut working, &boxes, &all_boxes, total_unopened);
        if !working.is_empty() {
            let crunched = if working.len() > 50 {
                let bound = find_boundary(&boxes, &mask);
                crunch(&working, &bound, &mask)
            } else {
                working
            };
            if held.is_empty() {
                held = crunched;
            } else if !crunched.is_empty() {
                held = convolve(&held, &crunched, total_unopened);
            }
        }
    }

    if held.is_empty() {
        return Err(1);
    }

    // Compute min/max possible mines in witnessed area
    let mut mine_min = usize::MAX;
    let mut mine_max: usize = 0;

    for pl in &held {
        mine_min = min(mine_min, pl.mine_count);
        mine_max = max(mine_max, pl.mine_count);
    }


    // Clamp total_mines to feasible range for tally computation
    let min_possible = mine_min;
    let max_possible = min(total_unopened, mine_max + off_edge);
    let cur_mines = min(max(total_mines, min_possible), max_possible);

    // Expand with off-edge and compute probabilities
    let min_witnessed = if cur_mines > off_edge { cur_mines - off_edge } else { 0 };

    let mut tally = vec![BigNumber { a: 0.0, b: 0 }; box_count];
    let mut total_tally = BigNumber { a: 0.0, b: 0 };
    let mut outside_tally = BigNumber { a: 0.0, b: 0 };
    let mut has_valid = false;

    for pl in &held {
        if pl.mine_count < min_witnessed { continue; }
        if pl.mine_count > cur_mines { continue; }
        let rem = cur_mines - pl.mine_count;
        if rem > off_edge { continue; }
        has_valid = true;
        let mult = comb_val(off_edge, rem);
        let new_sol = &mult * &pl.solution_count;
        total_tally += &new_sol;
        if off_edge > 0 {
            let oc = comb_val(off_edge, rem);
            let tmp1 = &oc * &BigNumber { a: rem as f64, b: 0 };
            outside_tally += &(&tmp1 * &pl.solution_count);
        }
        for j in 0..box_count {
            let nt = boxes[j].tiles.len();
            if nt > 0 {
                let tmp2 = &mult * &pl.mine_box_count[j];
                tally[j] += &(&tmp2 / &BigNumber { a: nt as f64, b: 0 });
            }
        }
    }

    if !has_valid || total_tally.a == 0.0 {
        return Err(1);
    }

    let mut result = Vec::new();
    for j in 0..box_count {
        let ratio: f64 = (&tally[j] / &total_tally).into();
        for &t in &boxes[j].tiles {
            result.push((t, ratio));
        }
    }

    let p_off = if off_edge > 0 {
        let tmp3 = &total_tally * &BigNumber { a: off_edge as f64, b: 0 };
        let ratio: f64 = (&outside_tally / &tmp3).into();
        ratio
    } else {
        f64::NAN
    };

    Ok((
        result,
        p_off,
        [mine_min, cur_mines, min(total_unopened, mine_max + off_edge)],
        0,
    ))
}

fn fallback_pure_binomial(
    board_of_game: &Vec<Vec<i32>>,
    total_mines: usize,
    total_unopened: usize,
) -> Result<(Vec<((usize, usize), f64)>, f64, [usize; 3], usize), usize> {
    let mut result = Vec::new();
    for r in 0..board_of_game.len() {
        for c in 0..board_of_game[0].len() {
            if board_of_game[r][c] >= 10 {
                let prob = if total_unopened > 0 { total_mines as f64 / total_unopened as f64 } else { 0.0 };
                result.push(((r, c), prob));
            }
        }
    }
    let p = if total_unopened > 0 { total_mines as f64 / total_unopened as f64 } else { f64::NAN };
    Ok((result, p, [0, total_mines, total_unopened], 0))
}

fn get_boundary_witness(
    witnesses: &[BoxWitness],
    mask: &[bool],
) -> Option<NextWitness> {
    for (i, w) in witnesses.iter().enumerate() {
        if w.processed { continue; }
        // Check if any of this witness's boxes is already in the mask
        if w.boxes.iter().any(|&b| b < mask.len() && mask[b]) {
            let old: Vec<usize> = w.boxes.iter().filter(|&&b| b < mask.len() && mask[b]).copied().collect();
            let new: Vec<usize> = w.boxes.iter().filter(|&&b| !(b < mask.len() && mask[b])).copied().collect();
            return Some(NextWitness { witness_idx: i, old_boxes: old, new_boxes: new });
        }
    }
    None
}

fn get_first_unprocessed(witnesses: &[BoxWitness]) -> Option<usize> {
    witnesses.iter().position(|w| !w.processed)
}

fn filter_lines(lines: &mut Vec<ProbabilityLine>, boxes: &[Box], mask: &[bool], _total_mines: usize) {
    lines.retain(|line| {
        for (j, b) in boxes.iter().enumerate() {
            if j < mask.len() && !mask[j] { continue; }
            if line.allocated_mines[j] < b.min_mines || line.allocated_mines[j] > b.max_mines {
                return false;
            }
        }
        true
    });
}

fn store_independent_set(
    held: &mut Vec<ProbabilityLine>,
    working: &mut Vec<ProbabilityLine>,
    boxes: &[Box],
    mask: &[bool],
    max_possible: usize,
) {
    if working.is_empty() { return; }
    filter_lines(working, boxes, mask, max_possible);
    if working.is_empty() { return; }
    let crunched = if working.len() > 50 {
        let bound = find_boundary(boxes, mask);
        crunch(working, &bound, mask)
    } else {
        std::mem::take(working)
    };
    if crunched.is_empty() { return; }
    if held.is_empty() {
        *held = crunched;
    } else {
        *held = convolve(held, &crunched, max_possible);
    }
}

fn distribute_mines(
    pl: &ProbabilityLine,
    nw: &NextWitness,
    boxes: &[Box],
    missing: usize,
    idx: usize,
    result: &mut Vec<ProbabilityLine>,
) {
    if idx >= nw.new_boxes.len() { return; }
    let bidx = nw.new_boxes[idx];
    let b = &boxes[bidx];

    if nw.new_boxes.len() - idx == 1 {
        if missing < b.min_mines || missing > b.max_mines { return; }
        result.push(extend_line(pl, bidx, boxes, missing));
        return;
    }

    let max_p = min(b.max_mines, missing);
    if b.min_mines > max_p { return; }
    for m in b.min_mines..=max_p {
        let npl = extend_line(pl, bidx, boxes, m);
        distribute_mines(&npl, nw, boxes, missing - m, idx + 1, result);
    }
}

fn extend_line(
    pl: &ProbabilityLine,
    bidx: usize,
    boxes: &[Box],
    mines: usize,
) -> ProbabilityLine {
    let b = &boxes[bidx];
    let n = b.tiles.len() - b.empty_tiles;
    let comb = comb_val(n, mines);
    let new_sc = &pl.solution_count * &comb;

    let mut mbc: Vec<BigNumber> = if comb.a != 1.0 || comb.b != 0 {
        pl.mine_box_count.iter().map(|x| x * &comb).collect()
    } else {
        pl.mine_box_count.clone()
    };
    mbc[bidx] = &BigNumber { a: mines as f64, b: 0 } * &new_sc;

    ProbabilityLine {
        mine_count: pl.mine_count + mines,
        solution_count: new_sc,
        mine_box_count: mbc,
        allocated_mines: {
            let mut am = pl.allocated_mines.clone();
            am[bidx] = mines;
            am
        },
    }
}

fn find_boundary(boxes: &[Box], mask: &[bool]) -> Vec<usize> {
    let mut bound = Vec::new();
    for (i, _b) in boxes.iter().enumerate() {
        if !mask[i] { continue; }
                bound.push(i);
    }
    bound
}

fn crunch(target: &[ProbabilityLine], _bound: &[usize], mask: &[bool]) -> Vec<ProbabilityLine> {
    if target.is_empty() { return target.to_vec(); }
    let mut sorted = target.to_vec();
    sorted.sort_by(|a, b| {
        let c = a.mine_count.cmp(&b.mine_count);
        if c != std::cmp::Ordering::Equal { return c; }
        for (i, &m) in mask.iter().enumerate() {
            if m {
                let cc = a.allocated_mines[i].cmp(&b.allocated_mines[i]);
                if cc != std::cmp::Ordering::Equal { return cc; }
            }
        }
        std::cmp::Ordering::Equal
    });

    let mut result = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let mut cur = sorted[i].clone();
        let mut j = i + 1;
        while j < sorted.len() {
            if sorted[j].mine_count != cur.mine_count { break; }
            let same = mask.iter().enumerate().all(|(k, &m)| {
                !m || sorted[j].allocated_mines[k] == cur.allocated_mines[k]
            });
            if !same { break; }
            cur.solution_count += &sorted[j].solution_count;
            for k in 0..cur.mine_box_count.len() {
                if k < mask.len() && mask[k] {
                    cur.mine_box_count[k] += &sorted[j].mine_box_count[k];
                }
            }
            j += 1;
        }
        result.push(cur);
        i = j;
    }
    result
}

fn convolve(
    a: &[ProbabilityLine],
    b: &[ProbabilityLine],
    max_mines: usize,
) -> Vec<ProbabilityLine> {
    let mut result = Vec::new();
    for ha in a {
        for hb in b {
            let mc = ha.mine_count + hb.mine_count;
            if mc > max_mines { continue; }
            let n = max(ha.mine_box_count.len(), hb.mine_box_count.len());
            let mut pl = ProbabilityLine {
                mine_count: mc,
                solution_count: &ha.solution_count * &hb.solution_count,
                mine_box_count: vec![BigNumber { a: 0.0, b: 0 }; n],
                allocated_mines: vec![0; n],
            };
            for k in 0..n {
                let w1 = if k < ha.mine_box_count.len() {
                    &ha.mine_box_count[k] * &hb.solution_count
                } else {
                    BigNumber { a: 0.0, b: 0 }
                };
                let w2 = if k < hb.mine_box_count.len() {
                    &hb.mine_box_count[k] * &ha.solution_count
                } else {
                    BigNumber { a: 0.0, b: 0 }
                };
                pl.mine_box_count[k] = &w1 + &w2;
            }
            for k in 0..min(ha.allocated_mines.len(), n) {
                pl.allocated_mines[k] += ha.allocated_mines[k];
            }
            for k in 0..min(hb.allocated_mines.len(), n) {
                pl.allocated_mines[k] += hb.allocated_mines[k];
            }
            result.push(pl);
        }
    }

    result.sort_by(|a, b| a.mine_count.cmp(&b.mine_count));
    let mut merged = Vec::new();
    if result.is_empty() { return merged; }
    let mut cur = result[0].clone();
    for i in 1..result.len() {
        if result[i].mine_count == cur.mine_count {
            cur.solution_count += &result[i].solution_count;
            for k in 0..cur.mine_box_count.len() {
                cur.mine_box_count[k] += &result[i].mine_box_count[k];
            }
        } else {
            merged.push(cur);
            cur = result[i].clone();
        }
    }
    merged.push(cur);
    merged
}
