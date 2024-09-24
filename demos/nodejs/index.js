// 测试步骤
// 在ms_toollib\wasm下wasm-pack build --target nodejs
// 在ms_toollib\demos\nodejs下npm install
// npm run test

const ms = require('ms-toollib');
const fs = require('fs');


const a = [[0, 0, 1, -1, 2, -1, 1, 0], [0, 0, 2, 2, 3, 1, 1, 0], [0, 1, 2, -1, 1, 0, 0, 0],
[0, 1, -1, 3, 2, 0, 0, 0], [2, 3, 3, -1, 2, 1, 1, 0], [-1, -1, 2, 2, 3, -1, 1, 0],
[2, 2, 1, 1, -1, 3, 2, 0], [0, 0, 0, 1, 2, -1, 1, 0]];
let bv = ms.cal_bbbv(JSON.stringify(a));  //计算3BV
console.log(bv);

const video_file = '../../base/aaa.avf';
const data = fs.readFileSync(video_file)


let v = ms.AvfVideo.new(data, video_file)
v.parse_video();
v.analyse();
console.log(v.get_bbbv);
// 时间切到10.0秒
v.current_time = 10.0;
console.log("此时的x坐标，单位为像素", v.get_x_y.x);
console.log("此时的y坐标，单位为像素", v.get_x_y.y);

const time_period = ms.valid_time_period("Arbiter")
var newDate = new Date();
console.log(time_period.start_time);
newDate.setTime(time_period.end_time * 1000);
console.log("Arbiter video valid time:", newDate.toDateString());



