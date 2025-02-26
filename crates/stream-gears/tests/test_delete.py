import json
import stream_gears
import sys

failure_codes = [
    -2,  # 已退回
    -3,  # 网警锁定 网警删除
    -4,  # 已锁定
    -12,  # 上传失败
    -16,  # 转码失败
]

# 获取投稿列表
archive_page = json.loads(
    stream_gears.archives(
        sys.argv[1],  # Cookie 文件
        "is_pubing,pubed,not_pubed",  # 稿件状态 (pubed: 已发布, is_pubing: 审核中, not_pubed: 未过审)
        1,  # 页码, 每页最多 10 条
    )
)

# 遍历投稿列表
for audit in archive_page["arc_audits"]:
    archive = audit["Archive"]  # 投稿信息

    bvid = archive["bvid"]  # 投稿 BV 号
    title = archive["title"]  # 投稿标题
    state = archive["state"]  # 投稿状态 (0: 过审)

    # 打印投稿信息 (debug 用)
    print(title, bvid, state)

    # 如果状态码是失败，删除视频
    if state in failure_codes:
        stream_gears.delete(sys.argv[1], bvid)
        continue
