import ms_toollib as ms

file_path = "../../test_files/HI-SCORE Exp_49.25_3BV=127_3BVs=2.57_Wang Jianing G01825.avf"

class MyVideo(ms.AvfVideo):
    def __new__(cls, *args, **kargs):
        return ms.AvfVideo.__new__(cls, *args, **kargs)
    def __init__(self, *args, **kargs):
        super(MyVideo, self).__init__()
    def print_something(self):
        self.parse_video()
        self.analyse()
        self.current_time = 999999
        print(f"mode: {self.mode}")
        print(f"level: {self.level}")
        print(f"time:{self.time}")
        print(f"bbbv: {self.bbbv}")
        print(f"cl:{self.cl}")
        print(f"ce: {self.ce}")
        print(f"flag: {self.flag}")
my_video = MyVideo(file_path)
my_video.print_something()
print(my_video.bbbv_solved)


# 使用绝对路径实例化
v_1 = ms.AvfVideo(file_path)

# 使用二进制列表实例化
with open(file_path, 'rb') as file:
    video_data_list = list(file.read())
    # 自定义另一个文件名
    v_2 = ms.AvfVideo(r"my_file.avf", video_data_list)
    # 也可以缺省第一个参数`file_name`，则file_name为空字符串
    v_3 = ms.AvfVideo(raw_data=video_data_list)

