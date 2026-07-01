use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::{Mutex, OnceLock};

use num_bigint::BigUint;
use num_traits::ToPrimitive;

use crate::binomial::{Binomial, BinomialCache};
use crate::tile::Tile;

pub const PLAY_STYLE_FLAGS: usize = 1;
pub const PLAY_STYLE_NOFLAGS: usize = 2;
pub const PLAY_STYLE_EFFICIENCY: usize = 3;
pub const PLAY_STYLE_NOFLAGS_EFFICIENCY: usize = 4;

pub const ACTION_CLEAR: usize = 1;
pub const ACTION_FLAG: usize = 2;

pub const BOMB: u8 = 9;

const SMALL_COMBINATIONS: [[usize; 9]; 9] = [
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

static BINOMIAL_CACHE: OnceLock<Mutex<BinomialCache>> = OnceLock::new();

pub fn init_binomial_cache(cache: BinomialCache) {
    BINOMIAL_CACHE
        .set(Mutex::new(cache))
        .unwrap_or_else(|_| panic!("BinomialCache already initialized"));
}

fn get_binomial_cache() -> &'static Mutex<BinomialCache> {
    BINOMIAL_CACHE.get_or_init(|| {
        let binomial = Binomial::new(65000, 500);
        let cache = BinomialCache::new(5000, 500, binomial);
        Mutex::new(cache)
    })
}

pub fn combination(mines: usize, squares: usize) -> BigUint {
    get_binomial_cache()
        .lock()
        .unwrap()
        .get_binomial(mines, squares)
}

pub fn divide_bigint(numerator: &BigUint, denominator: &BigUint, _dp: usize) -> f64 {
    divide_bigint_exact(numerator, denominator)
}

fn divide_bigint_exact(numerator: &BigUint, denominator: &BigUint) -> f64 {
    if *denominator == BigUint::from(0u32) {
        return 0.0;
    }
    if *numerator == BigUint::from(0u32) {
        return 0.0;
    }

    let max_bits = numerator.bits().max(denominator.bits());
    let shift = max_bits.saturating_sub(1024) as usize;
    let scaled_numerator = numerator >> shift;
    let scaled_denominator = denominator >> shift;

    let n = scaled_numerator.to_f64().unwrap_or(f64::INFINITY);
    let d = scaled_denominator.to_f64().unwrap_or(f64::INFINITY);
    n / d
}

pub trait Board {
    fn get_adjacent(&self, tile: &Tile) -> Vec<Rc<Tile>>;
    fn adjacent_found_mine_count(&self, tile: &Tile) -> usize;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

pub struct Box {
    pub processed: Cell<bool>,
    pub uid: usize,
    pub min_mines: Cell<usize>,
    pub max_mines: Cell<usize>,
    pub tiles: RefCell<Vec<Rc<Tile>>>,
    pub empty_tiles: Cell<usize>,
    pub box_witnesses: RefCell<Vec<Rc<BoxWitness>>>,
    pub mine_tally: RefCell<BigUint>,
}

pub struct BoxWitness {
    pub tile: Rc<Tile>,
    pub boxes: RefCell<Vec<Rc<Box>>>,
    pub tiles: Vec<Rc<Tile>>,
    pub processed: Cell<bool>,
    pub mines_to_find: isize,
}

impl BoxWitness {
    pub fn new(board: &dyn Board, tile: &Rc<Tile>) -> Self {
        let mut mines_to_find = tile.get_value() as isize;
        let adj = board.get_adjacent(tile);
        let mut tiles = Vec::new();
        for t in adj {
            if t.is_solver_found_bomb() {
                mines_to_find -= 1;
            } else if t.is_covered() {
                tiles.push(t);
            }
        }
        BoxWitness {
            tile: tile.clone(),
            boxes: RefCell::new(Vec::new()),
            tiles,
            processed: Cell::new(false),
            mines_to_find,
        }
    }

    pub fn overlap(&self, other: &BoxWitness) -> bool {
        let dx = if other.tile.x > self.tile.x { other.tile.x - self.tile.x } else { self.tile.x - other.tile.x };
        let dy = if other.tile.y > self.tile.y { other.tile.y - self.tile.y } else { self.tile.y - other.tile.y };
        if dx > 2 || dy > 2 {
            return false;
        }
        for tile1 in &other.tiles {
            for tile2 in &self.tiles {
                if tile1.is_equal(tile2) {
                    return true;
                }
            }
        }
        false
    }

    pub fn equivalent(&self, other: &BoxWitness) -> bool {
        if self.tiles.len() != other.tiles.len() {
            return false;
        }
        let dx = if other.tile.x > self.tile.x { other.tile.x - self.tile.x } else { self.tile.x - other.tile.x };
        let dy = if other.tile.y > self.tile.y { other.tile.y - self.tile.y } else { self.tile.y - other.tile.y };
        if dx > 2 || dy > 2 {
            return false;
        }
        for l1 in &self.tiles {
            let mut found = false;
            for l2 in &other.tiles {
                if l2.index == l1.index {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        true
    }

    pub fn add_box(&self, b: &Rc<Box>) {
        self.boxes.borrow_mut().push(b.clone());
    }
}

impl Box {
    pub fn new(box_witnesses: &[Rc<BoxWitness>], tile: &Rc<Tile>, uid: usize) -> Self {
        let mut my_witnesses = Vec::new();
        for bw in box_witnesses {
            if tile.is_adjacent(&bw.tile) {
                my_witnesses.push(bw.clone());
            }
        }
        Box {
            processed: Cell::new(false),
            uid,
            min_mines: Cell::new(0),
            max_mines: Cell::new(0),
            tiles: RefCell::new(vec![tile.clone()]),
            empty_tiles: Cell::new(0),
            box_witnesses: RefCell::new(my_witnesses),
            mine_tally: RefCell::new(BigUint::from(0u32)),
        }
    }

    pub fn fits(&self, tile: &Tile, count: usize) -> bool {
        let bw = self.box_witnesses.borrow();
        if count != bw.len() {
            return false;
        }
        for w in bw.iter() {
            if !w.tile.is_adjacent(tile) {
                return false;
            }
        }
        true
    }

    pub fn calculate(&self, mines_left: usize) {
        let tile_count = self.tiles.borrow().len();
        self.max_mines.set(if tile_count < mines_left { tile_count } else { mines_left });
        self.min_mines.set(0);
        let bw = self.box_witnesses.borrow();
        for w in bw.iter() {
            let mtf = w.mines_to_find as usize;
            if mtf < self.max_mines.get() {
                self.max_mines.set(mtf);
            }
            if w.boxes.borrow().len() == 1 {
                self.min_mines.set(mtf);
            }
        }
    }

    pub fn increment_empty_tiles(&self) {
        self.empty_tiles.set(self.empty_tiles.get() + 1);
        let tile_count = self.tiles.borrow().len();
        let empty = self.empty_tiles.get();
        let max_possible = tile_count - empty;
        if self.max_mines.get() > max_possible {
            self.max_mines.set(max_possible);
        }
    }

    pub fn add_tile(&self, tile: &Rc<Tile>) {
        self.tiles.borrow_mut().push(tile.clone());
    }

    pub fn contains(&self, tile: &Tile) -> bool {
        for t in self.tiles.borrow().iter() {
            if t.index == tile.index {
                return true;
            }
        }
        false
    }
}

#[derive(Clone)]
pub struct ProbabilityLine {
    pub mine_count: usize,
    pub solution_count: BigUint,
    pub mine_box_count: Vec<BigUint>,
    pub allocated_mines: Vec<usize>,
}

impl ProbabilityLine {
    pub fn new(box_count: usize, solution_count: Option<BigUint>) -> Self {
        let sol = solution_count.unwrap_or(BigUint::from(0u32));
        ProbabilityLine {
            mine_count: 0,
            solution_count: sol,
            mine_box_count: vec![BigUint::from(0u32); box_count],
            allocated_mines: vec![0; box_count],
        }
    }
}

pub struct NextWitness {
    pub box_witness: Rc<BoxWitness>,
    pub old_boxes: Vec<Rc<Box>>,
    pub new_boxes: Vec<Rc<Box>>,
}

impl NextWitness {
    pub fn new(box_witness: Rc<BoxWitness>) -> Self {
        let mut old_boxes = Vec::new();
        let mut new_boxes = Vec::new();
        {
            let bw_boxes = box_witness.boxes.borrow();
            for b in bw_boxes.iter() {
                if b.processed.get() {
                    old_boxes.push(b.clone());
                } else {
                    new_boxes.push(b.clone());
                }
            }
        }
        NextWitness { box_witness, old_boxes, new_boxes }
    }
}

pub struct MergeSorter {
    pub checks: Vec<usize>,
}

impl MergeSorter {
    pub fn new(boundary_boxes: Option<&[Rc<Box>]>) -> Self {
        match boundary_boxes {
            Some(bxs) => {
                let checks: Vec<usize> = bxs.iter().map(|b| b.uid).collect();
                MergeSorter { checks }
            }
            None => MergeSorter { checks: Vec::new() },
        }
    }

    pub fn compare(&self, p1: &ProbabilityLine, p2: &ProbabilityLine) -> Ordering {
        let c = p1.mine_count.cmp(&p2.mine_count);
        if c != Ordering::Equal {
            return c;
        }
        for &idx in &self.checks {
            let c = p1.allocated_mines[idx].cmp(&p2.allocated_mines[idx]);
            if c != Ordering::Equal {
                return c;
            }
        }
        Ordering::Equal
    }
}

pub struct DeadCandidate {
    pub candidate: Option<Rc<Tile>>,
    pub my_box: Option<Rc<Box>>,
    pub is_alive: bool,
    pub good_boxes: Vec<Rc<Box>>,
    pub bad_boxes: Vec<Rc<Box>>,
    pub first_check: bool,
    pub total: usize,
}

impl DeadCandidate {
    pub fn new() -> Self {
        DeadCandidate {
            candidate: None,
            my_box: None,
            is_alive: false,
            good_boxes: Vec::new(),
            bad_boxes: Vec::new(),
            first_check: true,
            total: 0,
        }
    }
}

#[derive(Clone)]
pub struct Link {
    pub witness: Option<Rc<BoxWitness>>,
    pub tile1: Option<Rc<Tile>>,
    pub closed1: bool,
    pub dead1: bool,
    pub tile2: Option<Rc<Tile>>,
    pub closed2: bool,
    pub dead2: bool,
    pub processed: bool,
    pub pseudo: bool,
    pub unavoidable: bool,
    pub breaker: Vec<Rc<Tile>>,
}

impl Link {
    pub fn new() -> Self {
        Link {
            witness: None,
            tile1: None,
            closed1: true,
            dead1: false,
            tile2: None,
            closed2: true,
            dead2: false,
            processed: false,
            pseudo: false,
            unavoidable: true,
            breaker: Vec::new(),
        }
    }
}

pub struct Chain {
    pub whole5050: Vec<Rc<Tile>>,
    pub living5050: Vec<Rc<Tile>>,
    pub pseudo_tiles: Vec<Rc<Tile>>,
    pub open_tile: Option<Rc<Tile>>,
    pub open_tile2: Option<Rc<Tile>>,
    pub second_pass: bool,
    pub pseudo: bool,
    pub breaker: Vec<Rc<Tile>>,
}

impl Chain {
    pub fn new() -> Self {
        Chain {
            whole5050: Vec::new(),
            living5050: Vec::new(),
            pseudo_tiles: Vec::new(),
            open_tile: None,
            open_tile2: None,
            second_pass: false,
            pseudo: false,
            breaker: Vec::new(),
        }
    }
}

pub struct Action {
    pub x: usize,
    pub y: usize,
    pub prob: f64,
    pub action: usize,
    pub dead: bool,
    pub pruned: bool,
    pub progress: f64,
    pub expected_clears: f64,
    pub weight: f64,
    pub max_solutions: f64,
    pub common_clears: Option<Vec<Rc<Tile>>>,
    pub dominating_tile: Option<Rc<Tile>>,
}

impl Action {
    pub fn new(x: usize, y: usize, prob: f64, action: usize) -> Self {
        Action {
            x, y, prob, action,
            dead: false, pruned: false, progress: 0.0,
            expected_clears: 0.0, weight: prob, max_solutions: 0.0,
            common_clears: None, dominating_tile: None,
        }
    }

    pub fn as_text(&self) -> String {
        format!("({},{})", self.x, self.y)
    }
}

pub struct SequentialIterator {
    pub number_holes: usize,
    pub number_balls: usize,
    pub sample: Vec<usize>,
    pub more: bool,
    pub index: usize,
}

impl SequentialIterator {
    pub fn new(n: usize, m: usize) -> Self {
        if n == 0 {
            return SequentialIterator {
                number_holes: m, number_balls: n,
                sample: Vec::new(), more: false, index: 0,
            };
        }
        let mut sample: Vec<usize> = (0..n).collect();
        let index = n - 1;
        sample[index] = sample[index].wrapping_sub(1);
        SequentialIterator { number_holes: m, number_balls: n, sample, more: true, index }
    }

    pub fn get_next_sample(&mut self) -> Option<&[usize]> {
        if !self.more || self.number_balls == 0 {
            return None;
        }
        self.index = self.number_balls - 1;
        self.sample[self.index] = self.sample[self.index].wrapping_add(1);
        while self.sample[self.index] >= self.number_holes - self.number_balls + 1 + self.index {
            if self.index == 0 {
                self.more = false;
                return None;
            }
            self.index -= 1;
            self.sample[self.index] = self.sample[self.index].wrapping_add(1);
        }
        while self.index != self.number_balls - 1 {
            self.index += 1;
            self.sample[self.index] = self.sample[self.index - 1] + 1;
        }
        Some(&self.sample)
    }
}

pub struct WitnessWebIterator {
    pub sample: Vec<usize>,
    pub tiles: Vec<Rc<Tile>>,
    pub cogs: Vec<SequentialIterator>,
    pub square_offset: Vec<usize>,
    pub mine_offset: Vec<usize>,
    pub iterations_done: u64,
    pub top: isize,
    pub bottom: isize,
    pub done: bool,
    pub cycles: BigUint,
    pub mines_left: usize,
    pub tiles_left: usize,
}

impl WitnessWebIterator {
    pub fn new(
        independent_witnesses: &[Rc<BoxWitness>],
        all_covered_tiles: &[Rc<Tile>],
        mines_left: usize,
        tiles_left: usize,
        rotation: isize,
    ) -> Self {
        let (bottom, mut done) = if rotation == -1 { (0isize, false) } else { (1isize, false) };

        let mut loc: Vec<Rc<Tile>> = Vec::new();
        let mut ind_squares: usize = 0;
        let mut ind_mines: usize = 0;
        let mut cogs: Vec<SequentialIterator> = Vec::new();
        let mut square_offset: Vec<usize> = Vec::new();
        let mut mine_offset: Vec<usize> = Vec::new();
        let mut cycles = BigUint::from(1u32);

        for w in independent_witnesses {
            square_offset.push(ind_squares);
            mine_offset.push(ind_mines);
            cogs.push(SequentialIterator::new(w.mines_to_find as usize, w.tiles.len()));
            ind_squares += w.tiles.len();
            ind_mines += w.mines_to_find as usize;
            for t in &w.tiles {
                loc.push(t.clone());
            }
            cycles = cycles * combination(w.mines_to_find as usize, w.tiles.len());
        }

        for l in all_covered_tiles {
            let mut skip = false;
            for existing in &loc {
                if existing.is_equal(l) {
                    skip = true;
                    break;
                }
            }
            if !skip {
                loc.push(l.clone());
            }
        }

        let tiles = loc;

        if mines_left < ind_mines || mines_left - ind_mines > tiles_left - ind_squares {
            done = true;
            return WitnessWebIterator {
                sample: Vec::new(), tiles, cogs, square_offset, mine_offset,
                iterations_done: 0, top: 0, bottom, done, cycles, mines_left, tiles_left,
            };
        }

        if mines_left > ind_mines {
            square_offset.push(ind_squares);
            mine_offset.push(ind_mines);
            cogs.push(SequentialIterator::new(mines_left - ind_mines, tiles_left - ind_squares));
            cycles = cycles * combination(mines_left - ind_mines, tiles_left - ind_squares);
        }

        let top = (cogs.len() as isize) - 1;
        let sample_size = mines_left;
        let mut sample = vec![0usize; sample_size];

        if !cogs.is_empty() {
            for i in 0..(top as usize) {
                if let Some(s) = cogs[i].get_next_sample() {
                    for j in 0..s.len() {
                        sample[mine_offset[i] + j] = square_offset[i] + s[j];
                    }
                }
            }
        }

        WitnessWebIterator {
            sample, tiles, cogs, square_offset, mine_offset,
            iterations_done: 0, top, bottom, done, cycles, mines_left, tiles_left,
        }
    }

    pub fn get_sample(&mut self) -> Option<&[usize]> {
        if self.done {
            return None;
        }
        let mut index = self.top as usize;
        if index >= self.cogs.len() {
            self.done = true;
            return None;
        }
        let mut s = self.cogs[index].get_next_sample();
        while s.is_none() && index > self.bottom as usize {
            index -= 1;
            s = self.cogs[index].get_next_sample();
        }
        if index == self.bottom as usize && s.is_none() {
            self.done = true;
            return None;
        }
        if let Some(sample_slice) = s {
            for j in 0..sample_slice.len() {
                self.sample[self.mine_offset[index] + j] = self.square_offset[index] + sample_slice[j];
            }
        }
        index += 1;
        while index <= self.top as usize {
            self.cogs[index] = SequentialIterator::new(self.cogs[index].number_balls, self.cogs[index].number_holes);
            if let Some(next_s) = self.cogs[index].get_next_sample() {
                for j in 0..next_s.len() {
                    self.sample[self.mine_offset[index] + j] = self.square_offset[index] + next_s[j];
                }
            }
            index += 1;
        }
        self.iterations_done += 1;
        Some(&self.sample)
    }
}

pub struct Cruncher {
    pub tiles: Vec<Rc<Tile>>,
    pub witnesses: Vec<Rc<BoxWitness>>,
    pub all_solutions: Vec<Vec<u8>>,
    pub current_flags_tiles: Vec<usize>,
    pub current_flags_witnesses: Vec<usize>,
    pub duration: u64,
}

impl Cruncher {
    pub fn new(
        board: &dyn Board,
        dependent_witnesses: Vec<Rc<BoxWitness>>,
        tiles: Vec<Rc<Tile>>,
    ) -> Self {
        let mut current_flags_tiles = Vec::with_capacity(tiles.len());
        for t in &tiles {
            current_flags_tiles.push(board.adjacent_found_mine_count(t));
        }
        let mut current_flags_witnesses = Vec::with_capacity(dependent_witnesses.len());
        for w in &dependent_witnesses {
            current_flags_witnesses.push(board.adjacent_found_mine_count(&w.tile));
        }
        Cruncher {
            tiles,
            witnesses: dependent_witnesses,
            all_solutions: Vec::new(),
            current_flags_tiles,
            current_flags_witnesses,
            duration: 0,
        }
    }

    pub fn crunch(&mut self, board: &dyn Board, iterator: &mut WitnessWebIterator) -> usize {
        let start = std::time::Instant::now();
        let mut candidates = 0;
        while let Some(sample) = iterator.get_sample() {
            if self.check_sample(board, sample) {
                candidates += 1;
            }
        }
        self.duration = start.elapsed().as_millis() as u64;
        candidates
    }

    fn check_sample(&mut self, board: &dyn Board, sample: &[usize]) -> bool {
        let mine: Vec<Rc<Tile>> = sample.iter().map(|&idx| self.tiles[idx].clone()).collect();
        for i in 0..self.witnesses.len() {
            let flags1 = self.current_flags_witnesses[i];
            let mut flags2: usize = 0;
            for m in &mine {
                if m.is_adjacent(&self.witnesses[i].tile) {
                    flags2 += 1;
                }
            }
            if self.witnesses[i].tile.get_value() as usize != flags1 + flags2 {
                return false;
            }
        }
        let mut solution = vec![0u8; self.tiles.len()];
        for i in 0..self.tiles.len() {
            let mut is_mine = false;
            for &sj in sample {
                if i == sj {
                    is_mine = true;
                    break;
                }
            }
            if !is_mine {
                let mut flags2 = self.current_flags_tiles[i];
                for m in &mine {
                    if m.is_adjacent(&self.tiles[i]) {
                        flags2 += 1;
                    }
                }
                solution[i] = flags2 as u8;
            } else {
                solution[i] = BOMB;
            }
        }
        self.all_solutions.push(solution);
        true
    }
}

pub struct ProbabilityEngine {
    pub board: std::boxed::Box<dyn Board>,
    pub options: ProbabilityOptions,
    pub play_style: usize,
    pub verbose: bool,
    pub witnessed: Vec<Rc<Tile>>,
    pub duration: u64,
    pub pruned_witnesses: Vec<Rc<BoxWitness>>,
    pub mines_left: usize,
    pub tiles_left: usize,
    pub tiles_off_edge: usize,
    pub min_total_mines: usize,
    pub max_total_mines: usize,
    pub boxes: Vec<Rc<Box>>,
    pub box_witnesses: Vec<Rc<BoxWitness>>,
    pub mask: Vec<bool>,
    pub dead_candidates: Vec<DeadCandidate>,
    pub dead_tiles: Vec<Rc<Tile>>,
    pub lonely_tiles: Vec<DeadCandidate>,
    pub empty_boxes: Vec<Rc<Box>>,
    pub box_prob: Vec<f64>,
    pub working_probs: Vec<ProbabilityLine>,
    pub held_probs: Vec<ProbabilityLine>,
    pub best_probability: f64,
    pub off_edge_probability: f64,
    pub off_edge_mine_tally: BigUint,
    pub best_on_edge_probability: f64,
    pub final_solutions_count: BigUint,
    pub best_living_safety: f64,
    pub blended_safety: f64,
    pub single_safest_tile: Option<Rc<Tile>>,
    pub independent_witnesses: Vec<Rc<BoxWitness>>,
    pub dependent_witnesses: Vec<Rc<BoxWitness>>,
    pub independent_mines: usize,
    pub independent_iterations: BigUint,
    pub remaining_squares: usize,
    pub living_clear_tile: usize,
    pub clear_count: usize,
    pub local_clears: Vec<Rc<Tile>>,
    pub full_analysis: bool,
    pub mines_found: Vec<Rc<Tile>>,
    pub can_do_dead_tile_analysis: bool,
    pub isolated_edge_brute_force: Option<std::boxed::Box<Cruncher>>,
    pub valid_web: bool,
    pub recursions: u64,
}

pub struct ProbabilityOptions {
    pub play_style: usize,
    pub verbose: bool,
    pub analysis_mode: bool,
    pub full_probability: bool,
}

impl ProbabilityOptions {
    pub fn new(play_style: usize, verbose: bool, analysis_mode: bool, full_probability: bool) -> Self {
        ProbabilityOptions { play_style, verbose, analysis_mode, full_probability }
    }
}

struct TileBoard {
    width: usize,
    height: usize,
    num_bombs: usize,
    tiles: Vec<Rc<Tile>>,
}

impl Board for TileBoard {
    fn get_adjacent(&self, tile: &Tile) -> Vec<Rc<Tile>> {
        let col = tile.x;
        let row = tile.y;
        let r1 = if row > 0 { row - 1 } else { 0 };
        let r2 = if row + 1 < self.height { row + 1 } else { self.height - 1 };
        let c1 = if col > 0 { col - 1 } else { 0 };
        let c2 = if col + 1 < self.width { col + 1 } else { self.width - 1 };
        let mut res = Vec::new();
        for r in r1..=r2 {
            for c in c1..=c2 {
                if !(r == row && c == col) {
                    res.push(self.tiles[r * self.width + c].clone());
                }
            }
        }
        res
    }

    fn adjacent_found_mine_count(&self, tile: &Tile) -> usize {
        let mut cnt = 0;
        for adj in self.get_adjacent(tile) {
            if adj.is_solver_found_bomb() {
                cnt += 1;
            }
        }
        cnt
    }

    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }
}

fn extract_board_state(board: &TileBoard) -> (Vec<Rc<Tile>>, Vec<Rc<Tile>>, usize, usize) {
    let mut witnesses: Vec<Rc<Tile>> = Vec::new();
    let mut work_set = std::collections::HashSet::new();
    let mut mines_left = board.num_bombs;
    let mut squares_left = 0;

    for tile in &board.tiles {
        if tile.is_solver_found_bomb() {
            mines_left = mines_left.saturating_sub(1);
            continue;
        }
        if tile.is_covered() {
            squares_left += 1;
            continue;
        }
        if tile.is_safe.get() {
            continue;
        }
        let adj = board.get_adjacent(tile);
        let mut needs_work = false;
        for a in &adj {
            if a.is_covered() && !a.is_solver_found_bomb() {
                needs_work = true;
                work_set.insert(a.index);
            }
        }
        if needs_work {
            witnesses.push(tile.clone());
        }
    }

    let mut witnessed: Vec<Rc<Tile>> = Vec::new();
    for idx in &work_set {
        witnessed.push(board.tiles[*idx].clone());
    }

    (witnesses, witnessed, squares_left, mines_left)
}

impl ProbabilityEngine {
    pub fn new(
        board: std::boxed::Box<dyn Board>,
        all_witnesses: Vec<Rc<Tile>>,
        all_witnessed: Vec<Rc<Tile>>,
        squares_left: usize,
        mines_left: usize,
        options: ProbabilityOptions,
    ) -> Self {
        let verbose = options.verbose;
        let play_style = options.play_style;
        let tiles_off_edge = squares_left - all_witnessed.len();
        let min_total_mines = if mines_left > tiles_off_edge { mines_left - tiles_off_edge } else { 0 };
        let mut pe = ProbabilityEngine {
            board,
            options,
            play_style,
            verbose,
            witnessed: all_witnessed.clone(),
            duration: 0,
            pruned_witnesses: Vec::new(),
            mines_left,
            tiles_left: squares_left,
            tiles_off_edge,
            min_total_mines,
            max_total_mines: mines_left,
            boxes: Vec::new(),
            box_witnesses: Vec::new(),
            mask: Vec::new(),
            dead_candidates: Vec::new(),
            dead_tiles: Vec::new(),
            lonely_tiles: Vec::new(),
            empty_boxes: Vec::new(),
            box_prob: Vec::new(),
            working_probs: Vec::new(),
            held_probs: Vec::new(),
            best_probability: 0.0,
            off_edge_probability: 0.0,
            off_edge_mine_tally: BigUint::from(0u32),
            best_on_edge_probability: 0.0,
            final_solutions_count: BigUint::from(0u32),
            best_living_safety: 0.0,
            blended_safety: 0.0,
            single_safest_tile: None,
            independent_witnesses: Vec::new(),
            dependent_witnesses: Vec::new(),
            independent_mines: 0,
            independent_iterations: BigUint::from(1u32),
            remaining_squares: 0,
            living_clear_tile: 0,
            clear_count: 0,
            local_clears: Vec::new(),
            full_analysis: false,
            mines_found: Vec::new(),
            can_do_dead_tile_analysis: true,
            isolated_edge_brute_force: None,
            valid_web: true,
            recursions: 0,
        };

        if get_binomial_cache().lock().unwrap().get_max_n() < tiles_off_edge {
            pe.valid_web = false;
            pe.write_to_console(&format!("Off-edge tiles too many, cannot compute binomials, max allowed {}", get_binomial_cache().lock().unwrap().get_max_n()), true);
            return pe;
        }

        if mines_left == 0 {
            pe.valid_web = false;
            pe.write_to_console(&format!("Remaining mines = {}", mines_left), true);
            return pe;
        }

        // Step 1: Create BoxWitness for each witness, dedup
        let mut pruned: usize = 0;
        for wit in &all_witnesses {
            let box_wit = Rc::new(BoxWitness::new(&*pe.board, wit));
            if box_wit.mines_to_find < 0 || (box_wit.mines_to_find as usize) > box_wit.tiles.len() {
                pe.valid_web = false;
                pe.write_to_console(&format!("Mine count anomaly: {}", box_wit.mines_to_find), true);
            }
            let mut duplicate = false;
            for w in &pe.box_witnesses {
                if w.equivalent(&box_wit) {
                    duplicate = true;
                    break;
                }
            }
            if !duplicate {
                pe.pruned_witnesses.push(box_wit.clone());
            } else {
                pruned += 1;
            }
            pe.box_witnesses.push(box_wit);
        }
        pe.write_to_console(&format!("Pruned {} duplicate witnesses", pruned), false);
        pe.write_to_console(&format!("Total BoxWitnesses: {}", pe.box_witnesses.len()), false);

        // Step 2: Assign witnessed tiles to boxes
        let mut uid: usize = 0;
        for tile in &pe.witnessed {
            let mut count: usize = 0;
            for w in &all_witnesses {
                if tile.is_adjacent(w) {
                    count += 1;
                }
            }
            let mut found = false;
            for b in &pe.boxes {
                if b.fits(tile, count) {
                    b.add_tile(tile);
                    found = true;
                    break;
                }
            }
            if !found {
                let new_box = Rc::new(Box::new(&pe.box_witnesses, tile, uid));
                // Add this box to its matching witnesses
                let bw = new_box.box_witnesses.borrow();
                for w in bw.iter() {
                    w.add_box(&new_box);
                }
                drop(bw);
                uid += 1;
                pe.boxes.push(new_box);
            }
        }

        // Step 3: Calculate min/max mines for each box
        for b in &pe.boxes {
            b.calculate(pe.mines_left);
        }

        pe
    }

    pub fn write_to_console(&self, text: &str, always: bool) {
        if always || self.verbose {
            eprintln!("{}", text);
        }
    }

    pub fn check_for_unavoidable_guess(&self) -> Option<Vec<Rc<Tile>>> {
        for witness in &self.pruned_witnesses {
            if witness.mines_to_find > 0
                && (witness.mines_to_find as usize) < witness.tiles.len()
                && witness.tiles.len() > 1
            {
                let mut unavoidable = true;
                'check: for tile in &witness.tiles {
                    let adj_tiles = self.board.get_adjacent(tile);
                    for adj_tile in &adj_tiles {
                        if adj_tile.is_solver_found_bomb() {
                            continue;
                        }
                        let mut to_check = true;
                        for other_tile in &witness.tiles {
                            if other_tile.is_equal(adj_tile) {
                                to_check = false;
                                break;
                            }
                        }
                        if to_check {
                            for other_tile in &witness.tiles {
                                if !adj_tile.is_adjacent(other_tile) {
                                    unavoidable = false;
                                    break 'check;
                                }
                            }
                        }
                    }
                }
                if unavoidable {
                    self.write_to_console(&format!("Tile {} is an unavoidable guess", witness.tile.as_text()), false);
                    return Some(self.not_dead(&witness.tiles));
                }
            }
        }
        None
    }

    pub fn check_for_unavoidable_5050(&self) -> Option<Vec<Rc<Tile>>> {
        self.write_to_console("Checking for unavoidable 50/50.", false);
        let mut links: Vec<Link> = Vec::new();
        for witness in &self.pruned_witnesses {
            if witness.mines_to_find > 0
                && (witness.mines_to_find as usize) < witness.tiles.len()
                && witness.tiles.len() > 1
            {
                let mut link = Link::new();
                link.tile1 = Some(witness.tiles[0].clone());
                link.tile2 = Some(witness.tiles[1].clone());
                for tile in &witness.tiles {
                    let adj_tiles = self.board.get_adjacent(tile);
                    for adj_tile in &adj_tiles {
                        if adj_tile.is_solver_found_bomb() {
                            continue;
                        }
                        let mut to_check = true;
                        for other_tile in &witness.tiles {
                            if other_tile.is_equal(adj_tile) {
                                to_check = false;
                                break;
                            }
                        }
                        if to_check {
                            for other_tile in &witness.tiles {
                                if !adj_tile.is_adjacent(other_tile) {
                                    link.breaker.push(adj_tile.clone());
                                    if tile.is_equal(link.tile1.as_ref().unwrap()) {
                                        link.closed1 = false;
                                    } else {
                                        link.closed2 = false;
                                    }
                                    link.unavoidable = false;
                                }
                            }
                        }
                    }
                }
                if link.unavoidable {
                    self.write_to_console(&format!("Tile {} is an unavoidable guess", witness.tile.as_text()), false);
                    return Some(self.not_dead(&witness.tiles));
                }
                if witness.mines_to_find == 1 && witness.tiles.len() == 2 {
                    links.push(link);
                }
            }
        }

        let links_len = links.len();
        let mut area5050: Vec<Rc<Tile>> = Vec::new();
        for i in 0..links_len {
            if links[i].processed || (links[i].closed1 && links[i].closed2) { continue; }
            let mut open_tile: Option<Rc<Tile>>;
            let mut open_tile2: Option<Rc<Tile>> = None;
            let mut extensions: usize = 0;
            if !links[i].closed1 {
                open_tile = links[i].tile1.clone();
                if !links[i].closed2 {
                    open_tile2 = links[i].tile2.clone();
                }
            } else {
                open_tile = links[i].tile2.clone();
            }
            area5050 = vec![links[i].tile1.as_ref().unwrap().clone(), links[i].tile2.as_ref().unwrap().clone()];
            links[i].processed = true;
            let mut no_match = false;
            while open_tile.is_some() && !no_match {
                no_match = true;
                for j in 0..links_len {
                    if i == j { continue; }
                    if !links[j].processed {
                        if links[j].tile1.as_ref().unwrap().is_equal(open_tile.as_ref().unwrap()) {
                            links[j].processed = true;
                            no_match = false;
                            let j_breaker = links[j].breaker.clone();
                            links[i].breaker.extend(j_breaker.iter().cloned());
                            if open_tile2.is_none() || !links[j].tile2.as_ref().unwrap().is_equal(open_tile2.as_ref().unwrap()) {
                                extensions += 1;
                                area5050.push(links[j].tile2.as_ref().unwrap().clone());
                            }
                            if links[j].closed2 && open_tile2.is_some() {
                                open_tile = open_tile2.take();
                            } else if links[j].closed2 || (open_tile2.is_some() && links[j].tile2.as_ref().unwrap().is_equal(open_tile2.as_ref().unwrap())) {
                                if extensions % 2 == 0 && self.no_breaker(&links[i].breaker, &area5050) {
                                    self.write_to_console(&format!("Tile {} is an unavoidable guess, with {} extensions", open_tile.as_ref().unwrap().as_text(), extensions), false);
                                    return Some(self.not_dead(&area5050));
                                } else {
                                    self.write_to_console(&format!("Tile {} is a closed extension with {} parts", open_tile.as_ref().unwrap().as_text(), extensions + 1), false);
                                    open_tile = None;
                                }
                            } else {
                                open_tile = links[j].tile2.clone();
                            }
                            break;
                        }
                        if links[j].tile2.as_ref().unwrap().is_equal(open_tile.as_ref().unwrap()) {
                            links[j].processed = true;
                            no_match = false;
                            let j_breaker = links[j].breaker.clone();
                            links[i].breaker.extend(j_breaker.iter().cloned());
                            if open_tile2.is_none() || !links[j].tile1.as_ref().unwrap().is_equal(open_tile2.as_ref().unwrap()) {
                                extensions += 1;
                                area5050.push(links[j].tile1.as_ref().unwrap().clone());
                            }
                            if links[j].closed1 && open_tile2.is_some() {
                                open_tile = open_tile2.take();
                            } else if links[j].closed1 || (open_tile2.is_some() && links[j].tile1.as_ref().unwrap().is_equal(open_tile2.as_ref().unwrap())) {
                                if extensions % 2 == 0 && self.no_breaker(&links[i].breaker, &area5050) {
                                    self.write_to_console(&format!("Tile {} is an unavoidable guess, with {} extensions", open_tile.as_ref().unwrap().as_text(), extensions), false);
                                    return Some(self.not_dead(&area5050));
                                } else {
                                    self.write_to_console(&format!("Tile {} is a closed extension with {} parts", open_tile.as_ref().unwrap().as_text(), extensions + 1), false);
                                    open_tile = None;
                                }
                            } else {
                                open_tile = links[j].tile1.clone();
                            }
                            break;
                        }
                    }
                }
            }
        }
        None
    }

    pub fn check_for_unavoidable_5050_or_pseudo(&self) -> Option<Vec<Rc<Tile>>> {
        self.write_to_console("Checking for unavoidable 50/50 or pseudo 50/50.", false);
        let mut links: Vec<Link> = Vec::new();
        let mut pseudo_links: Vec<Rc<BoxWitness>> = Vec::new();
        for witness in &self.pruned_witnesses {
            if witness.mines_to_find > 0 && (witness.mines_to_find as usize) < witness.tiles.len() && witness.tiles.len() > 1 {
                let mut link = Link::new();
                link.tile1 = Some(witness.tiles[0].clone());
                link.tile2 = Some(witness.tiles[1].clone());
                for tile in &witness.tiles {
                    let adj_tiles = self.board.get_adjacent(tile);
                    for adj_tile in &adj_tiles {
                        if adj_tile.is_solver_found_bomb() { continue; }
                        let mut to_check = true;
                        for other_tile in &witness.tiles {
                            if other_tile.is_equal(adj_tile) { to_check = false; break; }
                        }
                        if to_check {
                            for other_tile in &witness.tiles {
                                if !adj_tile.is_adjacent(other_tile) {
                                    link.breaker.push(adj_tile.clone());
                                    if tile.is_equal(link.tile1.as_ref().unwrap()) {
                                        link.closed1 = false;
                                    } else {
                                        link.closed2 = false;
                                    }
                                    link.unavoidable = false;
                                }
                            }
                        }
                    }
                }
                if link.unavoidable {
                    self.write_to_console(&format!("Tile {} is an unavoidable guess", witness.tile.as_text()), false);
                    return Some(self.not_dead(&witness.tiles));
                }
                if witness.mines_to_find == 1 {
                    if witness.tiles.len() == 2 {
                        links.push(link);
                    } else {
                        let rooted = self.find_rooted_links(witness);
                        if rooted.is_empty() {
                            pseudo_links.push(witness.clone());
                        }
                        links.extend(rooted);
                    }
                }
            }
        }
        let mut unavoidable_link: Option<Link> = None;
        for link in &links {
            if link.unavoidable {
                if !link.pseudo {
                    unavoidable_link = Some(Link { ..link.clone() });
                    break;
                }
                if unavoidable_link.is_none() {
                    unavoidable_link = Some(Link { ..link.clone() });
                }
            }
        }
        drop(unavoidable_link);
        for link in &links {
            if link.unavoidable {
                if !link.pseudo {
                    self.write_to_console(&format!("Tiles {} and {} form an unavoidable 50/50 guess", link.tile1.as_ref().unwrap().as_text(), link.tile2.as_ref().unwrap().as_text()), false);
                    return Some(self.not_dead(&[link.tile1.as_ref().unwrap().clone(), link.tile2.as_ref().unwrap().clone()]));
                }
            }
        }
        for link in &links {
            if link.unavoidable && link.pseudo {
                self.write_to_console(&format!("Tiles {} and {} form an unavoidable 50/50 guess (pseudo)", link.tile1.as_ref().unwrap().as_text(), link.tile2.as_ref().unwrap().as_text()), false);
                return Some(self.not_dead(&[link.tile1.as_ref().unwrap().clone(), link.tile2.as_ref().unwrap().clone()]));
            }
        }

        let mut chains: Vec<Chain> = Vec::new();
        let link_count = links.len();
        for i in 0..link_count {
            if links[i].processed || (links[i].closed1 && links[i].closed2) {
                continue;
            }
            let mut chain = Chain::new();
            chain.whole5050.push(links[i].tile1.as_ref().unwrap().clone());
            chain.whole5050.push(links[i].tile2.as_ref().unwrap().clone());
            chain.breaker.extend(links[i].breaker.iter().cloned());
            if links[i].pseudo { chain.pseudo = true; }
            let mut extensions = 0usize;
            if !links[i].closed1 {
                chain.open_tile = links[i].tile1.clone();
                if !links[i].closed2 {
                    chain.open_tile2 = links[i].tile2.clone();
                }
            } else {
                chain.open_tile = links[i].tile2.clone();
            }
            if !links[i].dead1 {
                chain.living5050.push(links[i].tile1.as_ref().unwrap().clone());
                if links[i].pseudo { chain.pseudo_tiles.push(links[i].tile1.as_ref().unwrap().clone()); }
            }
            if !links[i].dead2 {
                chain.living5050.push(links[i].tile2.as_ref().unwrap().clone());
                if links[i].pseudo { chain.pseudo_tiles.push(links[i].tile2.as_ref().unwrap().clone()); }
            }
            links[i].processed = true;
            let mut no_match = false;
            while chain.open_tile.is_some() && !no_match {
                no_match = true;
                for j in 0..link_count {
                    if links[j].processed || (chain.pseudo && links[j].pseudo) { continue; }
                    if links[j].tile1.as_ref().unwrap().is_equal(chain.open_tile.as_ref().unwrap()) {
                        links[j].processed = true;
                        no_match = false;
                        if links[j].pseudo {
                            chain.pseudo = true;
                            if !links[j].dead1 { chain.pseudo_tiles.push(links[j].tile1.as_ref().unwrap().clone()); }
                            if !links[j].dead2 { chain.pseudo_tiles.push(links[j].tile2.as_ref().unwrap().clone()); }
                        }
                        chain.breaker.extend(links[j].breaker.iter().cloned());
                        if chain.open_tile2.is_none() || !links[j].tile2.as_ref().unwrap().is_equal(chain.open_tile2.as_ref().unwrap()) {
                            extensions += 1;
                            chain.whole5050.push(links[j].tile2.as_ref().unwrap().clone());
                            if !links[j].dead2 { chain.living5050.push(links[j].tile2.as_ref().unwrap().clone()); }
                        }
                        if links[j].closed2 && chain.open_tile2.is_some() {
                            chain.open_tile = chain.open_tile2.take();
                        } else if links[j].closed2 || (chain.open_tile2.is_some() && links[j].tile2.as_ref().unwrap().is_equal(chain.open_tile2.as_ref().unwrap())) {
                            if extensions % 2 == 0 && self.no_breaker(&chain.breaker, &chain.whole5050) {
                                self.write_to_console(&format!("Tile {} is an unavoidable guess, with {} extensions", chain.open_tile.as_ref().unwrap().as_text(), extensions), false);
                                if links[j].pseudo {
                                    return Some(self.not_dead(&[links[j].tile1.as_ref().unwrap().clone(), links[j].tile2.as_ref().unwrap().clone()]));
                                } else if !chain.living5050.is_empty() {
                                    return Some(chain.living5050);
                                } else {
                                    return Some(chain.whole5050);
                                }
                            } else {
                                self.write_to_console(&format!("Tile {} is a closed extension with {} parts", chain.open_tile.as_ref().unwrap().as_text(), extensions + 1), false);
                                chain.open_tile = None;
                            }
                        } else {
                            chain.open_tile = links[j].tile2.clone();
                        }
                        break;
                    }
                    if links[j].tile2.as_ref().unwrap().is_equal(chain.open_tile.as_ref().unwrap()) {
                        links[j].processed = true;
                        no_match = false;
                        if links[j].pseudo {
                            chain.pseudo = true;
                            if !links[j].dead1 { chain.pseudo_tiles.push(links[j].tile1.as_ref().unwrap().clone()); }
                            if !links[j].dead2 { chain.pseudo_tiles.push(links[j].tile2.as_ref().unwrap().clone()); }
                        }
                        chain.breaker.extend(links[j].breaker.iter().cloned());
                        if chain.open_tile2.is_none() || !links[j].tile1.as_ref().unwrap().is_equal(chain.open_tile2.as_ref().unwrap()) {
                            extensions += 1;
                            chain.whole5050.push(links[j].tile1.as_ref().unwrap().clone());
                            if !links[j].dead1 { chain.living5050.push(links[j].tile1.as_ref().unwrap().clone()); }
                        }
                        if links[j].closed1 && chain.open_tile2.is_some() {
                            chain.open_tile = chain.open_tile2.take();
                        } else if links[j].closed1 || (chain.open_tile2.is_some() && links[j].tile1.as_ref().unwrap().is_equal(chain.open_tile2.as_ref().unwrap())) {
                            if extensions % 2 == 0 && self.no_breaker(&chain.breaker, &chain.whole5050) {
                                self.write_to_console(&format!("Tile {} is an unavoidable guess, with {} extensions", chain.open_tile.as_ref().unwrap().as_text(), extensions), false);
                                if links[j].pseudo {
                                    return Some(self.not_dead(&[links[j].tile1.as_ref().unwrap().clone(), links[j].tile2.as_ref().unwrap().clone()]));
                                } else if !chain.living5050.is_empty() {
                                    return Some(chain.living5050);
                                } else {
                                    return Some(chain.whole5050);
                                }
                            } else {
                                self.write_to_console(&format!("Tile {} is a closed extension with {} parts", chain.open_tile.as_ref().unwrap().as_text(), extensions + 1), false);
                                chain.open_tile = None;
                            }
                        } else {
                            chain.open_tile = links[j].tile1.clone();
                        }
                        break;
                    }
                }
                if no_match && chain.open_tile2.is_some() && !chain.second_pass {
                    let temp = chain.open_tile.take();
                    chain.open_tile = chain.open_tile2.take();
                    chain.open_tile2 = temp;
                    chain.second_pass = true;
                    no_match = false;
                }
            }
            if no_match && chain.open_tile2.is_none() {
                let text = match chain.open_tile.as_ref() {
                    Some(t) => t.as_text(),
                    None => "null".to_string(),
                };
                self.write_to_console(&format!("Tile {} is the open end of a chain consisting of {} tiles", text, chain.whole5050.len()), false);
                chains.push(chain);
            }
        }

        'top: for witness in &pseudo_links {
            let mut chain1: Option<&Chain> = None;
            let mut chain2: Option<&Chain> = None;
            let mut tally1 = BigUint::from(0u32);
            let mut tally2 = BigUint::from(0u32);
            let mut chain1_idx = usize::MAX;
            let mut chain2_idx = usize::MAX;
            for tile in &witness.tiles {
                for (ci, chain) in chains.iter().enumerate() {
                    if let Some(ref ot) = chain.open_tile {
                        if ot.is_equal(tile) {
                            if chain1.is_none() {
                                chain1 = Some(chain);
                                chain1_idx = ci;
                                if let Some(b) = self.get_box(tile) {
                                    tally1 = b.mine_tally.borrow().clone();
                                }
                                break;
                            } else {
                                chain2 = Some(chain);
                                chain2_idx = ci;
                                if let Some(b) = self.get_box(tile) {
                                    tally2 = b.mine_tally.borrow().clone();
                                }
                                break;
                            }
                        }
                    }
                }
                if chain2.is_some() { break; }
            }
            if chain1.is_some() && chain2.is_some() {
                let mut combined = Chain::new();
                if let Some(c1) = chain1 {
                    combined.whole5050.extend(c1.whole5050.iter().cloned());
                    combined.breaker.extend(c1.breaker.iter().cloned());
                }
                if let Some(c2) = chain2 {
                    combined.whole5050.extend(c2.whole5050.iter().cloned());
                    combined.breaker.extend(c2.breaker.iter().cloned());
                }
                if combined.whole5050.len() % 2 == 0 && self.no_breaker(&combined.breaker, &combined.whole5050) {
                    if tally1 == tally2 {
                        return Some(self.not_dead(&[chain1.unwrap().open_tile.as_ref().unwrap().clone(), chain2.unwrap().open_tile.as_ref().unwrap().clone()]));
                    } else {
                        let (t1, t2) = (tally1.clone(), tally2.clone());
                        if t1 < t2 {
                            return Some(vec![chain1.unwrap().open_tile.as_ref().unwrap().clone()]);
                        } else {
                            return Some(vec![chain2.unwrap().open_tile.as_ref().unwrap().clone()]);
                        }
                    }
                }
            }
        }
        None
    }

    fn find_rooted_links(&self, witness: &Rc<BoxWitness>) -> Vec<Link> {
        let mut links = Vec::new();
        for i in 0..witness.tiles.len() {
            for j in 0..witness.tiles.len() {
                let tile1 = &witness.tiles[i];
                let tile2 = &witness.tiles[j];
                if tile2.x == tile1.x && tile2.y == tile1.y.wrapping_sub(1) {
                    let mut link = Link::new();
                    link.witness = Some(witness.clone());
                    link.pseudo = true;
                    link.tile1 = Some(tile1.clone());
                    link.tile2 = Some(tile2.clone());
                    self.assess_link(&mut link);
                    if link.closed1 || link.closed2 {
                        if !link.dead1 || !link.dead2 {
                            links.push(link);
                        }
                    }
                }
                if tile2.x == tile1.x + 1 && tile2.y == tile1.y {
                    let mut link = Link::new();
                    link.pseudo = true;
                    link.tile1 = Some(tile1.clone());
                    link.tile2 = Some(tile2.clone());
                    self.assess_link(&mut link);
                    if link.closed1 || link.closed2 {
                        if !link.dead1 || !link.dead2 {
                            links.push(link);
                        }
                    }
                }
            }
        }
        links
    }

    fn assess_link(&self, link: &mut Link) {
        let tiles = [link.tile1.clone(), link.tile2.clone()];
        for tile_opt in &tiles {
            if let Some(ref tile) = tile_opt {
                let adj = self.board.get_adjacent(tile);
                for adj_tile in &adj {
                    if adj_tile.is_solver_found_bomb() { continue; }
                    let mut to_check = true;
                    for other_opt in &tiles {
                        if let Some(ref other) = other_opt {
                            if other.is_equal(adj_tile) {
                                to_check = false;
                                break;
                            }
                        }
                    }
                    if to_check {
                        for other_opt in &tiles {
                            if let Some(ref other) = other_opt {
                                if !adj_tile.is_adjacent(other) {
                                    link.breaker.push(adj_tile.clone());
                                    if tile.is_equal(link.tile1.as_ref().unwrap()) {
                                        link.closed1 = false;
                                    } else {
                                        link.closed2 = false;
                                    }
                                    link.unavoidable = false;
                                }
                            }
                        }
                    }
                }
            }
        }
        link.dead1 = link.tile1.as_ref().is_some_and(|t| self.is_dead_tile(t));
        link.dead2 = link.tile2.as_ref().is_some_and(|t| self.is_dead_tile(t));
    }

    fn not_dead(&self, area: &[Rc<Tile>]) -> Vec<Rc<Tile>> {
        let mut result: Vec<Rc<Tile>> = Vec::new();
        for tile in area {
            if !self.is_dead_tile(tile) {
                result.push(tile.clone());
            }
        }
        if result.is_empty() {
            area.to_vec()
        } else {
            result
        }
    }

    fn is_dead_tile(&self, tile: &Rc<Tile>) -> bool {
        for dt in &self.dead_tiles {
            if dt.is_equal(tile) {
                return true;
            }
        }
        false
    }

    fn is_new_mine(&self, tile: &Rc<Tile>) -> bool {
        for m in &self.mines_found {
            if m.is_equal(tile) {
                return true;
            }
        }
        false
    }

    fn no_breaker(&self, breaker: &[Rc<Tile>], area: &[Rc<Tile>]) -> bool {
        'top: for tile in breaker {
            for t5050 in area {
                if tile.is_equal(t5050) {
                    continue 'top;
                }
            }
            let mut adj_count = 0usize;
            for t5050 in area {
                if tile.is_adjacent(t5050) {
                    adj_count += 1;
                }
            }
            if adj_count % 2 != 0 {
                self.write_to_console(&format!("Tile {} breaks the 50/50 as it isn't adjacent to an even number of tiles in the extended candidate 50/50, adjacent {} of {}", tile.as_text(), adj_count, area.len()), false);
                return false;
            }
        }
        true
    }

    pub fn process(&mut self) {
        if !self.valid_web {
            self.final_solutions_count = BigUint::from(0u32);
            self.clear_count = 0;
            return;
        }
        let pe_start = std::time::Instant::now();
        self.mask = vec![false; self.boxes.len()];
        self.get_candidate_dead_locations();
        self.held_probs.push(ProbabilityLine::new(self.boxes.len(), Some(BigUint::from(1u32))));
        self.working_probs.push(ProbabilityLine::new(self.boxes.len(), Some(BigUint::from(1u32))));
        let mut next_witness = self.find_first_witness();
        while next_witness.is_some() {
            let nw = next_witness.as_ref().unwrap();
            for nb in &nw.new_boxes {
                self.mask[nb.uid] = true;
            }
            self.working_probs = self.merge_probabilities(nw);
            next_witness = self.find_next_witness(nw);
        }
        if self.local_clears.is_empty() {
            self.calculate_box_probabilities();
        } else {
            self.best_probability = 1.0;
        }
        if self.full_analysis {
            self.write_to_console("Probability engine completed full analysis - probability data available", false);
        } else {
            self.write_to_console("Probability engine did truncated analysis - probability data not available", false);
        }
        self.duration = pe_start.elapsed().as_millis() as u64;
    }

    pub fn process_edge_constraints_only(&mut self) {
        if !self.valid_web {
            self.final_solutions_count = BigUint::from(0u32);
            return;
        }
        self.mask = vec![false; self.boxes.len()];
        self.get_candidate_dead_locations();
        self.held_probs.clear();
        self.working_probs.clear();
        self.held_probs.push(ProbabilityLine::new(self.boxes.len(), Some(BigUint::from(1u32))));
        self.working_probs.push(ProbabilityLine::new(self.boxes.len(), Some(BigUint::from(1u32))));
        let mut next_witness = self.find_first_witness();
        while let Some(nw) = next_witness.as_ref() {
            for nb in &nw.new_boxes {
                self.mask[nb.uid] = true;
            }
            self.working_probs = self.merge_probabilities(nw);
            next_witness = self.find_next_witness(nw);
        }
        self.final_solutions_count = self
            .held_probs
            .iter()
            .fold(BigUint::from(0u32), |acc, pl| acc + &pl.solution_count);
    }

    fn merge_probabilities(&mut self, nw: &NextWitness) -> Vec<ProbabilityLine> {
        let mut new_probs: Vec<ProbabilityLine> = Vec::new();
        let working = self.working_probs.clone();
        for pl in &working {
            let missing_mines = (nw.box_witness.mines_to_find as usize).wrapping_sub(self.count_placed_mines(pl, nw));
            if nw.box_witness.mines_to_find < 0 {
                continue;
            }
            let needed = nw.box_witness.mines_to_find as usize;
            let placed = self.count_placed_mines(pl, nw);
            if needed < placed {
            } else if needed == placed {
                new_probs.push(pl.clone());
            } else if nw.new_boxes.is_empty() {
            } else {
                let missing = needed - placed;
                let result = self.distribute_missing_mines(pl, nw, missing, 0);
                new_probs.extend(result);
            }
        }
        nw.box_witness.processed.set(true);
        for b in &nw.new_boxes {
            b.processed.set(true);
        }
        if new_probs.len() < 100 && self.can_do_dead_tile_analysis {
            return new_probs;
        }
        self.can_do_dead_tile_analysis = false;
        let mut boundary_boxes: Vec<Rc<Box>> = Vec::new();
        for box_ in &self.boxes {
            let mut not_processed = false;
            let mut processed = false;
            for w in box_.box_witnesses.borrow().iter() {
                if w.processed.get() { processed = true; } else { not_processed = true; }
                if processed && not_processed {
                    boundary_boxes.push(box_.clone());
                    break;
                }
            }
        }
        let sorter = MergeSorter::new(Some(&boundary_boxes));
        self.crunch_by_mine_count(&mut new_probs, &sorter);
        new_probs
    }

    fn count_placed_mines(&self, pl: &ProbabilityLine, nw: &NextWitness) -> usize {
        let mut result: usize = 0;
        for b in &nw.old_boxes {
            result += pl.allocated_mines[b.uid];
        }
        result
    }

    fn distribute_missing_mines(&mut self, pl: &ProbabilityLine, nw: &NextWitness, missing_mines: usize, index: usize) -> Vec<ProbabilityLine> {
        self.recursions += 1;
        if self.recursions % 1000 == 0 {
            self.write_to_console(&format!("Probability engine recursion depth = {}", self.recursions), false);
        }
        let mut result = Vec::new();
        if nw.new_boxes.len() - index == 1 {
            if nw.new_boxes[index].max_mines.get() < missing_mines {
                return result;
            }
            if nw.new_boxes[index].min_mines.get() > missing_mines {
                return result;
            }
            if pl.mine_count + missing_mines > self.max_total_mines {
                return result;
            }
            result.push(self.extend_probability_line(pl, &nw.new_boxes[index], missing_mines));
            return result;
        }
        let max_to_place = std::cmp::min(nw.new_boxes[index].max_mines.get(), missing_mines);
        for i in nw.new_boxes[index].min_mines.get()..=max_to_place {
            let npl = self.extend_probability_line(pl, &nw.new_boxes[index], i);
            let r1 = self.distribute_missing_mines(&npl, nw, missing_mines - i, index + 1);
            result.extend(r1);
        }
        result
    }

    fn extend_probability_line(&self, pl: &ProbabilityLine, new_box: &Rc<Box>, mines: usize) -> ProbabilityLine {
        let modified_tiles_count = new_box.tiles.borrow().len() - new_box.empty_tiles.get();
        let combination = SMALL_COMBINATIONS[modified_tiles_count][mines];
        let big_com = BigUint::from(combination as u64);
        let new_solution_count = &pl.solution_count * &big_com;
        let mut result = ProbabilityLine::new(self.boxes.len(), Some(new_solution_count));
        result.mine_count = pl.mine_count + mines;
        if combination != 1 {
            for i in 0..pl.mine_box_count.len() {
                result.mine_box_count[i] = &pl.mine_box_count[i] * &big_com;
            }
        } else {
            result.mine_box_count = pl.mine_box_count.clone();
        }
        result.mine_box_count[new_box.uid] = BigUint::from(mines as u64) * &result.solution_count;
        result.allocated_mines = pl.allocated_mines.clone();
        result.allocated_mines[new_box.uid] = mines;
        result
    }

    fn store_probabilities(&mut self) {
        let mut result: Vec<ProbabilityLine> = Vec::new();
        if self.working_probs.is_empty() {
            self.held_probs.clear();
            return;
        }
        let crunched: Vec<ProbabilityLine>;
        if self.working_probs.len() == 1 {
            self.check_edge_is_isolated();
            crunched = self.working_probs.clone();
        } else {
            crunched = self.working_probs.clone();
        }
        for pl in &crunched {
            for epl in &self.held_probs {
                let mut npl = ProbabilityLine::new(self.boxes.len(), None);
                npl.mine_count = pl.mine_count + epl.mine_count;
                if npl.mine_count <= self.max_total_mines {
                    npl.solution_count = &pl.solution_count * &epl.solution_count;
                    for k in 0..npl.mine_box_count.len() {
                        let w1 = &pl.mine_box_count[k] * &epl.solution_count;
                        let w2 = &epl.mine_box_count[k] * &pl.solution_count;
                        npl.mine_box_count[k] = w1 + w2;
                    }
                    result.push(npl);
                }
            }
        }
        result.sort_by(|a, b| a.mine_count.cmp(&b.mine_count));
        self.held_probs.clear();
        if result.is_empty() {
            return;
        }
        let mut mc = result[0].mine_count;
        let mut npl = ProbabilityLine::new(self.boxes.len(), None);
        npl.mine_count = mc;
        for pl in &result {
            if pl.mine_count != mc {
                self.held_probs.push(npl);
                mc = pl.mine_count;
                npl = ProbabilityLine::new(self.boxes.len(), None);
                npl.mine_count = mc;
            }
            npl.solution_count = &npl.solution_count + &pl.solution_count;
            for j in 0..pl.mine_box_count.len() {
                npl.mine_box_count[j] = &npl.mine_box_count[j] + &pl.mine_box_count[j];
            }
        }
        self.held_probs.push(npl);
    }

    fn crunch_by_mine_count(&self, target: &mut Vec<ProbabilityLine>, sorter: &MergeSorter) {
        if target.is_empty() {
            return;
        }
        target.sort_by(|a, b| sorter.compare(a, b));
        let mut result: Vec<ProbabilityLine> = Vec::new();
        let mut i = 0;
        while i < target.len() {
            let mut cur = target[i].clone();
            i += 1;
            while i < target.len() && sorter.compare(&cur, &target[i]) == Ordering::Equal {
                self.merge_line_probabilities(&mut cur, &target[i]);
                i += 1;
            }
            result.push(cur);
        }
        self.write_to_console(&format!("{} Probability Lines compressed to {}", target.len(), result.len()), false);
        *target = result;
    }

    fn merge_line_probabilities(&self, npl: &mut ProbabilityLine, pl: &ProbabilityLine) {
        npl.solution_count = &npl.solution_count + &pl.solution_count;
        for i in 0..pl.mine_box_count.len() {
            if self.mask[i] {
                npl.mine_box_count[i] = &npl.mine_box_count[i] + &pl.mine_box_count[i];
            }
        }
    }

    fn find_first_witness(&self) -> Option<NextWitness> {
        for bw in &self.box_witnesses {
            if !bw.processed.get() {
                return Some(NextWitness::new(bw.clone()));
            }
        }
        None
    }

    fn find_next_witness(&mut self, prev_witness: &NextWitness) -> Option<NextWitness> {
        let mut best_todo = 99999usize;
        let mut best_witness: Option<Rc<BoxWitness>> = None;
        for b in &self.boxes {
            if b.processed.get() {
                let bw_ref = b.box_witnesses.borrow();
                for w in bw_ref.iter() {
                    if !w.processed.get() {
                        let mut todo = 0usize;
                        let wb_ref = w.boxes.borrow();
                        for b1 in wb_ref.iter() {
                            if !b1.processed.get() {
                                todo += 1;
                            }
                        }
                        if todo == 0 {
                            return Some(NextWitness::new(w.clone()));
                        } else if todo < best_todo {
                            best_todo = todo;
                            best_witness = Some(w.clone());
                        }
                    }
                }
            }
        }
        if let Some(ref bw) = best_witness {
            return Some(NextWitness::new(bw.clone()));
        } else {
            self.write_to_console("Current independent edge processed", false);
        }
        if self.play_style != PLAY_STYLE_EFFICIENCY
            && self.play_style != PLAY_STYLE_NOFLAGS_EFFICIENCY
            && !self.options.analysis_mode
            && !self.options.full_probability
        {
            for i in 0..self.mask.len() {
                if self.mask[i] {
                    let mut is_clear = true;
                    for wp in &self.working_probs {
                        if wp.mine_box_count[i] != BigUint::from(0u32) {
                            is_clear = false;
                            break;
                        }
                    }
                    if is_clear {
                        for tile in self.boxes[i].tiles.borrow().iter() {
                            self.write_to_console(&format!("{} has been determined to be locally clear", tile.as_text()), false);
                            self.local_clears.push(tile.clone());
                        }
                    }
                    let mut is_flag = true;
                    let tile_count = BigUint::from(self.boxes[i].tiles.borrow().len() as u64);
                    for wp in &self.working_probs {
                        if wp.mine_box_count[i] != &wp.solution_count * &tile_count {
                            is_flag = false;
                            break;
                        }
                    }
                    if is_flag {
                        for tile in self.boxes[i].tiles.borrow().iter() {
                            self.write_to_console(&format!("{} has been determined to be locally a mine", tile.as_text()), false);
                            self.mines_found.push(tile.clone());
                        }
                    }
                }
            }
            if !self.local_clears.is_empty() {
                return None;
            }
        }
        self.check_candidate_dead_locations(self.can_do_dead_tile_analysis);
        if self.can_do_dead_tile_analysis {
            let sorter = MergeSorter::new(None);
            let mut working = std::mem::take(&mut self.working_probs);
            self.crunch_by_mine_count(&mut working, &sorter);
            self.working_probs = working;
        } else {
            self.can_do_dead_tile_analysis = true;
        }
        let nw = self.find_first_witness();
        if nw.is_some() {
            self.write_to_console("Starting a new independent edge", false);
        }
        self.store_probabilities();
        self.working_probs = vec![ProbabilityLine::new(self.boxes.len(), Some(BigUint::from(1u32)))];
        self.mask = vec![false; self.boxes.len()];
        nw
    }

    fn check_candidate_dead_locations(&mut self, check_possible: bool) {
        let mut complete_scan = false;
        if self.tiles_off_edge == 0 {
            complete_scan = true;
            for flag in &self.mask {
                if !flag {
                    complete_scan = false;
                    break;
                }
            }
            if complete_scan {
                self.write_to_console("This is a complete scan", false);
            } else {
                self.write_to_console("This is not a complete scan", false);
            }
        } else {
            self.write_to_console("This is not a complete scan because there are squares off the edge", false);
        }
        let mut messages: Vec<String> = Vec::new();
        for dc in &mut self.dead_candidates {
            if dc.is_alive {
                continue;
            }
            let mut boxes_in_scope = 0usize;
            for b in &dc.good_boxes {
                if self.mask[b.uid] {
                    boxes_in_scope += 1;
                }
            }
            for b in &dc.bad_boxes {
                if self.mask[b.uid] {
                    boxes_in_scope += 1;
                }
            }
            if boxes_in_scope == 0 {
                continue;
            } else if boxes_in_scope != dc.good_boxes.len() + dc.bad_boxes.len() {
                messages.push(format!("Location {} has some boxes in scope and some out of scope so assumed alive", dc.candidate.as_ref().unwrap().as_text()));
                dc.is_alive = true;
                continue;
            }
            if !check_possible {
                messages.push(format!("Location {} was on compressed edge so assumed alive", dc.candidate.as_ref().unwrap().as_text()));
                dc.is_alive = true;
                continue;
            }
            let mut okay = true;
            let mut mine_count = 0usize;
            'line: for pl in &self.working_probs {
                if complete_scan && pl.mine_count != self.mines_left {
                    continue;
                }
                if pl.allocated_mines[dc.my_box.as_ref().unwrap().uid] == dc.my_box.as_ref().unwrap().tiles.borrow().len() {
                    mine_count += 1;
                    continue;
                }
                for b in &dc.bad_boxes {
                    let needed_mines = if b.uid == dc.my_box.as_ref().unwrap().uid {
                        BigUint::from((b.tiles.borrow().len() - 1) as u64) * &pl.solution_count
                    } else {
                        BigUint::from(b.tiles.borrow().len() as u64) * &pl.solution_count
                    };
                    if pl.mine_box_count[b.uid] != BigUint::from(0u32) && pl.mine_box_count[b.uid] != needed_mines {
                        messages.push(format!("Location {} is not dead because a bad box has neither zero or all mines: {}/{}", dc.candidate.as_ref().unwrap().as_text(), pl.mine_box_count[b.uid], needed_mines));
                        okay = false;
                        break 'line;
                    }
                }
                let mut tally = 0usize;
                for b in &dc.good_boxes {
                    tally += pl.allocated_mines[b.uid];
                }
                if dc.first_check {
                    dc.total = tally;
                    dc.first_check = false;
                } else if dc.total != tally {
                    messages.push(format!("Location {} is not dead because the sum of mines in good boxes is not constant. Was {} now {}.", dc.candidate.as_ref().unwrap().as_text(), dc.total, tally));
                    okay = false;
                    break;
                }
            }
            if !okay || mine_count == self.working_probs.len() {
                dc.is_alive = true;
            }
        }
        for msg in messages {
            self.write_to_console(&msg, false);
        }
    }

    fn get_candidate_dead_locations(&mut self) {
        for tile in &self.witnessed {
            let adj_boxes = self.get_adjacent_boxes(tile);
            if adj_boxes.is_none() {
                continue;
            }
            let adj_boxes = adj_boxes.unwrap();
            let mut dc = DeadCandidate::new();
            dc.candidate = Some(tile.clone());
            dc.my_box = self.get_box(tile);
            for box_ in &adj_boxes {
                let mut good = true;
                for square in box_.tiles.borrow().iter() {
                    if !square.is_adjacent(tile) && square.index != tile.index {
                        good = false;
                        break;
                    }
                }
                if good {
                    dc.good_boxes.push(box_.clone());
                } else {
                    dc.bad_boxes.push(box_.clone());
                }
            }
            if dc.good_boxes.is_empty() && dc.bad_boxes.is_empty() {
                self.write_to_console(&format!("{} is lonely since it has no open tiles around it", dc.candidate.as_ref().unwrap().as_text()), false);
                self.lonely_tiles.push(dc);
            } else {
                self.dead_candidates.push(dc);
            }
        }
        for dc in &self.dead_candidates {
            self.write_to_console(&format!("{} is candidate dead with {} good boxes and {} bad boxes", dc.candidate.as_ref().unwrap().as_text(), dc.good_boxes.len(), dc.bad_boxes.len()), false);
        }
    }

    fn get_box(&self, tile: &Tile) -> Option<Rc<Box>> {
        for b in &self.boxes {
            if b.contains(tile) {
                return Some(b.clone());
            }
        }
        None
    }

    fn get_adjacent_boxes(&self, loc: &Tile) -> Option<Vec<Rc<Box>>> {
        let mut result: Vec<Rc<Box>> = Vec::new();
        let adj_locs = self.board.get_adjacent(loc);
        for adj_loc in &adj_locs {
            if !adj_loc.is_covered() || adj_loc.is_solver_found_bomb() {
                continue;
            }
            let mut box_found = false;
            for box_ in &self.boxes {
                if box_.contains(adj_loc) {
                    box_found = true;
                    let mut found = false;
                    for existing in &result {
                        if box_.uid == existing.uid {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        result.push(box_.clone());
                    }
                }
            }
            if !box_found {
                return None;
            }
        }
        Some(result)
    }

    fn check_edge_is_isolated(&mut self) -> bool {
        let mut edge_tiles: Vec<Rc<Tile>> = Vec::new();
        let mut edge_witnesses: Vec<Rc<Tile>> = Vec::new();
        let mut everything = true;
        for i in 0..self.mask.len() {
            if self.mask[i] {
                for tile in self.boxes[i].tiles.borrow().iter() {
                    if !edge_tiles.iter().any(|t| t.is_equal(tile)) {
                        edge_tiles.push(tile.clone());
                    }
                }
                for w in self.boxes[i].box_witnesses.borrow().iter() {
                    if !edge_witnesses.iter().any(|t| t.is_equal(&w.tile)) {
                        edge_witnesses.push(w.tile.clone());
                    }
                }
            } else {
                everything = false;
            }
        }
        if !everything {
            for i in 0..self.mask.len() {
                if self.mask[i] {
                    for tile in self.boxes[i].tiles.borrow().iter() {
                        let adj = self.board.get_adjacent(tile);
                        for adj_tile in &adj {
                            if adj_tile.is_covered() && !adj_tile.is_solver_found_bomb() && !edge_tiles.iter().any(|t| t.is_equal(adj_tile)) {
                                self.write_to_console(&format!("Not isolated because a tile's adjacent tiles isn't on the edge: {} ==> {}", tile.as_text(), adj_tile.as_text()), false);
                                return false;
                            }
                        }
                    }
                }
            }
            self.write_to_console("*** Isolated Edge found ***", false);
            let tiles = edge_tiles;
            let witnesses = edge_witnesses;
            let mines = self.working_probs[0].mine_count;
            let new_board: std::boxed::Box<dyn Board>;
        }
        false
    }

    fn generate_independent_witnesses(&mut self) {
        self.remaining_squares = self.witnessed.len();
        for w in &self.pruned_witnesses {
            let mut okay = true;
            for iw in &self.independent_witnesses {
                if w.overlap(iw) {
                    okay = false;
                    break;
                }
            }
            if okay {
                self.remaining_squares -= w.tiles.len();
                self.independent_iterations = &self.independent_iterations * combination(w.mines_to_find as usize, w.tiles.len());
                self.independent_mines += w.mines_to_find as usize;
                self.independent_witnesses.push(w.clone());
            } else {
                self.dependent_witnesses.push(w.clone());
            }
        }
        self.write_to_console(&format!("Calculated {} independent witnesses", self.independent_witnesses.len()), false);
    }

    fn calculate_box_probabilities(&mut self) {
        let mut tally: Vec<BigUint> = vec![BigUint::from(0u32); self.boxes.len()];
        let mut total_tally = BigUint::from(0u32);
        let mut outside_tally = BigUint::from(0u32);
        for pl in &self.held_probs {
            if pl.mine_count >= self.min_total_mines && pl.mine_count <= self.mines_left {
                let mult = combination(self.mines_left - pl.mine_count, self.tiles_off_edge);
                let new_solutions = &mult * &pl.solution_count;
                outside_tally = &outside_tally + &mult * BigUint::from((self.mines_left - pl.mine_count) as u64) * &pl.solution_count;
                total_tally = &total_tally + &new_solutions;
                for j in 0..tally.len() {
                    tally[j] = &tally[j] + (&mult * &pl.mine_box_count[j]) / BigUint::from(self.boxes[j].tiles.borrow().len() as u64);
                }
            }
        }
        self.mines_found.clear();
        self.box_prob = vec![0.0; self.boxes.len()];
        for i in 0..self.boxes.len() {
            if total_tally != BigUint::from(0u32) {
                if tally[i] == total_tally {
                    self.box_prob[i] = 0.0;
                } else if tally[i] == BigUint::from(0u32) {
                    self.box_prob[i] = 1.0;
                    self.empty_boxes.push(self.boxes[i].clone());
                } else {
                    self.box_prob[i] = 1.0 - divide_bigint(&tally[i], &total_tally, 8);
                }
                *self.boxes[i].mine_tally.borrow_mut() = tally[i].clone();
            } else {
                self.box_prob[i] = 0.0;
                *self.boxes[i].mine_tally.borrow_mut() = BigUint::from(0u32);
            }
            if self.box_prob[i] == 0.0 {
                for tile in self.boxes[i].tiles.borrow().iter() {
                    self.mines_found.push(tile.clone());
                }
            }
        }
        for dc in &self.lonely_tiles {
            if self.box_prob[dc.my_box.as_ref().unwrap().uid] != 0.0 {
                self.write_to_console(&format!("PE found Lonely tile {} is dead", dc.candidate.as_ref().unwrap().as_text()), false);
                self.dead_tiles.push(dc.candidate.as_ref().unwrap().clone());
            }
        }
        for dc in &self.dead_candidates {
            if !dc.is_alive && self.box_prob[dc.my_box.as_ref().unwrap().uid] != 0.0 {
                self.write_to_console(&format!("PE found {} to be dead", dc.candidate.as_ref().unwrap().as_text()), false);
                self.dead_tiles.push(dc.candidate.as_ref().unwrap().clone());
            }
        }
        if self.tiles_off_edge != 0 && total_tally != BigUint::from(0u32) {
            self.off_edge_probability = 1.0 - divide_bigint(&outside_tally, &(&total_tally * BigUint::from(self.tiles_off_edge as u64)), 8);
            self.off_edge_mine_tally = &outside_tally / BigUint::from(self.tiles_off_edge as u64);
        } else {
            self.off_edge_probability = 0.0;
            self.off_edge_mine_tally = BigUint::from(0u32);
        }
        self.final_solutions_count = total_tally.clone();
        self.local_clears.clear();
        if total_tally > BigUint::from(0u32) {
            for i in 0..self.boxes.len() {
                if tally[i] == BigUint::from(0u32) {
                    self.clear_count += self.boxes[i].tiles.borrow().len();
                    for tile in self.boxes[i].tiles.borrow().iter() {
                        self.local_clears.push(tile.clone());
                    }
                    for tile in self.boxes[i].tiles.borrow().iter() {
                        let mut tile_living = true;
                        for dt in &self.dead_tiles {
                            if dt.is_equal(tile) {
                                tile_living = false;
                                break;
                            }
                        }
                        if tile_living {
                            self.living_clear_tile += 1;
                        }
                    }
                }
            }
        }
        let mut hwm = 0.0f64;
        let mut best_safety1 = self.off_edge_probability;
        let mut best_safety2 = self.off_edge_probability;
        let mut best_tile: Option<Rc<Tile>> = None;
        for i in 0..self.boxes.len() {
            let prob = self.box_prob[i];
            let mut box_living = false;
            for tile in self.boxes[i].tiles.borrow().iter() {
                let mut tile_living = true;
                for dt in &self.dead_tiles {
                    if dt.is_equal(tile) {
                        tile_living = false;
                        break;
                    }
                }
                if tile_living {
                    box_living = true;
                    if prob > best_safety2 {
                        if prob > best_safety1 {
                            best_safety2 = best_safety1;
                            best_safety1 = prob;
                            best_tile = Some(tile.clone());
                        } else {
                            best_safety2 = prob;
                        }
                    }
                }
            }
            if box_living || prob == 1.0 {
                if hwm < prob {
                    hwm = prob;
                }
            }
        }
        self.best_living_safety = best_safety1;
        if best_safety1 > best_safety2 {
            self.single_safest_tile = best_tile;
        }
        self.blended_safety = (best_safety1 * 4.0 + best_safety2) / 5.0;
        self.best_on_edge_probability = hwm;
        self.best_probability = if self.best_on_edge_probability > self.off_edge_probability {
            self.best_on_edge_probability
        } else {
            self.off_edge_probability
        };
        self.write_to_console(&format!("Safe tiles {}, Mines found {}", self.local_clears.len(), self.mines_found.len()), false);
        self.write_to_console(&format!("Off edge Safety is {}", self.off_edge_probability), false);
        self.write_to_console(&format!("Best on edge safety is {}", self.best_on_edge_probability), false);
        self.write_to_console(&format!("Best safety is {}", self.best_probability), false);
        self.write_to_console(&format!("Best living safety is {}", self.best_living_safety), false);
        self.write_to_console(&format!("Blended safety is {}", self.blended_safety), false);
        self.write_to_console(&format!("Game has {} candidate solutions", self.final_solutions_count), false);
        self.full_analysis = true;
    }

    pub fn get_best_candidates(&self, threshold: f64) -> Vec<Action> {
        let mut best: Vec<Action> = Vec::new();
        let test = if self.best_probability == 1.0 {
            self.best_probability
        } else {
            self.best_probability * threshold
        };
        self.write_to_console(&format!("Best probability is {} threshold is {}", self.best_probability, test), false);
        for i in 0..self.box_prob.len() {
            if self.box_prob[i] >= test {
                for tile in self.boxes[i].tiles.borrow().iter() {
                    let mut dead = false;
                    for dt in &self.dead_tiles {
                        if dt.is_equal(tile) {
                            dead = true;
                            break;
                        }
                    }
                    if !dead || self.box_prob[i] == 1.0 {
                        best.push(Action::new(tile.x, tile.y, self.box_prob[i], ACTION_CLEAR));
                    } else {
                        self.write_to_console(&format!("Tile {} is ignored because it is dead", tile.as_text()), false);
                    }
                }
            }
        }
        best.sort_by(|a, b| b.prob.partial_cmp(&a.prob).unwrap_or(Ordering::Equal));
        best
    }

    pub fn get_dead_tiles(&self) -> &[Rc<Tile>] {
        &self.dead_tiles
    }

    pub fn is_dead(&self, tile: &Tile) -> bool {
        for dt in &self.dead_tiles {
            if dt.is_equal(tile) {
                return true;
            }
        }
        false
    }

    pub fn get_probability(&self, l: &Tile) -> f64 {
        for b in &self.boxes {
            if b.contains(l) {
                return self.box_prob[b.uid];
            }
        }
        self.off_edge_probability
    }

    pub fn get_fifty_percenters(&self) -> Vec<Rc<Tile>> {
        let mut picks = Vec::new();
        for i in 0..self.box_prob.len() {
            if (self.box_prob[i] - 0.5).abs() < 1e-12 {
                for tile in self.boxes[i].tiles.borrow().iter() {
                    picks.push(tile.clone());
                }
            }
        }
        picks
    }

    pub fn set_must_be_empty(&mut self, tile: &Tile) -> bool {
        let box_ = self.get_box(tile);
        match box_ {
            None => {
                self.valid_web = false;
                false
            }
            Some(b) => {
                b.increment_empty_tiles();
                true
            }
        }
    }
}

/// JSMinesweeper 概率引擎移植版。
/// - 输入：局面、总雷数。
/// - 返回：所有边缘格子是雷的概率、内部未知格子是雷的概率、局面中总未知雷数范围（[最小可能的总雷数, 当前总雷数, 最大可能的总雷数]，包含已经标出的雷）、最大独立集格数（或其它直观反映求解复杂度的值）。
/// - 错误码：0=正常, 1=盘面矛盾, 2=枚举过长, 3=输入参数非法。
/// 输入局面中，0-8代表数字0-8，10代表未开格，11标记视为已知雷（算法标记出来的此处必定为雷），12标记视为已知安全（算法标记出来的此处必定不为雷）。
/// 总雷数可能为低于或超出上下限的整数，此时算法将其钳位为最小值或最大值；总雷数还可能为0-1之间的浮点数，此时算法应将其视为局面中雷的密度，然后再钳位为最小值或最大值
pub fn cal_probability_csp(
    board_of_game: &Vec<Vec<i32>>,
    minenum: f64,
) -> Result<(Vec<((usize, usize), f64)>, f64, [usize; 3], usize), usize> {
    use std::rc::Rc;

    // Validate input
    let height = board_of_game.len();
    if height == 0 {
        return Err(3);
    }
    let width = board_of_game[0].len();
    if width == 0 {
        return Err(3);
    }
    for row in board_of_game.iter() {
        if row.len() != width {
            return Err(3);
        }
        for &v in row.iter() {
            if v > 12 || v == 9 {
                return Err(3);
            }
        }
    }

    // Count flagged (11) cells
    let mut flagged_count = 0usize;
    for row in board_of_game.iter() {
        for &v in row.iter() {
            if v == 11 {
                flagged_count += 1;
            }
        }
    }

    let total_cells = width * height;

    // If minenum is in (0,1), treat as density
    let raw_total = if minenum > 0.0 && minenum < 1.0 {
        (minenum * total_cells as f64).floor() as usize
    } else {
        minenum as usize
    };

    let build_tiles = || {
        let mut tiles = Vec::with_capacity(total_cells);
        for r in 0..height {
            for c in 0..width {
                tiles.push(Rc::new(Tile::new(c, r, r * width + c)));
            }
        }
        for r in 0..height {
            for c in 0..width {
                let v = board_of_game[r][c];
                let tile = &tiles[r * width + c];
                match v {
                    10 => {}
                    11 => {
                        tile.set_found_bomb();
                        tile.set_covered(false);
                    }
                    12 => {
                        tile.set_covered(false);
                        tile.set_value(0);
                        tile.is_safe.set(true);
                    }
                    0..=8 => {
                        tile.set_covered(false);
                        tile.set_value(v as u8);
                    }
                    _ => unreachable!(),
                }
            }
        }
        tiles
    };

    let base_tiles = build_tiles();
    let base_board = TileBoard {
        width,
        height,
        num_bombs: flagged_count,
        tiles: base_tiles,
    };

    let (range_witnesses, range_witnessed, squares_left, _) = extract_board_state(&base_board);
    let tiles_off_edge = squares_left - range_witnessed.len();

    let cache_max = get_binomial_cache().lock().unwrap().get_max_n();
    if tiles_off_edge > cache_max {
        return Err(2);
    }

    let (min_edge_mines, max_edge_mines) = if range_witnesses.is_empty() {
        (0usize, 0usize)
    } else {
        let analysis_remaining_mines = squares_left.max(1);
        let analysis_board: std::boxed::Box<dyn Board> = std::boxed::Box::new(TileBoard {
            width,
            height,
            num_bombs: flagged_count + analysis_remaining_mines,
            tiles: base_board.tiles.clone(),
        });
        let options = ProbabilityOptions::new(PLAY_STYLE_EFFICIENCY, false, false, true);
        let mut range_pe = ProbabilityEngine::new(
            analysis_board,
            range_witnesses.clone(),
            range_witnessed.clone(),
            squares_left,
            analysis_remaining_mines,
            options,
        );
        range_pe.min_total_mines = 0;
        range_pe.max_total_mines = range_witnessed.len();
        if !range_pe.valid_web {
            return Err(1);
        }
        range_pe.process_edge_constraints_only();
        if range_pe.held_probs.is_empty() || range_pe.final_solutions_count == BigUint::from(0u32) {
            return Err(1);
        }
        let min_edge = range_pe.held_probs.iter().map(|pl| pl.mine_count).min().unwrap_or(0);
        let max_edge = range_pe.held_probs.iter().map(|pl| pl.mine_count).max().unwrap_or(0);
        (min_edge, max_edge)
    };

    let min_possible = flagged_count + min_edge_mines;
    let max_possible = flagged_count + max_edge_mines + tiles_off_edge;
    let total_mines = raw_total.max(min_possible).min(max_possible);

    if range_witnesses.is_empty() {
        let remaining_mines = total_mines.saturating_sub(flagged_count);
        let mine_prob = if squares_left == 0 {
            0.0
        } else {
            remaining_mines as f64 / squares_left as f64
        };
        let edge_probs = base_board
            .tiles
            .iter()
            .filter(|t| t.is_covered() && !t.is_solver_found_bomb())
            .map(|t| ((t.y, t.x), mine_prob))
            .collect();
        return Ok((
            edge_probs,
            mine_prob,
            [min_possible, total_mines, max_possible],
            squares_left,
        ));
    }

    let tiles = build_tiles();
    let tile_board = TileBoard {
        width,
        height,
        num_bombs: total_mines,
        tiles: tiles.clone(),
    };

    let (witnesses, witnessed, squares_left, mines_left) = extract_board_state(&tile_board);

    // No remaining mines — all probabilities are zero
    if mines_left == 0 {
        let edge_probs: Vec<((usize, usize), f64)> = witnessed
            .iter()
            .map(|t| ((t.y, t.x), 0.0))
            .collect();
        return Ok((
            edge_probs,
            0.0,
            [min_possible, total_mines, max_possible],
            0,
        ));
    }

    let board_box: std::boxed::Box<dyn Board> = std::boxed::Box::new(TileBoard {
        width,
        height,
        num_bombs: total_mines,
        tiles,
    });

    // Use full_probability=true to force full enumeration
    let options = ProbabilityOptions::new(PLAY_STYLE_EFFICIENCY, false, false, true);
    let mut pe = ProbabilityEngine::new(
        board_box,
        witnesses.clone(),
        witnessed.clone(),
        squares_left,
        mines_left,
        options,
    );

    // Check for contradiction detected during construction
    if !pe.valid_web {
        for w in &pe.box_witnesses {
            if w.mines_to_find < 0 || (w.mines_to_find as usize) > w.tiles.len() {
                return Err(1);
            }
        }
        return Err(1);
    }

    pe.process();

    // Zero valid solutions → contradiction
    if pe.final_solutions_count == BigUint::from(0u32) {
        return Err(1);
    }

    // Edge cell mine probabilities (converted from safety)
    let mut edge_probs: Vec<((usize, usize), f64)> = Vec::with_capacity(witnessed.len());
    for t in &witnessed {
        let mine_prob = match pe.get_box(t) {
            Some(b) => divide_bigint_exact(&b.mine_tally.borrow(), &pe.final_solutions_count),
            None => {
                if pe.tiles_off_edge == 0 {
                    0.0
                } else {
                    1.0 - pe.off_edge_probability
                }
            }
        };
        edge_probs.push(((t.y, t.x), mine_prob));
    }

    // Off-edge (interior) mine probability
    let off_edge_mine_prob = if pe.tiles_off_edge != 0 {
        let mut outside_tally = BigUint::from(0u32);
        let mut total_tally = BigUint::from(0u32);
        for pl in &pe.held_probs {
            if pl.mine_count >= pe.min_total_mines
                && pl.mine_count <= pe.mines_left
                && pe.mines_left - pl.mine_count <= pe.tiles_off_edge
            {
                let mult = combination(pe.mines_left - pl.mine_count, pe.tiles_off_edge);
                outside_tally = &outside_tally
                    + &mult * BigUint::from((pe.mines_left - pl.mine_count) as u64) * &pl.solution_count;
                total_tally = &total_tally + &mult * &pl.solution_count;
            }
        }
        divide_bigint_exact(
            &outside_tally,
            &(total_tally * BigUint::from(pe.tiles_off_edge as u64)),
        )
    } else {
        0.0
    };

    // API compatibility: expose the number of edge cells with explicit probabilities.
    let complexity = witnessed.len();

    Ok((
        edge_probs,
        off_edge_mine_prob,
        [min_possible, total_mines, max_possible],
        complexity,
    ))
}
