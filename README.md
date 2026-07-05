[English](./README.en.md) | 中文

# ms_toollib & EVF Format

[![ms_toollib](https://img.shields.io/badge/ms_toollib-v1.5.10-brightgreen.svg)](https://github.com/eee555/ms_toollib)
[![crates.io](https://img.shields.io/crates/v/ms_toollib)](https://crates.io/crates/ms_toollib)
[![npm](https://img.shields.io/npm/v/ms-toollib)](https://www.npmjs.com/package/ms-toollib)
[![PyPI](https://img.shields.io/pypi/v/ms-toollib)](https://pypi.org/project/ms-toollib/)

扫雷算法工具箱 — 跨平台、多语言绑定的扫雷核心算法库。

API 文档：[docs.rs/ms_toollib](https://docs.rs/ms_toollib)

---

## 总览 Architecture

```
base/          — 核心算法（纯 Rust）
  ├── c/       — C FFI 绑定（MSVC）
  ├── java/    — Java JNI 绑定（JDK ≥ 21）
  ├── wasm/    — WASM 绑定（wasm-pack → npm）
  └── python_package/ — Python 绑定（maturin → PyPI）
```

各绑定层将 `base/` 编译为对应语言的 native 库，零运行时开销。

---

## 快速上手 Quick Start

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
# 示例见 demos/c/
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

### Julia（通过 Python 绑定）

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

---

## 功能 Features

| 功能 | 说明 |
|------|------|
| **埋雷** | 无猜埋雷（筛选法 + 调整法） |
| **指标计算** | 3BV / ZiNi / ISL / OP |
| **概率计算** | 基于当前局面的逐格雷概率 |
| **状态机** | 逐步骤推衍局面变化 |
| **录像解析** | 支持 AVF / EVF / MVF / RMV 格式 |
| **局面识别** | 图片输入 → 识别局面（需 `rs` 或 `py` feature） |
| **录像分析** | 效率指标、pluck 检测、跳判检测 |

---

## 版本号

| 平台 | 最新版本 | 发布渠道 |
|------|----------|----------|
| Rust crate | 1.5.10 | [crates.io](https://crates.io/crates/ms_toollib) |
| Python | 1.5.13 | `pip install ms-toollib` |
| WASM (bundler) | 1.5.13 | `npm install ms-toollib` |
| WASM (Node.js) | 1.5.10-alpha | `npm install ms-toollib@alpha` |
| C | 0.2.0 | 源码编译 |
| C++ | 0.2.0 | 源码编译（Corrosion + CXX） |
| Java | 1.5.10 | 源码编译（JNI） |

### Python 支持平台

Python >= 3.8, <= 3.13:
- Windows: x86, x64
- Linux: x86, x86_64, aarch64
- macOS: x86_64, aarch64

---

## 构建依赖

| 目标 | 运行时 | 构建工具 |
|------|--------|----------|
| Rust | 无 | `rustup` + `cargo` |
| Python | 无 | `maturin`（pip 安装） |
| WASM | 无 | `wasm-pack`（cargo install） |
| C (Windows) | 无 | MSVC（VS 2022）+ `cargo` |
| Java | 无 | JDK ≥ 21（含 `jni.h`）+ MSVC + `cargo` |

所有平台都需 Rust 工具链编译核心库。Python / JS / WASM 有预编译包可直接 `pip` / `npm install`。

---

## 目录说明

```
├── base/              # 核心库（features: rs, py, js）
│   ├── src/
│   │   ├── algorithms/   # 求解、概率、OCR
│   │   ├── videos/       # 录像解析 + 状态机
│   │   └── ...
│   └── tests/
├── c/                 # C FFI（staticlib）
│   ├── include/ms_toollib/  # C 头文件
│   ├── src/lib.rs          # extern "C" 桥接
│   └── tests/
├── java/              # Java JNI 绑定
│   ├── src/main/java/ms_toollib/  # Java 类
│   └── src/main/c/jni_wrapper.c   # JNI 胶水
├── wasm/              # WASM 绑定（wasm-bindgen）
├── python_package/    # Python 绑定（pyo3 / maturin）
├── demos/             # 各语言示例
│   ├── c/
│   ├── java/
│   └── ...
└── test_files/        # 示例录像文件
```

---

## 开发

```bash
git clone https://github.com/eee555/ms_toollib.git
cd base

# 运行测试
cargo test --features rs

# 构建 C 绑定
cd ../c && cargo build --release
```

详细见 [CONTRIBUTING.md](./CONTRIBUTING.md)。

---

## 相关项目

- [Metasweeper](https://github.com/eee555/Metasweeper) — Win10/11 开源无猜扫雷
- [Saolei Website](https://github.com/eee555/saolei_website) — 开源扫雷排名网
- [益智扫雷](https://apps.apple.com/cn/app/益智扫雷/id6748243595?uo=4) — App Store 无猜扫雷

---

## License

MIT
