// 测试步骤
// 在ms_toollib\wasm下wasm-pack build --target nodejs
// 在ms_toollib\demos\nodejs下npm install
// npm run test

const ms = require('ms-toollib');
const fs = require('fs');
const { assert } = require('console');

const video_file = '../../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf';
const data = fs.readFileSync(video_file)


let v = ms.AvfVideo.new(data, video_file)
v.parse_video();
v.analyse();
assert(v.bbbv == 127);

// 时间切到10.0秒
v.current_time = 10.0;
// 此时的x坐标，单位为像素
assert(v.x_y.x == 136);
// 此时的y坐标，单位为像素
assert(v.x_y.y == 38);
// left click efficiency
assert(v.lce == 24);


const time_period = ms.valid_time_period("Arbiter")
var newDate = new Date();
console.log(time_period.start_time);
newDate.setTime(time_period.end_time * 1000);
console.log("Arbiter video valid time:", newDate.toDateString());

for (e of v.events) {
    console.log(e.time, e.x, e.y, e.mouse, e.path, e.comments, e.mouse_state);
    // left, right, double, lce, rce, dce, flag, bbbv_solved, op_solved, isl_solved
    console.log(e.key_dynamic_params.left);
}

for (e of v.events) {
    if (e.useful_level >= 2) {
        // 该事件发生前的游戏局面在game_board_stream中的id索引
        const prior_game_board_id = e.prior_game_board_id
        // 该事件发生后的游戏局面在game_board_stream中的id索引
        const next_game_board_id = e.next_game_board_id
        // 内置的游戏局面类
        const builtin_game_board = v.game_board_stream[prior_game_board_id]
        // 内置的游戏局面类的列表类型的游戏局面
        console.log(builtin_game_board.game_board)
        // 内置的游戏局面类的每格是雷的概率
        console.log(builtin_game_board.poss)
        // 用1或2个方程求解时，得出的所有非雷的位置
        console.log(builtin_game_board.basic_not_mine)
        // 用1或2个方程求解时，得出的所有是雷的位置
        console.log(builtin_game_board.basic_is_mine)
        // 用枚举法求解时，得出的所有非雷的位置，不包括basic_not_mine
        console.log(builtin_game_board.enum_not_mine)
        // 用枚举法求解时，得出的所有是雷的位置，不包括basic_is_mine
        console.log(builtin_game_board.enum_is_mine)
    }
}

