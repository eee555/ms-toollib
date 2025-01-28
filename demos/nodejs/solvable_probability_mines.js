const ms = require('ms-toollib');

// 初始化空数组 y 用于存储结果
let y = [];
// 定义随机仿真的局数
let n = 100;
// 遍历，雷数从 1 到 200
for (let mine_num = 1; mine_num <= 200; mine_num++) {
    // 初始化可解局面的数量
    let solvable_n = 0;
    // 随机仿真 n 局，统计最后结果
    for (let i = 0; i < n; i++) {
        // 标准埋雷，16 行、30 列、mine_num 个雷、从第 9 行、16 列开始扫
        let board = ms.laymine(16, 30, mine_num, 8, 15);
        // win7 规则埋雷，起手必开空
        // let board = ms.laymine_op(16, 30, mine_num, 8, 15);
        // 调用函数判断这个局面是否无猜
        if (ms.is_solvable(board, 8, 15)) {
            solvable_n++;
        }
    }
    // 计算不可解局面的比例并添加到 y 数组中
    y.push(1 - solvable_n / n);
}
// 打印结果
console.log(y);

