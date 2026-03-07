use ms_toollib::{cal_cell_nums, cal_isl, cal_op, laymine, refresh_matrix, refresh_matrixs};
#[macro_use]
extern crate bencher;
use bencher::Bencher;

// cargo bench --bench utils_bench
fn bench_cal_op(bencher: &mut Bencher) {
    let board = laymine(16, 30, 99, 8, 15);
    bencher.iter(|| cal_op(&board));
}

fn bench_cal_isl(bencher: &mut Bencher) {
    let board = laymine(16, 30, 99, 8, 15);
    bencher.iter(|| cal_isl(&board));
}

fn bench_cal_cell_nums(bencher: &mut Bencher) {
    let board = laymine(16, 30, 99, 8, 15);
    bencher.iter(|| cal_cell_nums(&board));
}


benchmark_group!(
    utils_bench,
    bench_cal_op,
    bench_cal_isl,
    bench_cal_cell_nums,
);
benchmark_main!(utils_bench);
