import * as ms from "ms_toollib";

// wasm.greet();
const a = [[0, 0, 1, -1, 2, -1, 1, 0], [0, 0, 2, 2, 3, 1, 1, 0], [0, 1, 2, -1, 1, 0, 0, 0],
[0, 1, -1, 3, 2, 0, 0, 0], [2, 3, 3, -1, 2, 1, 1, 0], [-1, -1, 2, 2, 3, -1, 1, 0],
[2, 2, 1, 1, -1, 3, 2, 0], [0, 0, 0, 1, 2, -1, 1, 0]];
let bv = ms.cal_bbbv(JSON.stringify(a));
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
let poss = ms.cal_possibility_onboard(JSON.stringify(b), 10);
console.log(poss);

const board = ms.laymine_solvable(16, 30, 99, 0, 0, 10000)
console.log(board);

let m = ms.MinesweeperBoard.new(JSON.stringify(a));
m.step("lc", 2, 2)
m.step("lr", 2, 2)
console.log(m.get_game_board);
m.step_flow(JSON.stringify([["lc", 0, 0], ["lr", 0, 0]]))
console.log(m.get_game_board);


const request = new XMLHttpRequest()
request.onload = () => {
    let r = new Uint8Array(request.response);
    let video = ms.EvfVideo.new(r, "test.evf")
    video.parse_video()
    console.log(video.get_bbbv);
    console.log(video.get_player);
    video.analyse()
    for (var i = 0; i < 100; i++) {
        console.log(video.events_x(i), video.events_y(i), video.events_mouse(i));
    }
    console.log(video.get_is_fair);
    console.log(video.get_is_official);
    console.log(video.get_is_completed);
    console.log(video.is_valid());
} 
request.onerror = (e) => {
    console.log(555);
}
request.open('GET', "test.evf")
request.responseType = 'arraybuffer'
request.send()

console.log(ms.TimePeriod("Arbiter"));




