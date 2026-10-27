# ms_toollib

[![pypi](https://img.shields.io/pypi/v/ms_toollib.svg)](https://pypi.org/project/ms_toollib/)
[![MIT](https://img.shields.io/badge/license-MIT-brightgreen.svg)](https://github.com/eee555/ms_toollib/blob/main/LICENSE)
[![docs](https://img.shields.io/badge/docs-ms_toollib-blue.svg)](https://docs.rs/ms_toollib/latest/ms_toollib)

A cross-platform minesweeper algorithm toolbox with a native Rust core.

Supports Python 3.7–3.13 on Windows, Linux, and macOS.

---

## Installation

```bash
pip install ms_toollib
```

No additional dependencies required.

---

## Features

| Category | Description |
|---|---|
| **Mine Laying** | Random mine placement (first-click safe, Win7-style), no-guess mine generation (filter & adjustment methods) |
| **Board Metrics** | 3BV, 3BV/s, openings (Op), islands (Isl), ZiNi (Greedy/Human/Random) |
| **Probabilities** | Per-cell mine probability with block decomposition and enumeration |
| **Deduction Solver** | 3-tier solver: direct subset, subtract-and-compare, full enumeration (≤55 cells) |
| **Solvability Check** | Determine if a board is no-guess, find unsolvable structures |
| **Game State Machine** | `MinesweeperBoard` / `SafeMinesweeperBoard` that accepts mouse clicks and tracks full game state |
| **Replay Parsing** | Read AVF, EVF, MVF, RMV formats; save/write EVF (v0–v4) and EVFS containers |
| **Optical Recognition** | Recognize board state from an RGBA image via ONNX neural network (Sobel, Hough, DP) |

---

## Quick Start

```python
import ms_toollib as ms

# Generate a no-guess Beginner board
board = ms.laymine_solvable(9, 9, 10)
print(ms.cal_bbbv(board))          # 3BV
print(ms.cal_zini(board, 1))       # Greedy ZiNi

# Per-cell mine probability
prob = ms.cal_probability_onboard(board, 10)

# Parse a replay
video = ms.AvfVideo("replay.avf")
video.parse()
video.analyse()
print(video.bbbv)
```

See the [full documentation](https://docs.rs/ms_toollib/latest/ms_toollib) for detailed API references.

---

## Public API

**Functions (30+):**
`cal_all_solution`, `cal_bbbv`, `cal_board_numbers`, `cal_hzini`, `cal_op`, `cal_probability`, `cal_probability_onboard`, `cal_rzini`, `cal_zini`, `get_all_not_and_is_mine_on_board`, `is_able_to_solve`, `is_guess_while_needless`, `is_solvable`, `laymine`, `laymine_op`, `laymine_solvable`, `laymine_solvable_adjust`, `laymine_solvable_thread`, `mark_board`, `obr_board`, `refresh_board`, `refresh_matrix`, `refresh_matrixs`, `refresh_matrixses`, `sample_bbbvs_exp`, `solve_direct`, `solve_enumerate`, `solve_minus`, `unsolvable_structure`, `valid_time_period`

**Classes (15):**
`AvfVideo`, `BaseVideo`, `Board`, `EvfVideo`, `Evfs`, `EvfsCell`, `GameBoard`, `KeyDynamicParams`, `MinesweeperBoard`, `MvfVideo`, `RmvVideo`, `SafeBoard`, `SafeBoardRow`, `SafeMinesweeperBoard`, `VideoActionStateRecorder`

---

## Supported Platforms

| OS | Architectures |
|---|---|
| Windows | x86, x64 |
| Linux | x86, x86_64, aarch64, armv7, ppc64le, s390x |
| macOS | x86_64, aarch64 |

---

## Source & Issues

### Generate Python Type Stubs

The source-based generator requires Python 3.10+ and two development-only packages:

```bash
pip install "tree-sitter>=0.25,<0.27" "tree-sitter-rust>=0.24,<0.25"
python python_package/generate_pyi.py --compare python_package/ms_toollib.pyi
```

Run from the repository root. By default this writes
`python_package/ms_toollib.generated.pyi`, leaving the existing stub unchanged.
Use `--output python_package/ms_toollib.pyi` to explicitly replace the package stub.
Use `--check` to verify the output is current without writing it, and `--diff`
with `--compare` for a full text diff in addition to the semantic API report.

The generator parses registered PyO3 functions and classes, renamed bindings,
constructors, properties, signatures/defaults, documentation and nested Rust
types. It expands the single-identifier `generate_video!` template used here.
Unregistered event classes referenced by signatures are emitted as private
support types. Getter-only properties remain read-only; setter-only properties
use writable attribute annotations. Complex Rust default expressions use `...`.

This is a source parser, not a Rust compiler: general macro expansion, conditional
compilation, externally defined types and arbitrary PyO3 extensions are not
supported. Unknown Rust types and unsupported top-level macros cause an error.
`PyAny` remains `Any`; types selected dynamically inside Rust function bodies
cannot be recovered from signatures. API comparison normalizes `typing.List`
and `typing.Tuple` aliases, but is not a complete type-equivalence checker.

Regression tests: `python -m unittest discover -s python_package -p test_generate_pyi.py`.

- GitHub: <https://github.com/eee555/ms_toollib>
- Documentation: <https://docs.rs/ms_toollib/latest/ms_toollib>
- PyPI: <https://pypi.org/project/ms_toollib/>
