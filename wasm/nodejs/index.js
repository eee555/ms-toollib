// npm run
const { log } = require("console");
const ms = require("../pkg/ms_toollib.js")
const fs = require('fs');

let b = ms.laymine_solvable(16, 30, 99, 0, 0, 100, 381, 100000, 40)
console.log(b);

fs.readFile("jze.avf", (err, data) => {
    if (err) {
        console.error('Error reading file:', err);
        return;
    }
    // 将Buffer转换为Uint8Array
    const jze_video_data = new Uint8Array(data);
    let jze_video = ms.AvfVideo.new(jze_video_data, "jze.avf")
    jze_video.parse()
    jze_video.analyse()
    console.log(jze_video.get_cell3);
    jze_video.current_time = 999.0;
    console.log(jze_video.get_bbbv_s);
    console.log(jze_video.get_double_s);
});




