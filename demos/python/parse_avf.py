import ms_toollib as ms
v = ms.AvfVideo("../../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf") # 第一步，读取文件的二进制内容
v.parse_video() # 第二步，解析文件的二进制内容
v.analyse() # 第三步，根据解析到的内容，推衍整个局面
v.current_time = 999.999 # set time to the end of the v
print(v.left)
print(v.right)
print(v.double)
print(v.left_s)
print(v.right_s)
print(v.double_s)
print(v.level)
print(v.cl)
print(v.cl_s)
print(v.ce)
print(v.ce_s)
print(v.bbbv)
print(v.bbbv_solved)
print(v.bbbv_s)
print(v.flag)
print(v.path)
print(v.time)  # the time shown on the counter currently
print(v.rtime) # game time, shown on leaderboard
print(v.etime) # the estimated time shown on the counter currently
print(v.start_time)
print(v.end_time)
print(v.mode)
print(v.software)
print(v.stnb)
print(v.corr)
print(v.thrp)
print(v.ioe)
print(v.is_official)
print(v.is_fair)
v.analyse_for_features(["high_risk_guess"]) # 用哪些分析方法。分析结果会记录到events.comments里
for e in v.events:
    print(e.time, e.x, e.y, e.mouse, e.path, e.comments, e.mouse_state)
    # left, right, double, lce, rce, dce, flag, bbbv_solved, op_solved, isl_solved
    print(e.key_dynamic_params.left)
    ...
for e in v.events:
    if e.useful_level >= 2:
        # 该事件发生前的游戏局面在game_board_stream中的id索引
        prior_game_board_id = e.prior_game_board_id
        # 内置的游戏局面类
        builtin_game_board = v.game_board_stream[prior_game_board_id]
        # 内置的游戏局面类的列表类型的游戏局面
        print(builtin_game_board.game_board)
        # 内置的游戏局面类的每格是雷的概率
        print(builtin_game_board.poss)
        # 用1或2个方程求解时，得出的所有非雷的位置
        print(builtin_game_board.basic_not_mine)
        # 用1或2个方程求解时，得出的所有是雷的位置
        print(builtin_game_board.basic_is_mine)
        # 用枚举法求解时，得出的所有非雷的位置，不包括basic_not_mine
        print(builtin_game_board.enum_not_mine)
        # 用枚举法求解时，得出的所有是雷的位置，不包括basic_is_mine
        print(builtin_game_board.enum_is_mine)
        