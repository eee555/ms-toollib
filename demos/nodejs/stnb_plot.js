// 此脚本对比 stnb 的两种公式，并绘制曲线图。
// 公式1（当前激活）：stnb = c * bbbv_solved / t^1.7 * (bbbv_solved / total_bbbv)^0.5
// 公式2（被注释）：stnb = c * total_bbbv / etime^1.7 * (bbbv_solved / total_bbbv)^0.5
// 其中 etime = t / bbbv_solved * total_bbbv（预估总用时），c 为难度常数（高级=435）。
// 数据来自 ms_toollib/test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf

const ms = require("ms-toollib");
const fs = require("fs");
const { execSync } = require("child_process");

const video_file =
  "../../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf";
const data = fs.readFileSync(video_file);

let v = new ms.AvfVideo(data, video_file);
v.parse();
v.analyse();

const C = 435;
const total_bbbv = v.bbbv; // 127

// Collect stnb at each event
let points = [];
for (const e of v.events) {
  const t = e.time;
  if (t <= 0) continue;
  const bbbv_solved = e.key_dynamic_params.bbbv_solved;
  if (bbbv_solved === 0) continue;

  // stnb1: active formula — uses current_time
  const stnb1 = C * bbbv_solved / Math.pow(t, 1.7) * Math.pow(bbbv_solved / total_bbbv, 0.5);

  // stnb2: commented formula — uses etime = current_time / bbbv_solved * total_bbbv
  const etime = t / bbbv_solved * total_bbbv;
  const stnb2 = C * total_bbbv / Math.pow(etime, 1.7) * Math.pow(bbbv_solved / total_bbbv, 0.5);

  // Also get v.stnb (which uses rtime=49.25 = constant since state is Win)
  v.current_time = t;
  const stnb_win_state = v.stnb;

  points.push({ t, bbbv_solved, stnb1, stnb2, stnb_win_state });
}

// Print CSV
console.log("t,bbbv_solved,stnb1,stnb2,stnb_win_state");
for (const p of points) {
  console.log(`${p.t.toFixed(3)},${p.bbbv_solved},${p.stnb1.toFixed(6)},${p.stnb2.toFixed(6)},${p.stnb_win_state.toFixed(6)}`);
}

// Generate python plot
const desktop = require("os").homedir() + "\\Desktop";
const csvRows = points.map(p => `${p.t},${p.bbbv_solved},${p.stnb1},${p.stnb2},${p.stnb_win_state}`).join("\\n");
const plotScript = `
import matplotlib.pyplot as plt
import csv, io

data = """${csvRows}"""
reader = csv.DictReader(io.StringIO(data))
ts, stnb1s, stnb2s, stnb_win = [], [], [], []
for row in reader:
    ts.append(float(row["t"]))
    stnb1s.append(float(row["stnb1"]))
    stnb2s.append(float(row["stnb2"]))
    stnb_win.append(float(row["stnb_win_state"]))

plt.figure(figsize=(14, 10))

plt.subplot(2,2,1)
plt.plot(ts, stnb1s, label="stnb1 (active): c * bbbv_solved / t^1.7 * (bbbv_solved / total)^0.5")
plt.plot(ts, stnb2s, label="stnb2 (commented): c * total / etime^1.7 * (bbbv_solved / total)^0.5")
plt.xlabel("time (s)")
plt.ylabel("stnb")
plt.legend()
plt.grid()

plt.subplot(2,2,2)
plt.plot(ts, stnb1s, label="stnb1 (active)")
plt.plot(ts, stnb2s, label="stnb2 (commented)")
plt.xlabel("time (s)")
plt.ylabel("stnb")
plt.yscale("log")
plt.legend()
plt.grid()

plt.subplot(2,2,3)
plt.plot(ts, stnb_win, label="stnb via v.stnb (Win state, rtime=const)")
plt.xlabel("time (s)")
plt.ylabel("stnb")
plt.legend()
plt.grid()

plt.subplot(2,2,4)
plt.plot(ts, stnb1s, label="stnb1 (active)")
plt.plot(ts, stnb2s, label="stnb2 (commented)")
plt.plot(ts, stnb_win, "--", label="stnb via v.stnb (Win state)")
plt.xlabel("time (s)")
plt.ylabel("stnb")
plt.legend()
plt.grid()

plt.tight_layout()
plt.savefig(r"${desktop}\\stnb_comparison.png", dpi=150)
print("Saved to", r"${desktop}\\stnb_comparison.png")
`;
const scriptPath = require("os").tmpdir() + "\\stnb_plot.py";
fs.writeFileSync(scriptPath, plotScript);
console.log("\nGenerating plot...");
try {
  execSync(`python "${scriptPath}"`, { stdio: "inherit", timeout: 30000 });
} catch (e) {
  console.log("Python script failed, but CSV data was printed above.");
  console.log("You can manually run:", scriptPath);
}
