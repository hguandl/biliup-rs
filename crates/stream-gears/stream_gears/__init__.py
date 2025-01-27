from .pyobject import Credit, Segment
from .pyuploader import UploadHandle, upload_handle
from .stream_gears import *

__all__ = ["Segment", "Credit", "UploadHandle", "upload_handle"]
__doc__ = stream_gears.__doc__
if hasattr(stream_gears, "__all__"):
    __all__.extend(stream_gears.__all__)
