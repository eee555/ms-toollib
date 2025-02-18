import ms_toollib as ms

help(ms.laymine)
board = ms.laymine(row=16, column=30, mine_num=99, x0=0, y0=0)
print(board)

help(ms.laymine_op)
board_op = ms.laymine_op(row=16, column=30, mine_num=99, x0=0, y0=0)
print(board_op)
