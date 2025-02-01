use ms_toollib::{sample_bbbvs_exp};
#[macro_use]
extern crate bencher;
use bencher::Bencher;

// 测试高级3BV抽样算法性能
// cargo bench --bench sample_boards
fn bench_sample_bbbvs_exp(bencher: &mut Bencher) {
    bencher.iter(|| sample_bbbvs_exp(5, 5, 1000));
} // 4,241,895 ns/iter

benchmark_group!(sample_boards, bench_sample_bbbvs_exp);
benchmark_main!(sample_boards);
