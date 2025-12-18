// 测试步骤
// 在ms_toollib\wasm下wasm-pack build --target nodejs
// 在ms_toollib\demos\nodejs下npm install
// npm run test

const ms = require('ms-toollib');
const fs = require('fs');
const { assert } = require('console');

const video_file = '../../test_files/Cus_8x11_7mines_5.42_3BV=8_3BVs=1.47_Wang Jianing G15208.avf';
const data = fs.readFileSync(video_file)

// 使用二进制数据和文件名初始化
let v = new ms.AvfVideo(data, video_file)
v.parse();
v.analyse();
assert(v.bbbv == 8);

// 时间切到10.0秒
v.current_time = 10.0;

