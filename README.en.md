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
| Rust crate | 1.5.10 | [crates.io](https://crates.io/crates/ms_toollib) |
| Python | 1.5.11 | `pip install ms-toollib` |
| WASM (bundler) | 1.5.11 | `npm install ms-toollib` |
| WASM (Node.js) | 1.5.10-alpha | `npm install ms-toollib@alpha` |
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
