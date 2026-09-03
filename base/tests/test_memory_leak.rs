use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};

static LIVE_ALLOCS: AtomicIsize = AtomicIsize::new(0);

struct CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = System.alloc(layout);
        if !p.is_null() {
            LIVE_ALLOCS.fetch_add(1, Ordering::SeqCst);
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        LIVE_ALLOCS.fetch_sub(1, Ordering::SeqCst);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let p = System.realloc(ptr, layout, new_size);
        if !p.is_null() {
            // net unchanged on success
        }
        p
    }
}

#[global_allocator]
static ALLOC: CountingAlloc = CountingAlloc;

fn make_board() -> Vec<Vec<i32>> {
    let mut board = vec![vec![10i32; 30]; 16];
    // a few numbered clues with covered neighbors to form edge/witness cells
    board[3][3] = 2;
    board[3][4] = 1;
    board[4][3] = 1;
    board[6][9] = 3;
    board[7][8] = 1;
    board[5][15] = 2;
    board[6][16] = 1;
    board[5][17] = 1;
    board[8][22] = 2;
    board[9][21] = 1;
    board[9][23] = 1;
    board
}

#[test]
fn cal_probability_onboard_does_not_leak() {
    let board = make_board();

    // Warm-up + establish baseline live alloc count after a full call completes.
    for _ in 0..20 {
        let _ = ms_toollib::cal_probability_onboard(&board, 99.0);
    }
    let baseline = LIVE_ALLOCS.load(Ordering::SeqCst);

    // Run many iterations. If a cycle leaks, live alloc count grows each call.
    let mut max_live = baseline;
    for _ in 0..300 {
        let _ = ms_toollib::cal_probability_onboard(&board, 99.0);
        let cur = LIVE_ALLOCS.load(Ordering::SeqCst);
        if cur > max_live {
            max_live = cur;
        }
    }
    let after = LIVE_ALLOCS.load(Ordering::SeqCst);

    // After each call, allocations should return to (near) baseline.
    // Allow small drift for temporary allocations, but a leak would accumulate
    // hundreds of Box/BoxWitness allocations, so enforce a tight-ish bound.
    let drift = after - baseline;
    assert!(
        drift < 20,
        "live allocations grew by {} across 300 calls (baseline={}, after={}, max={}) -> leak",
        drift,
        baseline,
        after,
        max_live
    );
}
