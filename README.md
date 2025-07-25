# ms_toollib、EVF Format

[![ms_toollib](https://img.shields.io/badge/ms_toollib-v1.4.17-brightgreen.svg)](https://github.com/eee555/ms_toollib)

史上唯一专门的、跨平台、跨语言的扫雷算法工具箱。已发布到：

- crates.io
- pypi.org
- npmjs.com

可在Rust、Python(Windows、Linux)、Javascript(bundler、nodejs)、Typescript(bundler、nodejs)、C(Windows)、C++(Linux)、julia等语言/平台中使用，并提供案例，可快速上手。项目呈比较成熟阶段，相关技术问题可以提供一对一交流解决。欢迎star、pull request、fork、issue（提需求、报bug等）

Algorithms for minesweeper, published on various platforms.

- 目前主要文档见[https://docs.rs/ms_toollib](https://docs.rs/ms_toollib)。
最新版本号统计：  
版本号越大，代表越新、功能越完善、bug越少（可以催）。

python>=3.7, <=3.12 (适用于以下架构windows: x86, x64; linux: aarch64, armv7, ppc64le, s390x, x86, x86_64; macos: aarch64, x86_64): 1.4.18

javascript/typescript (webpack等bundler): 1.4.18

javascript/typescript (nodejs): 1.4.12-alpha

rust crate: 1.4.17

C(仅windows): 1.0.0 （没有类，即没有录像解析工具、局面状态机等。调试环境为MSVC。Linux未经测试，但估计可用。没有包管理平台，需要用户用源码自行编译，目前需要安装rust工具链，自行编译得到.lib文件）

C++(仅Linux): 1.0.0
（采用Cmake构建、没有包管理平台，需要用户用源码自行编译，需要安装rust工具链。）

Julia: 同python


### 核心功能

- **埋雷**：无猜埋雷，包括筛选法埋雷（雷密度极高时无法成功、成功时保证无猜）、调整法埋雷（百分百成功、成功时概率有猜）。

- **局面状态机**：接受用户的鼠标点击为输入，自动计时、推衍局面变化、计算指标数据。

- **光学局面识别**：输入局面的彩色图片数据，智能识别局面。

- **概率计算**：输入局面、总雷数，计算各位置是雷的概率。

- **录像处理**：avf、evf、mvf、rmv等扫雷录像格式的读取；将局面状态机中的数据保存或另存为evf录像。

### 如何调试源码

在编译之前，请确保自己拥有：

*   Rust工具链(rustup -V能够打印版本号、cargo -V能够打印版本号)
*   Visual Studio Code及对应插件(例如rust-analyzer)
*   会用Powershell或者其它命令行工具的能力
*   安装完全部环境以后，还剩余至少6G的硬盘容量

以下为调试步骤：

*   克隆这个仓库到本地
```sh
    git clone https://github.com/eee555/ms_toollib.git
```

*   用Visual Studio Code打开base文件夹

*   编辑器打开需要执行的测试程序文件，例如tests/test_analyse.rs

*   找到您想要执行的测试程序，例如minesweeper_board_works这个函数，用鼠标点击#[test]下方的灰色的Run Test按钮，即可打印执行结果！


### 其他示例项目

- [https://github.com/eee555/Metasweeper](https://github.com/eee555/Metasweeper)：元扫雷，win10/11平台的开源专业扫雷。具有全部6种无猜扫雷模式+标准+win7，演示了本仓库的埋雷、局面状态机、光学局面识别、概率计算、录像保存播放等功能。

- [https://github.com/eee555/saolei_website](https://github.com/eee555/saolei_website)：开源扫雷网，前后端均开源的的扫雷纪录排名网站。演示了本仓库在前后端用于录像校验、数据计算功能。

- [https://apps.apple.com/cn/app/益智扫雷/id6748243595?uo=4][https://apps.apple.com/cn/app/益智扫雷/id6748243595?uo=4]：益智扫雷，App Store上的无猜扫雷软件。使用了本仓库的埋雷算法，有条件可以体验。






