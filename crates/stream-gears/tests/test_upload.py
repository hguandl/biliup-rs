import time

import stream_gears

if __name__ == "__main__":
    handle = stream_gears.upload_handle(
        ["examples/test.mp4"],
        "cookies.json",
        "title",
        171,
        "tag",
        None,
        1,
        "source",
        "desc",
        "dynamic",
        "",
        0,
        0,
        0,
        0,
        3,
        [],
        None,
        stream_gears.UploadLine.Bda2,
        "",
        None,
    )

    handle.start()

    while handle.total == 0 or handle.uploaded < handle.total:
        speed = int(handle.speed / 10000) / 100
        print(f"{handle.uploaded}/{handle.total} Bytes [{speed}MB/s]")
        time.sleep(1)

    handle.wait()

    stream_gears.upload_by_app(
        ["examples/test.mp4", "examples/test2.mp4"],
        "cookies.json",
        "dadad",
        171,
        "演示",
        None,
        1,
        "",
        "",
        "",
        "",
        0,
        0,
        0,
        0,
        True,
        False,
        False,
        3,
        [],
        None,
        stream_gears.UploadLine.Qn,
        "",
        None,
        None,
    )
