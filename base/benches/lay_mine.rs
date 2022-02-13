use ms_toollib::{laymine_solvable_thread, laymine_solvable, laymine, laymine_op};
#[macro_use]
extern crate bencher;
use bencher::Bencher;

// 测试高级3BV抽样算法性能
// cargo bench --bench lay_mine

fn bench_laymine(bencher: &mut Bencher) {
    bencher.iter(|| laymine(16, 30, 99, 8, 15));
} // 10,951 ns/iter

fn bench_laymine_op(bencher: &mut Bencher) {
    bencher.iter(|| laymine_op(16, 30, 99, 8, 15));
} // 11,522 ns/iter

benchmark_group!(lay_mine, bench_laymine, bench_laymine_op);
benchmark_main!(lay_mine);
