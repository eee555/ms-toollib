# MINESWEEPER-BOARD Specification v0.1

## 1. Introduction

MINESWEEPER-BOARD（简称 **Board**）是一种用于存储静态扫雷局面的纯文本文件格式。

设计目标：

- 人类可读
- 易于程序解析
- 易于复制、粘贴与分享
- 支持版本演进
- 支持多种渲染方式（Render）

文件扩展名：

```
.board
```

推荐文本编码：

```
UTF-8
```

---

## 2. File Structure

一个 Board 文件由以下几个部分组成：

```
Header
Metadata
real Board
view Board
```

其中：

- **Header**：标识文件格式及版本（必需）
- **Metadata**：描述局面信息（可选）
- **real Board**：保存真实局面
- **view Board**：保存玩家当前看到的局面

real Board 与 view Board 至少存在一个，两者也可以同时存在。

---

## 3. Header

文件第一行为格式声明：

```
# MINESWEEPER-BOARD v0.1
```

第二行为 Render：

```
# Render: ascii
```

或

```
# Render: emoji
```

实现必须忽略未知的注释行（以 `#` 开头）。

---

## 4. Metadata

Metadata 采用 `key: value` 形式，每个字段占一行。value不可包含换行。

示例：

```
author: eee555
comment: Hello World

rows: 16
columns: 30
mines: 99

game_mode: classic
```

当前定义字段：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| author | String | 否 | 作者 |
| comment | String | 否 | 注释 |
| rows | Integer | 是 | 行数 |
| columns | Integer | 是 | 列数 |
| mines | Integer | 是 | 雷数 |
| game_mode | String | 是 | 游戏模式 |

解析逻辑是每行第一个":"用于分割，后续value中可以出现":"。未知字段应被忽略，不应导致解析失败。

---

## 5. real Board

real Board 保存真实局面。

格式：

```
[real]
```

之后紧跟 `rows` 行。

每行必须包含 `columns` 个字符。

### Character Definition

| 字符 | 含义 |
|------|------|
| * | 地雷 |
| 0~8 | 周围地雷数量 |

示例：

```
[real]
1112*21
1112*21
1112*21
```

---

## 6. view Board

view Board 保存玩家当前看到的局面。

格式：

```
[view]
```

之后紧跟 `rows` 行。

每行必须包含 `columns` 个字符。

---

## 7. Render

Render 仅影响字符显示，不影响文件语义。

所有 Render 必须能够无损转换。

所有实现必须支持：

```
ascii
```

实现可选支持：

```
emoji
```

---

## 8. Character Definition

### Render: ascii（Canonical）

| 字符 | 含义 |
|------|------|
| U | 未翻开（Unopened） |
| 0~8 | 已翻开的数字 |
| F | 已插旗 |
| ? | 问号标记 |
| X | 踩中的地雷 |
| @ | 游戏结束后显示的未标记地雷 |
| # | 游戏结束后显示的错误旗帜 |

示例：

```
[view]
UUU2100
UF1X1?U
U@U#111
```

---

### Render: emoji（Pretty）

| ASCII | Emoji |
|--------|--------|
| U | ⬜ |
| 0 | 0️⃣ |
| 1 | 1️⃣ |
| 2 | 2️⃣ |
| 3 | 3️⃣ |
| 4 | 4️⃣ |
| 5 | 5️⃣ |
| 6 | 6️⃣ |
| 7 | 7️⃣ |
| 8 | 8️⃣ |
| F | 🚩 |
| ? | ❓ |
| X | 💥 |
| @ | 💣 |
| # | ❌ |

示例：

```
[view]
⬜⬜⬜2️⃣1️⃣0️⃣0️⃣
⬜🚩1️⃣💥1️⃣❓⬜
⬜💣⬜❌1️⃣1️⃣1️⃣
```

---

## 9. Validation Rules

解析器应验证：

- Header 合法。
- Render 合法。
- `rows`、`columns`、`mines` 存在。
- 至少存在一个 Board Section（real 或 view）。
- 若同时存在 real 与 view，则两者尺寸必须一致。
- 每行字符数必须等于 `columns`。
- Board 行数必须等于 `rows`。
- 不允许出现未定义字符。

---

## 10. Example

```
# MINESWEEPER-BOARD v0.1
# Render: ascii

author: eee555
comment: Example Board

rows: 16
columns: 30
mines: 99

game_mode: classic

[real]
1112*21
1112*21
1112*21

[view]
UUU2100
UF1X1?U
U@U#111
```

---

## 11. Compatibility

未来版本应遵循以下兼容性原则：

- 新增 Metadata 字段不得影响旧版解析器。
- 新增 Render 不得改变文件语义。
- 未知 Metadata 字段应被忽略。
- 未知注释应被忽略。
- 已定义字符的语义不得修改。