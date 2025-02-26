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

cookie_file = sys.argv[1]

# 获取投稿列表
archive_page = json.loads(
    stream_gears.archives(
        cookie_file,  # Cookie 文件
        "not_pubed",  # 稿件状态 (pubed: 已发布, is_pubing: 审核中, not_pubed: 未过审)
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

    # 如果正常过审就跳过
    if state == 0:
        continue

    # 稿件具体信息
    studio = stream_gears.fetch(cookie_file, bvid)
    studio = json.loads(studio)

    # 如果只有一分P，直接删除
    if len(studio["videos"]) == 1:
        stream_gears.delete(cookie_file, bvid)
        continue

    # 遍历每一分P，找出未过审的
    rejected_filenames = []
    for video in studio["videos"]:
        if video["status"] != 0:
            print(
                f"P{video['index']}: {video['title']} ({video['status']})"
                f" {video['reject_reason']}"
            )
            rejected_filenames.append(video["filename"])

    # 如果所有分P都未过审，直接删除
    if len(rejected_filenames) == len(studio["videos"]):
        stream_gears.delete(cookie_file, bvid)
        continue

    # 修改视频信息
    stream_gears.edit(
        cookie_file,
        bvid,
        None,  # 视频标题, None 为不修改
        None,  # 封面图片路径, None 为不修改
        None,  # 视频标签, None 为不修改
        rejected_filenames,  # 删除的分P文件名列表
    )
