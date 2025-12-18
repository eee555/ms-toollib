// 测试步骤
// 在ms_toollib\wasm下wasm-pack build --target nodejs
// 在ms_toollib\demos\nodejs下npm install
// npm run test

const ms = require('ms-toollib');
const fs = require('fs');
const { assert } = require('console');

const video_file = '../../test_files/exp_98763_FL_1738209872.rmv';
const data = fs.readFileSync(video_file)

// 使用二进制数据和文件名初始化
let v = new ms.RmvVideo(data, video_file)
v.parse();
v.analyse();
assert(v.bbbv == 134);

// 时间切到10.0秒
v.current_time = 10.0;

// 此时的x坐标，单位为像素
assert(v.x_y.x == 88);
// 此时的y坐标，单位为像素
assert(v.x_y.y == 143);
// left click efficiency
assert(v.lce == 11);


const time_period = ms.valid_time_period("Viennasweeper")
var newDate = new Date();
console.log(time_period.start_time);
newDate.setTime(time_period.end_time * 1000);
console.log("Viennasweeper video valid time:", newDate.toDateString());

for (e of v.events) {
    console.log(e.time, e.x, e.y, e.mouse, e.path, e.comments, e.mouse_state);
    // left, right, double, lce, rce, dce, flag, bbbv_solved, op_solved, isl_solved
    console.log(e.key_dynamic_params.left);
}

for (e of v.events) {
    if (e.useful_level >= 2) {
        // 该事件发生前的游戏局面
        const prior_game_board = e.prior_game_board
        // 该事件发生后的游戏局面
        const next_game_board = e.next_game_board
        // 内置的游戏局面类
        console.log(next_game_board.game_board)
        // 内置的游戏局面类的每格是雷的概率
        console.log(next_game_board.poss)
        // 用1或2个方程求解时，得出的所有非雷的位置
        console.log(next_game_board.basic_not_mine)
        // 用1或2个方程求解时，得出的所有是雷的位置
        console.log(next_game_board.basic_is_mine)
        // 用枚举法求解时，得出的所有非雷的位置，不包括basic_not_mine
        console.log(next_game_board.enum_not_mine)
        // 用枚举法求解时，得出的所有是雷的位置，不包括basic_is_mine
        console.log(next_game_board.enum_is_mine)
    }
}

