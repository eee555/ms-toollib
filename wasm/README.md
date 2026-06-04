# ms-toollib

[![npm](https://img.shields.io/npm/v/ms-toollib.svg)](https://www.npmjs.com/package/ms-toollib)
[![MIT](https://img.shields.io/badge/license-MIT-brightgreen.svg)](https://github.com/eee555/ms_toollib/blob/main/LICENSE)
[![docs](https://img.shields.io/badge/docs-ms_toollib-blue.svg)](https://docs.rs/ms_toollib/latest/ms_toollib)

A minesweeper algorithm toolbox with a native Rust core, compiled to WebAssembly.

Works in browsers (via bundler) and Node.js. No Python runtime required.

---

## Who is this for?

- **Minesweeper enthusiasts** — analyze your replays, compute difficulty metrics (3BV, ZiNi), compare performance
- **Trainer / bot developers** — generate no-guess boards on demand, compute probabilities, drive a game state machine
- **Minesweeper website builders** — embed board generation, solving, and replay parsing entirely on the client side

---

## What problems does it solve?

| Problem | Solution |
|---|---|
| I need to generate minesweeper boards that are guaranteed solvable (no-guess) | `laymine_solvable()` / other laymine variants — filter & adjustment methods |
| I want to rate a board's difficulty objectively | `cal_bbbv()` / `Board.bbbv` → 3BV, ZiNi, openings, islands |
| I need to know the exact mine probability per cell for advanced play | `cal_probability_onboard()` — block decomposition + enumeration |
| I want to check if a given board is solvable without guessing | `is_solvable()` — detects guess-while-needless positions |
| I'm building an interactive minesweeper UI and don't want to write game logic | `MinesweeperBoard` class — handles clicks, flags, chords, win/loss detection |
| I downloaded a replay (.avf / .evf / .mvf / .rmf) and want to extract metrics | Parse with e.g. `AvfVideo`, then read `.bbbv`, `.zini`, `.rtime`, `.iop`, etc. |
| I want to embed all of this in a web app without a backend | Package is pure WebAssembly — runs entirely in the browser |

---

## Installation

```bash
npm install ms-toollib
```

---

## Quick Start

```javascript
import init, {
  cal_bbbv, cal_zini, laymine_solvable, cal_probability_onboard,
  Board, AvfVideo, MinesweeperBoard
} from "ms-toollib";

await init();

// ── Generate a no-guess Beginner board ──
const board = laymine_solvable(9, 9, 10, 0, 0, 8000);
// board[0] = 2D board, board[1] = success flag
const grid = board[0];
console.log("3BV:", cal_bbbv(grid));

// ── Board metrics via the Board class ──
const b = new Board(grid);
console.log("Openings:", b.op, "Islands:", b.isl);

// ── Mine probability per cell ──
const prob = cal_probability_onboard(grid, 10);
// prob[0] = probability grid, prob[1] = mine count range [min, max, total]

// ── Interactive game state machine ──
const ms = new MinesweeperBoard(grid);
ms.step("lc", 4, 4);   // left-click (row, col)
ms.step("lr", 4, 4);   // left-release (row, col)
console.log("Flags placed:", ms.flag, "Cells opened:", ms.lce);

// ── Parse a replay ──
// (requires fetching the file as Uint8Array)
const response = await fetch("replay.avf");
const data = new Uint8Array(await response.arrayBuffer());
const video = new AvfVideo(data, "replay.avf");
video.parse();
video.analyse();
console.log("3BV:", video.bbbv);
console.log("ZiNi:", video.zini);
console.log("Time (ms):", video.rtime_ms);
console.log("3BV/s:", video.bbbv_s);
```

---

## Public API

**Functions (8):**
`cal_bbbv`, `cal_op`, `cal_probability_onboard`, `is_solvable`, `laymine`, `laymine_op`, `laymine_solvable`, `valid_time_period`

**Classes (16):**
`AvfVideo`, `Board`, `BoardEvent`, `CursorPos`, `Event`, `EvfVideo`, `GameBoard`, `GameStateEvent`, `IndexEvent`, `KeyDynamicParams`, `MinesweeperBoard`, `MouseEvent`, `MvfVideo`, `RmvVideo`, `TimePeriod`, `VideoActionStateRecorder`

---

## Supported Environments

| Environment | Build Target |
|---|---|
| Browser (webpack, vite, etc.) | `wasm-pack build` (bundler) |
| Node.js | `wasm-pack build --target nodejs` |

---

## Build from Source

```bash
# Requires wasm-pack: https://rustwasm.github.io/wasm-pack/
wasm-pack build       # browser bundler
wasm-pack build --target nodejs   # Node.js
```

---

## Source & Issues

- GitHub: <https://github.com/eee555/ms_toollib>
- Documentation: <https://docs.rs/ms_toollib/latest/ms_toollib>
- npm: <https://www.npmjs.com/package/ms-toollib>
