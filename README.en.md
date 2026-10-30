# ms_toollib & EVF Format

[![ms_toollib](https://img.shields.io/badge/ms_toollib-v1.5.10-brightgreen.svg)](https://github.com/eee555/ms_toollib)
[![crates.io](https://img.shields.io/crates/v/ms_toollib)](https://crates.io/crates/ms_toollib)
[![npm](https://img.shields.io/npm/v/ms-toollib)](https://www.npmjs.com/package/ms-toollib)
[![PyPI](https://img.shields.io/pypi/v/ms-toollib)](https://pypi.org/project/ms-toollib/)

Minesweeper algorithm toolkit — cross-platform, multi-language bindings.

API docs: [docs.rs/ms_toollib](https://docs.rs/ms_toollib)

---

## Architecture

```
base/              — Core algorithms (pure Rust)
  ├── c/           — C FFI (MSVC)
  ├── java/        — Java JNI (JDK ≥ 21)
  ├── wasm/        — WASM bindings (wasm-pack → npm)
  └── python_package/ — Python bindings (maturin → PyPI)
```

Each binding compiles `base/` into a native library for the target language, zero runtime overhead.

---

## Quick Start

### Rust

```toml
[dependencies]
ms_toollib = "1.5"
```

```rust
use ms_toollib::{laymine, cal_bbbv};

let board = laymine(16, 30, 99, 0, 0);
println!("3BV: {}", cal_bbbv(&board));
```

### Python

```bash
pip install ms-toollib
```

```python
from ms_toollib import laymine, cal_bbbv

board = laymine(16, 30, 99)
print("3BV:", cal_bbbv(board))
```

### JavaScript / TypeScript (bundler)

```bash
npm install ms-toollib
```

```js
import { laymine, calBBBV } from "ms-toollib";

const board = laymine(16, 30, 99, 0, 0);
console.log("3BV:", calBBBV(board));
```

### Node.js

```bash
npm install ms-toollib@alpha
```

### C (Windows, MSVC)

```bash
cd c && cargo build --release
# See demos/c/
```

### C++ (Linux, CMake + Corrosion)

```bash
cd c++ && cmake -B build . && make -C build -j4
```

```cpp
#include "cxxbridge_code/src/lib.rs.h"

int main() {
    rust::Box<AvfVideo> v = new_AvfVideo("replay.avf");
    v->parse();
    v->analyse();
    std::cout << "player: " << v->get_player() << std::endl;
    std::cout << "3BV: " << v->get_bbbv() << std::endl;
}
```

### Julia (via Python bindings)

```bash
pip install ms-toollib
```

```julia
using PyCall
ms = @pyimport ms_toollib

board = ms.laymine(16, 30, 99)
println("3BV: ", ms.cal_bbbv(board))

v = ms.AvfVideo("replay.avf")
v.parse()
v.analyse()
println(v.player_identifier)
```

```bash
julia demos/julia/callms_toollib.jl
```

### Java (JDK ≥ 21)

```bash
cd java && build.bat "C:\Path\to\jdk-21"
```

```java
import ms_toollib.*;

var board = MsToollib.laymine(16, 30, 99, 0, 0);
System.out.println("3BV: " + MsToollib.cal3BV(board));

try (var video = new EvfVideo("replay.evf")) {
    video.parse();
    var data = video.getData();
    System.out.println(data.getPlayer() + " " + data.getRtime());
}
```

---

## Versions

| Platform | Latest | Channel |
|----------|--------|---------|
| Rust crate | 1.5.16 | [crates.io](https://crates.io/crates/ms_toollib) |
| Python | 1.5.16 | `pip install ms-toollib` |
| WASM (bundler) | 1.5.16 | `npm install ms-toollib` |
| WASM (Node.js) | 1.5.16-alpha | `npm install ms-toollib@alpha` |
| C | 0.2.0 | source build |
| C++ | 0.2.0 | source build (Corrosion + CXX) |
| Java | 1.5.10 | source build (JNI) |

### Python supported platforms

Python >= 3.8, <= 3.13:
- Windows: x86, x64
- Linux: x86, x86_64, aarch64
- macOS: x86_64, aarch64

---

## Features

| Feature | Description |
|---------|-------------|
| **laymine** | No-guess mine placement (filter + adjust) |
| **Metrics** | 3BV, ZiNi, ISL, OP |
| **Probability** | Per-cell mine probability on any game state |
| **State machine** | Step-by-step board simulation |
| **Video parsing** | AVF, EVF, MVF, RMV replay format support |
| **OCR** | Image → board state (`rs` or `py` feature) |
| **Analysis** | Efficiency, pluck, jump detection |

### Effective-click path metrics

`ce = lce + rce + dce`. An effective click is a mouse event that increases one of these counters:
opening a safe cell, flagging a particular mine correctly for the first time, or a useful chord
(including a chord that hits a mine). Moves, ineffective clicks, wrong flags, unflags, repeated flags
on the same mine, and automatic recursive actions that do not increase ce are excluded.
Preflags that contribute to this game's ce are included. Cancelling all preflags back to Ready or
resetting the game clears both metrics.

Let effective-click pixel coordinates be `p_i = (x_i, y_i)`, the cell size be `s` pixels, and
cell coordinates be `q_i = (floor(x_i / s), floor(y_i / s))`:

- `c_path = (16 / s) × Σ ‖p_i − p_(i−1)‖₂`: cumulative straight-line click distance, using the same scale as `path`.
- `e_path = 16 × Σ ‖q_i − q_(i−1)‖₂`: cumulative Euclidean distance between cell coordinates, normalized to 16 pixels per cell.
  Clicks in the same cell contribute 0; orthogonally adjacent cells contribute 16 and diagonally adjacent cells contribute `16√2`.

Both metrics are 0 before the second effective click and retain floating-point precision.
Live games return the latest cumulative value; replay playback returns the value at the current event.
Rust exposes `get_c_path()` / `get_e_path()`; Python and JavaScript expose read-only `c_path` / `e_path` properties.

---

## Build Dependencies

| Target | Runtime | Build tooling |
|--------|---------|---------------|
| Rust | none | `rustup` + `cargo` |
| Python | none | `maturin` (pip) |
| WASM | none | `wasm-pack` (cargo install) |
| C (Windows) | none | MSVC (VS 2022) + `cargo` |
| Java | none | JDK ≥ 21 (with `jni.h`) + MSVC + `cargo` |

Pre-built wheels are available for Python, JS, and WASM.

---

## Layout

```
├── base/              # Core library (features: rs, py, js)
│   ├── src/algorithms/   # Solving, probability, OCR
│   ├── src/videos/       # Replay parsers + state machine
│   └── tests/
├── c/                 # C FFI (staticlib)
│   ├── include/ms_toollib/  # Public C headers
│   ├── src/lib.rs          # extern "C" bridge
│   └── tests/
├── c++/               # C++ (CMake, Linux)
├── java/              # Java JNI
│   ├── src/main/java/ms_toollib/  # Java classes
│   └── src/main/c/jni_wrapper.c   # JNI glue
├── wasm/              # WASM (wasm-bindgen)
├── python_package/    # Python (pyo3 / maturin)
├── demos/             # Examples per language
└── test_files/        # Sample replay files
```

---

## Development

```bash
git clone https://github.com/eee555/ms_toollib.git
cd base

cargo test --features rs

cd ../c && cargo build --release
```

See [CONTRIBUTING.md](./CONTRIBUTING.md).

---

## Projects Using ms_toollib

- [Metasweeper](https://github.com/eee555/Metasweeper) — No-guess minesweeper for Windows
- [Saolei Website](https://github.com/eee555/saolei_website) — Open-source leaderboard site
- [Puzzle Minesweeper](https://apps.apple.com/cn/app/益智扫雷/id6748243595?uo=4) — No-guess minesweeper on App Store

---

## License

MIT
