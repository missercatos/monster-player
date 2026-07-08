"""
msplayer Python bindings via ctypes.
Drop libmonster_player.so alongside this file, or set MSPLAYER_LIB_PATH.
"""

import ctypes
import json
import os
from ctypes import c_char_p, c_double, c_int32, c_uint32, c_void_p, POINTER, byref

_lib_path = os.environ.get("MSPLAYER_LIB_PATH", "./libmonster_player.so")
_lib = ctypes.CDLL(_lib_path)

# -- function signatures -----------------------------------------------------

_lib.msplayer_new.restype = c_void_p
_lib.msplayer_free.argtypes = [c_void_p]
_lib.msplayer_update.argtypes = [c_void_p]

_lib.msplayer_play_at.argtypes = [c_void_p, c_uint32]
_lib.msplayer_toggle_pause.argtypes = [c_void_p]
_lib.msplayer_restart.argtypes = [c_void_p]
_lib.msplayer_seek_to.argtypes = [c_void_p, c_double]
_lib.msplayer_seek_forward.argtypes = [c_void_p]
_lib.msplayer_seek_backward.argtypes = [c_void_p]
_lib.msplayer_volume_up.argtypes = [c_void_p]
_lib.msplayer_volume_down.argtypes = [c_void_p]
_lib.msplayer_cycle_mode.argtypes = [c_void_p]

_lib.msplayer_next_album.argtypes = [c_void_p]
_lib.msplayer_prev_album.argtypes = [c_void_p]
_lib.msplayer_next_song.argtypes = [c_void_p]
_lib.msplayer_prev_song.argtypes = [c_void_p]

_lib.msplayer_is_playing.argtypes = [c_void_p]; _lib.msplayer_is_playing.restype = c_int32
_lib.msplayer_is_buffering.argtypes = [c_void_p]; _lib.msplayer_is_buffering.restype = c_int32
_lib.msplayer_is_global_mode.argtypes = [c_void_p]; _lib.msplayer_is_global_mode.restype = c_int32
_lib.msplayer_volume.argtypes = [c_void_p]; _lib.msplayer_volume.restype = c_uint32
_lib.msplayer_song_count.argtypes = [c_void_p]; _lib.msplayer_song_count.restype = c_uint32
_lib.msplayer_album_count.argtypes = [c_void_p]; _lib.msplayer_album_count.restype = c_uint32
_lib.msplayer_elapsed.argtypes = [c_void_p]; _lib.msplayer_elapsed.restype = c_double
_lib.msplayer_duration.argtypes = [c_void_p]; _lib.msplayer_duration.restype = c_double
_lib.msplayer_progress.argtypes = [c_void_p]; _lib.msplayer_progress.restype = c_double
_lib.msplayer_play_mode.argtypes = [c_void_p]; _lib.msplayer_play_mode.restype = c_uint32
_lib.msplayer_mode_count.restype = c_uint32

_lib.msplayer_set_play_mode.argtypes = [c_void_p, c_uint32]
_lib.msplayer_toggle_love.argtypes = [c_void_p]

_lib.msplayer_snapshot.argtypes = [c_void_p, c_char_p, c_uint32]
_lib.msplayer_snapshot.restype = c_uint32

_lib.msplayer_enter_search.argtypes = [c_void_p]
_lib.msplayer_search_input.argtypes = [c_void_p, c_char_p, c_uint32]
_lib.msplayer_search_result_count.argtypes = [c_void_p]; _lib.msplayer_search_result_count.restype = c_uint32
_lib.msplayer_search_result_at.argtypes = [c_void_p, c_uint32, c_char_p, c_uint32]; _lib.msplayer_search_result_at.restype = c_int32
_lib.msplayer_search_confirm.argtypes = [c_void_p]

MODE_LABELS = [
    "AlbumList", "AlbumRandom", "GlobalList", "GlobalRandom",
    "Single", "LoveList", "LoveRandom",
]


class Engine:
    """Python wrapper around the msplayer C FFI engine."""

    def __init__(self):
        self._ptr = _lib.msplayer_new()

    def __del__(self):
        if self._ptr:
            _lib.msplayer_free(self._ptr)

    def update(self):
        _lib.msplayer_update(self._ptr)

    # -- playback --
    def play_at(self, index: int):
        _lib.msplayer_play_at(self._ptr, index)

    def toggle_pause(self):
        _lib.msplayer_toggle_pause(self._ptr)

    def restart(self):
        _lib.msplayer_restart(self._ptr)

    def seek_to(self, progress: float):
        _lib.msplayer_seek_to(self._ptr, progress)

    def seek_forward(self):
        _lib.msplayer_seek_forward(self._ptr)

    def seek_backward(self):
        _lib.msplayer_seek_backward(self._ptr)

    def volume_up(self):
        _lib.msplayer_volume_up(self._ptr)

    def volume_down(self):
        _lib.msplayer_volume_down(self._ptr)

    def cycle_mode(self):
        _lib.msplayer_cycle_mode(self._ptr)

    # -- navigation --
    def next_album(self):
        _lib.msplayer_next_album(self._ptr)

    def prev_album(self):
        _lib.msplayer_prev_album(self._ptr)

    def next_song(self):
        _lib.msplayer_next_song(self._ptr)

    def prev_song(self):
        _lib.msplayer_prev_song(self._ptr)

    # -- status --
    @property
    def playing(self) -> bool:       return bool(_lib.msplayer_is_playing(self._ptr))
    @property
    def buffering(self) -> bool:     return bool(_lib.msplayer_is_buffering(self._ptr))
    @property
    def is_global(self) -> bool:     return bool(_lib.msplayer_is_global_mode(self._ptr))
    @property
    def volume(self) -> int:         return _lib.msplayer_volume(self._ptr)
    @property
    def song_count(self) -> int:     return _lib.msplayer_song_count(self._ptr)
    @property
    def album_count(self) -> int:    return _lib.msplayer_album_count(self._ptr)
    @property
    def elapsed(self) -> float:      return _lib.msplayer_elapsed(self._ptr)
    @property
    def duration(self) -> float:     return _lib.msplayer_duration(self._ptr)
    @property
    def progress(self) -> float:     return _lib.msplayer_progress(self._ptr)
    @property
    def mode(self) -> int:           return _lib.msplayer_play_mode(self._ptr)
    @property
    def mode_label(self) -> str:     return MODE_LABELS[self.mode]

    def set_mode(self, m: int):
        _lib.msplayer_set_play_mode(self._ptr, m)

    @property
    def snapshot(self) -> dict:
        buf = ctypes.create_string_buffer(2048)
        n = _lib.msplayer_snapshot(self._ptr, buf, 2048)
        return json.loads(buf.value.decode())

    # -- favorites --
    def toggle_love(self):
        _lib.msplayer_toggle_love(self._ptr)

    # -- search --
    def enter_search(self):
        _lib.msplayer_enter_search(self._ptr)

    def search(self, query: str):
        q = query.encode()
        _lib.msplayer_search_input(self._ptr, q, len(q))

    @property
    def search_count(self) -> int:
        return _lib.msplayer_search_result_count(self._ptr)

    def search_result(self, idx: int) -> str | None:
        buf = ctypes.create_string_buffer(512)
        rc = _lib.msplayer_search_result_at(self._ptr, idx, buf, 512)
        return buf.value.decode() if rc == 0 else None

    def search_confirm(self):
        _lib.msplayer_search_confirm(self._ptr)


if __name__ == "__main__":
    e = Engine()
    print("mode:", e.mode_label)
    print("volume:", e.volume)
    print("snapshot:", e.snapshot)
