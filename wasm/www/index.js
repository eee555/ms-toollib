import * as wasm from "wasm-main";

// wasm.greet();
const a = [[0, 0, 1, -1, 2, -1, 1, 0], [0, 0, 2, 2, 3, 1, 1, 0], [0, 1, 2, -1, 1, 0, 0, 0],
[0, 1, -1, 3, 2, 0, 0, 0], [2, 3, 3, -1, 2, 1, 1, 0], [-1, -1, 2, 2, 3, -1, 1, 0],
[2, 2, 1, 1, -1, 3, 2, 0], [0, 0, 0, 1, 2, -1, 1, 0]];
let bv = wasm.cal3BV(JSON.stringify(a));
console.log(bv);

const b = [[1, 10, 10, 10, 10, 10, 10, 10, 10],
[1, 10, 10, 10, 10, 10, 10, 10, 10],
[2, 10, 10, 10, 10, 10, 10, 10, 10],
[10, 10, 10, 10, 10, 10, 10, 10, 10],
[10, 10, 10, 10, 10, 10, 10, 10, 10],
[10, 10, 10, 10, 10, 10, 10, 10, 10],
[10, 10, 10, 10, 10, 10, 10, 10, 10],
[10, 10, 10, 10, 10, 10, 10, 10, 10],
[10, 10, 10, 10, 10, 10, 10, 10, 10]]
let poss = wasm.cal_possibility_onboard(JSON.stringify(b), 10);
console.log(poss);

const board = wasm.laymine_solvable(16, 30, 99, 0, 0, 10000)
console.log(board);

let m = wasm.MinesweeperBoard.new(JSON.stringify(a));
m.step("lc", 2, 2)
m.step("lr", 2, 2)
console.log(m.get_game_board);
m.step_flow(JSON.stringify([["lc", 0, 0], ["lr", 0, 0]]))
console.log(m.get_game_board);

// let video = wasm.AvfVideo.new("jze.avf")
// video.parse_video()
// console.log(video.get_bbbv)



const request = new XMLHttpRequest()
request.onload = () => {
    let r = new Uint8Array(request.response);
    console.log(JSON.stringify(r));
}
request.onerror = (e) => {
    console.log(888);
}
request.open('GET', "jze.avf")
request.responseType = 'arraybuffer'
request.send()

// const reader = new FileReader()
// reader.onload = function () {
//     console.log(666);
// }
// reader.onerror = function (e) {
//     console.log(888);
// }
// reader.readAsArrayBuffer("jze.avf")

