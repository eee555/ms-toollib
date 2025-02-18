import ms_toollib as ms

bbbv_distribution = ms.sample_bbbvs_exp(0, 0, 100000);
print(bbbv_distribution)

for i in range(100000):
    board = ms.laymine(row=16, column=30, mine_num=99, x0=0, y0=0)
    wrap_board = ms.Board(board)
    print(wrap_board.bbbv)
    print(wrap_board.op)
    print(wrap_board.isl)
    print(wrap_board.cell0)
    print(wrap_board.cell1)
    print(wrap_board.cell2)
    print(wrap_board.cell3)
    ...
