# 需要有ms_toollib/demos/python/params.onnx
import ms_toollib
import matplotlib.image as mpimg
# 读取 PNG 图片
import numpy as np
file = r'../../test_files/exp.png'# 彩色图片
board_img_data = mpimg.imread(file)
(height, width) = board_img_data.shape[:2]
board_img_data = np.concatenate((board_img_data, 1.0 * np.ones((height, width, 1))), 2)
# 按照b, g, r, a，交换两个通道
t = board_img_data[:,:,0].copy()
board_img_data[:,:,0] = board_img_data[:,:,2]
board_img_data[:,:,2] = t
board_img_data = (np.reshape(board_img_data, -1) * 255).astype(np.uint8)
# board_img_data(b, g, r, a): [205, 197, 122, 255, 205, 197, 122, 255, 205, 197, 122, 255, ...]
# 对b,g,r三种颜色敏感；透明度随意。board_img_data的长度为4*height*width
board = ms_toollib.obr_board(board_img_data, height, width)
print(np.array(board))# 打印识别出的局面
poss = ms_toollib.cal_probability(board, 99)
# 用雷的总数计算概率
print(poss)
poss_onboard = ms_toollib.cal_probability_onboard(board, 99)
# 用雷的总数计算概率，输出局面对应的位置
print(poss_onboard)
poss_ = ms_toollib.cal_probability_onboard(board, 0.20625)
# 用雷的密度计算概率
print(poss_)