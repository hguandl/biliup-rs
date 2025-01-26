import stream_gears


def upload_callback():
    uploaded = 0

    def callback_inner(chunk_bytes: int, total_bytes: int):
        nonlocal uploaded
        uploaded += chunk_bytes
        print(
            f"Uploaded {uploaded}/{total_bytes} bytes"
            f" ({uploaded / total_bytes * 100:.2f}%)"
        )

    return callback_inner


if __name__ == "__main__":
    stream_gears.upload(
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
        None,
        upload_callback(),
    )

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
        None,
        None,
    )
