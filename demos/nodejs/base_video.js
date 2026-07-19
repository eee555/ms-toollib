// 测试步骤：
// 1. 在ms_toollib\wasm下 wasm-pack build --target nodejs
// 2. 在ms_toollib\demos\nodejs下 npm install
// 3. node base_video.js

const ms = require("ms-toollib");

function sleepMs(ms_time) {
    const sab = new SharedArrayBuffer(4);
    const int32 = new Int32Array(sab);
    Atomics.wait(int32, 0, 0, ms_time);
}

const board = [
    [1, 1, 2, 1, 1, 0, 0, 0],
    [1, -1, 2, -1, 1, 0, 0, 0],
    [1, 1, 2, 1, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [2, 2, 1, 0, 0, 0, 0, 0],
    [-1, -1, 2, 0, 0, 1, 1, 1],
    [-1, -1, 3, 0, 0, 2, -1, 2],
    [-1, -1, 2, 0, 0, 2, -1, 2],
];

let video = new ms.BaseVideo(board, 16);
sleepMs(600);

video.step("rc", 17, 16);
video.step("rr", 17, 16);
video.step("rc", 16, 49);
sleepMs(20);
video.step("rr", 16, 50);
video.step("mv", 48, 51);
video.step("mv", 42, 48);
sleepMs(20);
video.step("lc", 16, 32);
sleepMs(20);
video.step("lr", 16, 32);
sleepMs(20);
video.step("lc", 52, 0);
video.step("lr", 53, 0);
video.step("lc", 16, 32);
video.step("rc", 16, 32);
sleepMs(50);
video.step("rr", 16, 32);
sleepMs(50);
video.step("lr", 16, 32);
sleepMs(50);
video.step("lc", 0, 16);
sleepMs(50);
video.step("rc", 0, 16);
sleepMs(50);
video.step("rr", 0, 16);
console.log("left_s:", video.left_s);
sleepMs(50);
video.step("lr", 0, 16);
video.step("mv", 112, 112);
video.step("lc", 112, 112);
video.step("lr", 112, 112);
video.step("lc", 97, 112);
video.step("lr", 97, 112);

video.player_identifier = "eee555";
video.race_identifier = "G8888";
video.software = "a test software";
video.country = "CN";

console.log("game_board:", video.game_board);
console.log("player_identifier:", video.player_identifier);
console.log("game_board_state:", video.game_board_state);
console.log("bbbv_solved/bbbv:", video.bbbv_solved + "/" + video.bbbv);
console.log("ce:", video.ce);
console.log("row:", video.row);
console.log("mine_num:", video.mine_num);
console.log("rtime:", video.rtime);
console.log("rtime_ms:", video.rtime_ms);
console.log("is win:", video.is_completed);
console.log("STNB:", video.stnb);
console.log("start_time:", video.start_time);
console.log("end_time:", video.end_time);
console.log("path:", video.path);
console.log("etime:", video.etime);
console.log("op:", video.op);
console.log("cell0:", video.cell0);
console.log("pluck:", video.pluck);  // requires tract-onnx, unavailable in WASM
