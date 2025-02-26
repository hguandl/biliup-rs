import json
import stream_gears
import sys


# 获取用户信息
my_info = json.loads(
    stream_gears.my_info(
        sys.argv[1],  # Cookie 文件
    )
)

silence = my_info["profile"]["silence"]  # 是否被封禁
print(f"封禁: {silence}")

if silence == 1:
    stream_gears.my_appeal(
        sys.argv[1],  # Cookie 文件
        "我只上传了一个视频，这很可能是误封，请帮忙处理，谢谢！",  # 申诉理由
    )
