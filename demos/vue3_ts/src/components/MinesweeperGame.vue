<template>
  <div>
    <h3>BaseVideo 扫雷小游戏</h3>
    <div style="margin-bottom: 8px">
      <button @click="newGame">新游戏</button>
      <span style="margin-left: 12px">雷数: {{ mineCount }}</span>
      <span style="margin-left: 12px">状态: {{ stateText }}</span>
    </div>
    <div
      :style="{ display: 'inline-grid', gridTemplateColumns: `repeat(${cols}, ${cellPx}px)`, gap: '0px' }"
      @contextmenu.prevent
    >
      <div
        v-for="cell in cells"
        :key="cell.key"
        :style="cell.style"
        @mousedown.left.prevent="onDown('l', cell.c, cell.r)"
        @mouseup.left.prevent="onUp('l', cell.c, cell.r)"
        @mousedown.right.prevent="onDown('r', cell.c, cell.r)"
        @mouseup.right.prevent="onUp('r', cell.c, cell.r)"
      >{{ cell.text }}</div>
    </div>
    <p style="font-size: 12px; color: #666; margin-top: 8px">
      左键打开 / 右键标旗 / 双键同时按住开 chord
    </p>
    <div v-if="stats" style="font-size: 12px; line-height: 1.8; margin-top: 8px; background: #f5f5f5; padding: 8px 12px; border-radius: 4px;">
      <div><b>time:</b> {{ fmt(stats.time) }}s | <b>op:</b> {{ stats.op }} | <b>path:</b> {{ fmt(stats.path) }}</div>
      <div><b>left:</b> {{ stats.left }} ({{ fmt(stats.left_s) }}/s) | <b>right:</b> {{ stats.right }} ({{ fmt(stats.right_s) }}/s) | <b>double:</b> {{ stats.double }} ({{ fmt(stats.double_s) }}/s)</div>
      <div><b>flag:</b> {{ stats.flag }} ({{ fmt(stats.flag_s) }}/s) | <b>cl:</b> {{ stats.cl }} ({{ fmt(stats.cl_s) }}/s)</div>
      <div><b>3BV:</b> {{ stats.bbbv_solved }}/{{ stats.bbbv }} | <b>IOE:</b> {{ fmt(stats.ioe) }} | <b>thrp:</b> {{ fmt(stats.thrp) }}</div>
      <div><b>corr:</b> {{ fmt(stats.corr) }} | <b>STNB:</b> {{ fmt(stats.stnb) }} | <b>RQP:</b> {{ fmt(stats.rqp) }} | <b>pluck:</b> {{ fmt(stats.pluck) }}</div>
      <div><b>rtime:</b> {{ fmt(stats.rtime) }}s | <b>3BV/s:</b> {{ fmt(stats.bbbv_s) }} | <b>CE/s:</b> {{ fmt(stats.ce_s) }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";

const ROWS = 9;
const COLS = 9;
const MINES = 10;
const CELL_PX = 28;

let ms: any = null;
let video: any = null;

const board = ref<number[][]>([]);
const cols = COLS;
const cellPx = CELL_PX;
const mineCount = MINES;
const stats = ref<any>(null);
let timer: ReturnType<typeof setInterval> | null = null;

function fmt(v: any) {
  if (v === "--") return "--";
  if (typeof v !== "number") return "0";
  return v % 1 === 0 ? String(v) : v.toFixed(4);
}

const stateText = computed(() => {
  if (!video) return "";
  const s = video.game_board_state;
  if (s === 1) return "等待";
  if (s === 2) return "游戏中";
  if (s === 3) return "胜利!";
  if (s === 4) return "失败";
  return "";
});

const NUM_COLORS: Record<number, string> = {
  1: "#0000ff", 2: "#008000", 3: "#ff0000", 4: "#000080",
  5: "#800000", 6: "#008080", 7: "#000000", 8: "#808080",
};

const cells = computed(() => {
  const result = [];
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const v = board.value[r]?.[c];
      const opened = v !== undefined && v !== 10 && v !== 11 && v !== 12 && v !== 14 && v !== 15;
      let text = "";
      let color = "#000";
      let bg = "#bbb";
      if (v === 11 || v === 12 || v === 14 || v === 15) {
        text = "\u2691";
      } else if (v === 16 || v === -1) {
        text = "\u2620";
        bg = "#fdd";
      } else if (opened) {
        bg = "#ddd";
        if (v !== undefined && v > 0) {
          text = String(v);
          color = NUM_COLORS[v] ?? "#000";
        }
      }
      result.push({
        key: `${r}-${c}`,
        r, c,
        text,
        style: {
          width: CELL_PX + "px",
          height: CELL_PX + "px",
          lineHeight: CELL_PX + "px",
          textAlign: "center",
          fontSize: "14px",
          fontWeight: "bold",
          cursor: "pointer",
          userSelect: "none",
          boxSizing: "border-box",
          border: "1px solid #999",
          backgroundColor: bg,
          color,
        },
      });
    }
  }
  return result;
});

function syncBoard() {
  board.value = video.game_board;
}

function newGame() {
  const b: number[][] = ms.laymine_solvable(ROWS, COLS, MINES, 0, 0, 10000)[0];
  video = new ms.BaseVideo(b, CELL_PX);
  syncBoard();
}

function sendStep(e: string, c: number, r: number) {
  video.step(e, r * CELL_PX, c * CELL_PX);
}

function finish() {
  if (video.game_board_state === 4) video.loss_then_open_all_mine();
  if (video.game_board_state === 3) video.win_then_flag_all_mine();
  syncBoard();
}

function onDown(btn: string, c: number, r: number) {
  if (!video) return;
  const s = video.game_board_state;
  if (s === 3 || s === 4) return;
  sendStep(btn + "c", c, r);
  finish();
}

function onUp(btn: string, c: number, r: number) {
  if (!video) return;
  const s = video.game_board_state;
  if (s === 3 || s === 4) return;
  sendStep(btn + "r", c, r);
  finish();
}

function refreshStats() {
  if (!video) { stats.value = null; return; }
  const s = video.game_board_state;
  if (s !== 2 && s !== 3 && s !== 4) { stats.value = null; return; }
  const over = s === 3 || s === 4;
  stats.value = {
    time: video.time,
    op: video.op,
    path: video.path,
    bbbv: video.bbbv,
    left: video.left,
    left_s: video.left_s,
    right: video.right,
    right_s: video.right_s,
    double: video.double,
    double_s: video.double_s,
    flag: video.flag,
    flag_s: video.flag_s,
    cl: video.cl,
    cl_s: video.cl_s,
    rtime: over ? video.rtime : "--",
    bbbv_solved: over ? video.bbbv_solved : "--",
    bbbv_s: over ? video.bbbv_s : "--",
    ce: over ? video.ce : "--",
    ce_s: over ? video.ce_s : "--",
    corr: over ? video.corr : "--",
    thrp: over ? video.thrp : "--",
    ioe: over ? video.ioe : "--",
    stnb: over ? video.stnb : "--",
    rqp: over ? video.rqp : "--",
    pluck: over ? video.pluck : "--",
  };
}

onMounted(async () => {
  ms = await import("ms-toollib");
  const b: number[][] = ms.laymine_solvable(ROWS, COLS, MINES, 0, 0, 10000)[0];
  video = new ms.BaseVideo(b, CELL_PX);
  syncBoard();
  refreshStats();
  timer = setInterval(refreshStats, 100);
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
});
</script>
