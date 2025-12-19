// npm test运行此文件
const { log } = require("console");
const ms = require("../pkg/ms_toollib.js");
const fs = require("fs/promises");

async function main() {
  let b = ms.laymine_solvable(16, 30, 99, 0, 0, 100000);
  console.log(`该无猜局面的3BV为：${ms.cal_bbbv(b[0])}`);
  const data = await fs.readFile(
    "../../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf"
  );

  // Buffer -> Uint8Array（零拷贝视图）
  const video_data = new Uint8Array(data);

  const video = new ms.AvfVideo(
    video_data,
    "HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf"
  );

  video.parse();
  video.analyse();

  console.log(video.cell3);

  video.current_time = 999.0;

  console.log(video.bbbv_s);
  console.log(video.double_s);
}

main();
