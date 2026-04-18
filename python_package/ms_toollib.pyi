# 放到和ms_toollib.pyd同一目录下

from typing import Any, Sequence, Union, Tuple, List, Optional, Callable


def __getattr__(name: str) -> Any: ...

# === Functions (27) ===


def cal_all_solution(a: List[List[int]], b: List[int]) -> List[List[int]]: ...
def cal_bbbv(board: List[List[int]]) -> int: ...
def cal_board_numbers(board: List[List[int]]) -> List[List[int]]: ...
def cal_op(board: List[List[int]]) -> int: ...


def cal_probability(game_board: List[List[int]], mine_num: float) -> tuple[List[tuple[tuple[int,
                                                                                            int], float]], float, List[int], int]: ...


def cal_probability_onboard(game_board: List[List[int]], mine_num: float) -> tuple[List[List[float]], List[int]]:
    """计算局面中各位置是雷的概率，按照所在的位置返回。

# 参数
- `game_board`: 游戏局面。自动纠正错误的标雷。
- `mine_num`：雷数。>=1时，理解为总的雷数；<1时，理解为雷的比例。

# 返回值
- 元组的第一个元素是一个局面中，所有位置是雷的概率
- 元组的第二个元素是一个长度为3的列表，表示最小雷数、当前雷数、最大雷数

# 异常
- `PyRuntimeError`: `标记阶段无解的局面`和`枚举阶段无解的局面`两种。"""


def get_all_not_and_is_mine_on_board(
    game_board: List[List[int]]) -> tuple[List[List[int]], List[tuple[int, int]], List[tuple[int, int]]]: ...


def is_able_to_solve(
    board_of_game: List[List[int]], xy: tuple[int, int]) -> bool: ...
def is_guess_while_needless(
    board_of_game: List[List[int]], xy: tuple[int, int]) -> int: ...


def is_solvable(board: List[List[int]], x0: int, y0: int) -> bool: ...


def laymine(row: int, column: int, mine_num: int, x0: int, y0: int) -> List[List[int]]:
    """通用标准埋雷引擎。起手位置非雷，其余位置的雷服从均匀分布。

# 参数
- `row`: 局面行数。
- `column`：局面列数。。
- `mine_num`：雷数。
- `x0`：起手位置在第几行。
- `y0`：起手位置在第几列。

# 返回值
二维的局面，其中0代表空，1~8代表1~8，-1代表雷。"""


def laymine_op(row: int, column: int, mine_num: int, x0: int, y0: int) -> List[List[int]]:
    """通用win7规则埋雷引擎。起手位置开空，其余位置的雷服从均匀分布。

# 参数
- `row`: 局面行数。
- `column`：局面列数。。
- `mine_num`：雷数。
- `x0`：起手位置在第几行。
- `y0`：起手位置在第几列。

# 返回值
二维的局面，其中0代表空，1~8代表1~8，-1代表雷。"""


def laymine_solvable(row: int, column: int, mine_num: int, x0: int,
                     y0: int, max_times: int = 1000000) -> tuple[List[List[int]], bool]: ...


def laymine_solvable_adjust(row: int, column: int, mine_num: int,
                            x0: int, y0: int) -> tuple[List[List[int]], bool]: ...


def laymine_solvable_thread(row: int, column: int, mine_num: int, x0: int,
                            y0: int, max_times: int = 1000000) -> tuple[List[List[int]], bool]: ...


def mark_board(game_board: List[List[int]],
               remark: bool = False) -> List[List[int]]: ...


def obr_board(data_vec: List[int], height: int,
              width: int) -> List[List[int]]: ...


def refresh_board(board: List[List[int]], board_of_game: List[List[int]],
                  clicked_poses: List[tuple[int, int]]) -> List[List[int]]: ...


def refresh_matrix(game_board: List[List[int]]) -> tuple[List[List[int]],
                                                         List[tuple[int, int]], List[int]]: ...


def refresh_matrixs(game_board: List[List[int]]) -> tuple[List[List[List[int]]],
                                                          List[List[tuple[int, int]]], List[List[int]], int, int]: ...
def refresh_matrixses(board_of_game: List[List[int]]) -> tuple[List[List[List[List[int]]]],
                                                               List[List[List[tuple[int, int]]]], List[List[List[int]]]]: ...


def sample_bbbvs_exp(x0: int, y0: int, n: int) -> List[int]: ...


def solve_direct(a_mats: List[List[List[int]]], xs: List[List[tuple[int, int]]], bs: List[List[int]], board_of_game: List[List[int]]) -> tuple[List[List[List[int]]],
                                                                                                                                               List[List[tuple[int, int]]], List[List[int]], List[List[int]], List[tuple[int, int]], List[tuple[int, int]]]: ...


def solve_enumerate(
    board_of_game: List[List[int]]) -> tuple[List[tuple[int, int]], List[tuple[int, int]]]: ...
def solve_minus(a_mats: List[List[List[int]]], xs: List[List[tuple[int, int]]], bs: List[List[int]], board_of_game: List[List[int]]) -> tuple[List[List[List[int]]],
                                                                                                                                              List[List[tuple[int, int]]], List[List[int]], List[List[int]], List[tuple[int, int]], List[tuple[int, int]]]: ...


def unsolvable_structure(board_check: List[List[int]]) -> bool: ...


def valid_time_period(software: str) -> tuple[str, str]:
    """软件的合法时间范围。单位为秒。

# 参数
- `software`: 软件名称，"Arbiter"、"0.97 beta"、"Viennasweeper"、"元3.1.9"、"元3.1.11"、"元3.2.0"等等

# 返回值
秒为单位的开始时间戳字符串、秒为单位的结束时间戳字符串"""

# === Classes (15) ===


class AvfVideo:
    def __init__(self, file_name: str = '',
                 raw_data: List[int] = ...) -> None: ...

    file_name: str
    raw_data: List[int]
    time: float
    software: str
    row: int
    column: int
    level: int
    mode: int
    is_completed: bool
    is_official: bool
    is_fair: bool
    mine_num: int
    player_identifier: str
    race_identifier: str
    uniqueness_identifier: str
    country: str
    bbbv: int
    start_time: int
    end_time: int
    op: int
    isl: int
    hizi: int
    cell0: int
    cell1: int
    cell2: int
    cell3: int
    cell4: int
    cell5: int
    cell6: int
    cell7: int
    cell8: int
    rtime: float
    rtime_ms: int
    etime: float
    bbbv_s: float
    stnb: float
    rqp: float
    left: int
    right: int
    double: int
    cl: int
    flag: int
    bbbv_solved: int
    lce: int
    rce: int
    dce: int
    ce: int
    left_s: float
    right_s: float
    double_s: float
    cl_s: float
    flag_s: float
    path: float
    ce_s: float
    ioe: float
    thrp: float
    corr: float
    pluck: float
    events: List[VideoActionStateRecorder]

    def analyse(self) -> None: ...
    def analyse_for_features(self, controller: List[str]) -> None: ...
    def generate_evf_v0_raw_data(self) -> None: ...
    def generate_evf_v2_raw_data(self) -> None: ...
    def generate_evf_v3_raw_data(self) -> None: ...
    def generate_evf_v4_raw_data(self) -> None: ...
    def is_valid(self) -> int: ...
    def parse(self) -> None: ...
    def save_to_evf_file(self, file_name: str) -> str: ...


class BaseVideo:
    def __init__(self, board: Any, cell_pixel_size: Any = 16) -> None: ...

    raw_data: List[int]
    time: float
    software: str
    row: int
    column: int
    level: int
    mode: int
    is_completed: bool
    is_official: bool
    is_fair: bool
    mine_num: int
    player_identifier: str
    race_identifier: str
    uniqueness_identifier: str
    country: str
    bbbv: int
    start_time: int
    end_time: int
    op: int
    isl: int
    hizi: int
    cell0: int
    cell1: int
    cell2: int
    cell3: int
    cell4: int
    cell5: int
    cell6: int
    cell7: int
    cell8: int
    rtime: float
    rtime_ms: int
    etime: float
    video_start_time: float
    video_end_time: float
    bbbv_s: float
    stnb: float
    rqp: float
    left: int
    right: int
    double: int
    cl: int
    flag: int
    bbbv_solved: int
    lce: int
    rce: int
    dce: int
    ce: int
    left_s: float
    right_s: float
    double_s: float
    cl_s: float
    flag_s: float
    path: float
    ce_s: float
    ioe: float
    thrp: float
    corr: float
    pluck: float
    events: List[VideoActionStateRecorder]
    current_event_id: int
    board: SafeBoard
    game_board: List[List[int]]
    game_board_poss: List[List[float]]
    mouse_state: int
    game_board_state: int
    x_y: tuple[int, int]
    checksum: List[int]
    video_playing_pix_size: int
    current_time: float
    use_question: bool
    use_cursor_pos_lim: bool
    use_auto_replay: bool
    device_uuid: List[int]
    pix_size: int

    def generate_evf_v0_raw_data(self) -> None: ...
    def generate_evf_v2_raw_data(self) -> None: ...
    def generate_evf_v3_raw_data(self) -> None: ...
    def generate_evf_v4_raw_data(self) -> None: ...
    def loss_then_open_all_mine(self) -> None: ...
    def reset(self, row: int, column: int, pix_size: int) -> None: ...
    def save_to_evf_file(self, file_name: str) -> None: ...
    def step(self, e: str, pos: tuple[int, int]) -> None: ...
    def step_game_state(self, e: str) -> None: ...
    def win_then_flag_all_mine(self) -> None: ...


class Board:
    def __init__(self, board: List[List[int]]) -> None: ...

    bbbv: int
    op: int
    isl: int
    cell0: int
    cell1: int
    cell2: int
    cell3: int
    cell4: int
    cell5: int
    cell6: int
    cell7: int
    cell8: int


class EvfVideo:
    def __init__(self, file_name: str = '',
                 raw_data: List[int] = ...) -> None: ...

    file_name: str
    raw_data: List[int]
    time: float
    software: str
    row: int
    column: int
    level: int
    mode: int
    is_completed: bool
    is_official: bool
    is_fair: bool
    mine_num: int
    player_identifier: str
    race_identifier: str
    uniqueness_identifier: str
    country: str
    bbbv: int
    start_time: int
    end_time: int
    op: int
    isl: int
    hizi: int
    cell0: int
    cell1: int
    cell2: int
    cell3: int
    cell4: int
    cell5: int
    cell6: int
    cell7: int
    cell8: int
    rtime: float
    rtime_ms: int
    etime: float
    bbbv_s: float
    stnb: float
    rqp: float
    left: int
    right: int
    double: int
    cl: int
    flag: int
    bbbv_solved: int
    lce: int
    rce: int
    dce: int
    ce: int
    left_s: float
    right_s: float
    double_s: float
    cl_s: float
    flag_s: float
    path: float
    ce_s: float
    ioe: float
    thrp: float
    corr: float
    pluck: float
    events: List[VideoActionStateRecorder]

    def analyse(self) -> None: ...
    def analyse_for_features(self, controller: List[str]) -> None: ...
    def generate_evf_v0_raw_data(self) -> None: ...
    def generate_evf_v2_raw_data(self) -> None: ...
    def generate_evf_v3_raw_data(self) -> None: ...
    def generate_evf_v4_raw_data(self) -> None: ...
    def is_valid(self) -> int: ...
    def parse(self) -> None: ...
    def save_to_evf_file(self, file_name: str) -> str: ...


class Evfs:
    def __init__(self, file_name: Any = '',
                 raw_data: Any = Ellipsis) -> None: ...

    file_name: str
    software: str
    evf_version: int
    start_time: int
    end_time: int

    def analyse(self) -> None: ...
    def analyse_for_features(self, controller: List[str]) -> None: ...
    def clear(self) -> None: ...
    def generate_evfs_v0_raw_data(self) -> None: """生成evfs_v0文件的二进制数据"""
    def is_empty(self) -> bool: ...
    def is_valid(self) -> bool: """初步验证evfs文件的有效性。适用于网页前端，并不严格。"""
    def len(self) -> int: ...
    def parse(self) -> None: ...
    def pop(self) -> None: ...
    def push(self, data: List[int], file_name: str,
             checksum: List[int]) -> None: ...

    def save_evf_files(self, dir: str) -> None: ...
    def save_evfs_file(self, file_name: str) -> str: ...


class EvfsCell:
    def __init__(self, *args, **kwargs) -> None: ...

    evf_video: EvfVideo
    checksum: List[int]


class GameBoard:
    def __init__(self, mine_num: int) -> None: ...

    game_board: List[List[int]]
    poss: List[List[float]]
    basic_not_mine: List[tuple[int, int]]
    basic_is_mine: List[tuple[int, int]]
    enum_not_mine: List[tuple[int, int]]
    enum_is_mine: List[tuple[int, int]]


class KeyDynamicParams:
    def __init__(self, *args, **kwargs) -> None: ...

    left: int
    right: int
    double: int
    lce: int
    rce: int
    dce: int
    flag: int
    bbbv_solved: int
    op_solved: int
    isl_solved: int


class MinesweeperBoard:
    def __init__(self, board: List[List[int]]) -> None: ...

    board: List[List[int]]
    game_board: List[List[int]]
    left: int
    right: int
    double: int
    lce: int
    rce: int
    dce: int
    ce: int
    flag: int
    bbbv_solved: int
    row: int
    column: int
    game_board_state: int
    mouse_state: int

    def get_game_board_2(self, mine_num: float) -> List[List[List[float]]]: ...
    def reset(self) -> None: ...
    def step(self, e: str, pos: tuple[int, int]) -> None: ...
    def step_flow(
        self, operation: List[tuple[str, tuple[int, int]]]) -> None: ...


class MvfVideo:
    def __init__(self, file_name: str = '',
                 raw_data: List[int] = ...) -> None: ...

    file_name: str
    raw_data: List[int]
    time: float
    software: str
    row: int
    column: int
    level: int
    mode: int
    is_completed: bool
    is_official: bool
    is_fair: bool
    mine_num: int
    player_identifier: str
    race_identifier: str
    uniqueness_identifier: str
    country: str
    bbbv: int
    start_time: int
    end_time: int
    op: int
    isl: int
    hizi: int
    cell0: int
    cell1: int
    cell2: int
    cell3: int
    cell4: int
    cell5: int
    cell6: int
    cell7: int
    cell8: int
    rtime: float
    rtime_ms: int
    etime: float
    bbbv_s: float
    stnb: float
    rqp: float
    left: int
    right: int
    double: int
    cl: int
    flag: int
    bbbv_solved: int
    lce: int
    rce: int
    dce: int
    ce: int
    left_s: float
    right_s: float
    double_s: float
    cl_s: float
    flag_s: float
    path: float
    ce_s: float
    ioe: float
    thrp: float
    corr: float
    pluck: float
    events: List[VideoActionStateRecorder]

    def analyse(self) -> None: ...
    def analyse_for_features(self, controller: List[str]) -> None: ...
    def generate_evf_v0_raw_data(self) -> None: ...
    def generate_evf_v2_raw_data(self) -> None: ...
    def generate_evf_v3_raw_data(self) -> None: ...
    def generate_evf_v4_raw_data(self) -> None: ...
    def is_valid(self) -> int: ...
    def parse(self) -> None: ...
    def save_to_evf_file(self, file_name: str) -> str: ...


class RmvVideo:
    def __init__(self, file_name: str = '',
                 raw_data: List[int] = ...) -> None: ...

    file_name: str
    raw_data: List[int]
    time: float
    software: str
    row: int
    column: int
    level: int
    mode: int
    is_completed: bool
    is_official: bool
    is_fair: bool
    mine_num: int
    player_identifier: str
    race_identifier: str
    uniqueness_identifier: str
    country: str
    bbbv: int
    start_time: int
    end_time: int
    op: int
    isl: int
    hizi: int
    cell0: int
    cell1: int
    cell2: int
    cell3: int
    cell4: int
    cell5: int
    cell6: int
    cell7: int
    cell8: int
    rtime: float
    rtime_ms: int
    etime: float
    bbbv_s: float
    stnb: float
    rqp: float
    left: int
    right: int
    double: int
    cl: int
    flag: int
    bbbv_solved: int
    lce: int
    rce: int
    dce: int
    ce: int
    left_s: float
    right_s: float
    double_s: float
    cl_s: float
    flag_s: float
    path: float
    ce_s: float
    ioe: float
    thrp: float
    corr: float
    pluck: float
    events: List[VideoActionStateRecorder]

    def analyse(self) -> None: ...
    def analyse_for_features(self, controller: List[str]) -> None: ...
    def generate_evf_v0_raw_data(self) -> None: ...
    def generate_evf_v2_raw_data(self) -> None: ...
    def generate_evf_v3_raw_data(self) -> None: ...
    def generate_evf_v4_raw_data(self) -> None: ...
    def is_valid(self) -> int: ...
    def parse(self) -> None: ...
    def save_to_evf_file(self, file_name: str) -> str: ...


class SafeBoard:
    def __init__(self, board: List[List[int]]) -> None: ...

    def into_vec_vec(self) -> List[List[int]]: ...
    def set(self, board: List[List[int]]) -> None: ...


class SafeBoardRow:
    def __init__(self, row: List[int]) -> None: ...


class SafeMinesweeperBoard:
    def __init__(self, board: List[List[int]]) -> None: ...

    board: List[List[int]]
    game_board: List[List[int]]
    left: int
    right: int
    double: int
    lce: int
    rce: int
    dce: int
    ce: int
    flag: int
    bbbv_solved: int
    row: int
    column: int
    game_board_state: int
    mouse_state: int

    def get_game_board_2(self, mine_num: float) -> List[List[List[float]]]: ...
    def step(self, e: str, pos: tuple[int, int]) -> None: ...
    def step_flow(
        self, operation: List[tuple[str, tuple[int, int]]]) -> None: ...


class VideoActionStateRecorder:
    def __init__(self, *args, **kwargs) -> None: ...

    time: float
    event: "Event"
    useful_level: int
    prior_game_board: GameBoard
    next_game_board: GameBoard
    comments: str
    path: float
    mouse_state: int
    key_dynamic_params: KeyDynamicParams
