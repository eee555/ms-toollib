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

### pluck 相关指标

通过 `analyse_for_features(["pluck"])` 一次计算以下所有指标。它们沿当前录像累计，
与现有 pluck 共用首个有效左击豁免，以及基于操作前局面的单格／双击联合安全概率。

设第 `i` 次操作后的累计安全权重为 `L_i`，`L_0 = 1`，新增解决量为 `ΔB_i`，
自上次事件至本次操作的耗时为 `Δt_i`：

| 参数 | 定义 |
|------|------|
| `pluck` | `-log10(L_i)`，保留原有接口和失败标记 |
| `luck` | `10^(-pluck)` |
| `expected_bbbv_solved_per_attempt` | `Σ L_i × ΔB_i` |
| `expected_bbbv_solved_per_win` | 上述累计解决量除以当前 `L_i` |
| `expected_time_per_attempt` | `Σ L_(i-1) × Δt_i`，单位为秒 |
| `expected_time_per_win` | 上述累计时间除以当前 `L_i`，单位为秒 |

`per_attempt` 表示单次尝试的期望量，`per_win` 表示每次获胜的期望累计量。

时间使用操作前的 Luck，BV 贡献使用操作后的 Luck；移动、等待和不推进 BV 的猜雷也计入相应时间／风险。
一次解决多个 BV 时按新增数量累计，预标雷等游戏计时开始前的时间不计入。
触雷沿用原 pluck 约定：`luck = 0`，两个 `per_win` 指标为正无穷；已累计的加权工作量和耗时保留。
归一化结果直接累计，避免先求极小 Luck 再除法导致的数值下溢问题。

Python、JavaScript 使用同名只读属性；Rust 使用 `get_<参数名>()`。
录像播放读取事件快照，并在事件之间按当前 Luck 继续累计时间。
尚未执行 pluck 特征分析时返回 NaN。游戏结束后首次读取任一指标会按需计算并缓存全部结果；
重开或重新 `analyse()` 会使缓存失效。`KeyDynamicParams` 同时提供事件时刻的这些数值。

---

## 版本号

| 平台 | 最新版本 | 发布渠道 |
|------|----------|----------|
| Rust crate | 1.5.16 | [crates.io](https://crates.io/crates/ms_toollib) |
| Python | 1.6.0 | `pip install ms-toollib` |
| WASM (bundler) | 1.6.0 | `npm install ms-toollib` |
| WASM (Node.js) | 1.5.16-alpha | `npm install ms-toollib@alpha` |
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

### python-rust联调步骤
假设有个py项目在C:\path\main.py，需要`import ms_toollib`
```
cd python_package
cargo build --release
cp .\target\release\ms_toollib.dll C:\path\ms_toollib.pyd
```


## 相关项目

- [Metasweeper](https://github.com/eee555/Metasweeper) — Win10/11 开源无猜扫雷
- [Saolei Website](https://github.com/eee555/saolei_website) — 开源扫雷排名网
- [益智扫雷](https://apps.apple.com/cn/app/益智扫雷/id6748243595?uo=4) — App Store 无猜扫雷

---

## License

MIT
