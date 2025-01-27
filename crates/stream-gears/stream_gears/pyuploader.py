import datetime
import threading
from typing import List, Optional

from .pyobject import Credit, UploadLine
from .stream_gears import upload


class UploadHandle:
    """
    上传任务句柄，用于监控上传进度，上传任务在独立线程中执行

    - 需通过`upload_handle`函数创建实例
    - 创建后需调用`start`方法开始上传
    - 可调用`wait`方法等待上传线程结束
    """

    def __init__(self):
        self._start_time = None
        self._total = None
        self._thread = None
        self._uploaded = None

    @property
    def total(self):
        """
        上传文件总大小（字节）
        """
        if self._total is not None:
            return self._total
        raise RuntimeError("Upload not started yet")

    @property
    def uploaded(self):
        """
        已上传大小（字节）
        """
        if self._uploaded is not None:
            return self._uploaded
        raise RuntimeError("Upload not started yet")

    @property
    def speed(self):
        """
        上传速度（字节/秒）
        """
        if self._uploaded is not None and self._start_time is not None:
            delta = datetime.datetime.now() - self._start_time
            return self._uploaded / delta.total_seconds()
        raise RuntimeError("Upload not started yet")

    def setup(self, thread: threading.Thread):
        if self._thread:
            raise RuntimeError("Upload already started")
        self._thread = thread

    def start(self):
        """
        开始上传，请勿重复调用
        """
        if not self._thread:
            raise RuntimeError("Upload not setup yet")
        self._start_time = datetime.datetime.now()
        self._total = 0
        self._uploaded = 0
        self._thread.start()

    def update(self, chunk_bytes: int, total_bytes: int):
        if self._total is not None and self._uploaded is not None:
            self._total = total_bytes
            self._uploaded += chunk_bytes

    def wait(self):
        """
        等待上传线程结束
        """
        if self._thread:
            self._thread.join()


def upload_handle(
    video_path: List[str],
    cookie_file: str,
    title: str,
    tid: int = 171,
    tag: str = "",
    topic_id: Optional[int] = None,
    copyright: int = 2,
    source: str = "",
    desc: str = "",
    dynamic: str = "",
    cover: str = "",
    dolby: int = 0,
    lossless_music: int = 0,
    no_reprint: int = 0,
    open_elec: int = 0,
    limit: int = 3,
    desc_v2: List[Credit] = [],
    dtime: Optional[int] = None,
    line: Optional[UploadLine] = None,
    extra_fields: Optional[str] = "",
    proxy: Optional[str] = None,
) -> UploadHandle:
    """
    创建上传任务，在独立线程中执行，并返回句柄以供监控上传进度

    :param List[str] video_path: 视频文件路径
    :param str cookie_file: cookie文件路径
    :param str title: 视频标题
    :param int tid: 投稿分区
    :param str tag: 视频标签, 英文逗号分隔多个tag
    :param Optional[int] topic_id: 话题ID
    :param int copyright: 是否转载, 1-自制 2-转载
    :param str source: 转载来源
    :param str desc: 视频简介
    :param str dynamic: 空间动态
    :param str cover: 视频封面
    :param int dolby: 是否开启杜比音效, 0-关闭 1-开启
    :param int lossless_music: 是否开启Hi-Res, 0-关闭 1-开启
    :param int no_reprint: 是否禁止转载, 0-允许 1-禁止
    :param int open_elec: 是否开启充电, 0-关闭 1-开启
    :param int limit: 单视频文件最大并发数
    :param List[Credit] desc_v2: 视频简介v2
    :param Optional[dtime] int dtime: 定时发布时间, 距离提交大于2小时小于15天, 格式为10位时间戳
    :param Optional[UploadLine] line: 上传线路
    :param Optional[ExtraFields] line: 上传额外参数
    :param Optional[str] proxy: 代理
    """

    handle = UploadHandle()

    def upload_callback(chunk_bytes: int, total_bytes: int):
        nonlocal handle
        handle.update(chunk_bytes, total_bytes)

    thread = threading.Thread(
        target=upload,
        args=(
            video_path,
            cookie_file,
            title,
            tid,
            tag,
            topic_id,
            copyright,
            source,
            desc,
            dynamic,
            cover,
            dolby,
            lossless_music,
            no_reprint,
            open_elec,
            limit,
            desc_v2,
            dtime,
            line,
            extra_fields,
            upload_callback,
            proxy,
        ),
    )

    handle.setup(thread)

    return handle
