// C-compatible FFI bindings for the monster-player kernel.

use std::ffi::CStr;
use std::os::raw::c_char;

use crate::kernel::{Engine, PlayMode};

// --- Helpers ----------------------------------------------------------------

unsafe fn write_cstr(ptr: *mut c_char, cap: u32, s: &str) -> u32 {
    if ptr.is_null() || cap == 0 { return 0; }
    let bytes = s.as_bytes();
    let n = bytes.len().min(cap as usize - 1);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, n);
    *ptr.add(n) = 0;
    n as u32
}

unsafe fn engine(ptr: *mut Engine) -> &'static mut Engine { &mut *ptr }

// --- Lifecycle --------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn msplayer_new() -> *mut Engine {
    Box::into_raw(Box::new(Engine::new()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_free(ptr: *mut Engine) {
    if !ptr.is_null() { drop(Box::from_raw(ptr)); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_update(ptr: *mut Engine) {
    engine(ptr).update();
}

// --- Playback ---------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_play_at(ptr: *mut Engine, index: u32) {
    engine(ptr).play_song_at(index as usize);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_toggle_pause(ptr: *mut Engine) {
    engine(ptr).toggle_pause();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_restart(ptr: *mut Engine) {
    engine(ptr).restart_song();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_seek_to(ptr: *mut Engine, progress: f64) {
    engine(ptr).seek_to(progress);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_seek_forward(ptr: *mut Engine) {
    engine(ptr).seek_forward();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_seek_backward(ptr: *mut Engine) {
    engine(ptr).seek_backward();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_volume_up(ptr: *mut Engine) {
    engine(ptr).volume_up();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_volume_down(ptr: *mut Engine) {
    engine(ptr).volume_down();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_cycle_mode(ptr: *mut Engine) {
    engine(ptr).cycle_mode();
}

// --- Navigation -------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_next_album(ptr: *mut Engine) {
    engine(ptr).next_album();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_prev_album(ptr: *mut Engine) {
    engine(ptr).prev_album();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_next_song(ptr: *mut Engine) {
    engine(ptr).play_next_global();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_prev_song(ptr: *mut Engine) {
    engine(ptr).play_prev_global();
}

// --- Search -----------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_enter_search(ptr: *mut Engine) {
    engine(ptr).fetch_all_songs();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_search_input(
    ptr: *mut Engine,
    text: *const c_char,
    len: u32,
) {
    if text.is_null() { return; }
    let e = engine(ptr);
    let slice = std::slice::from_raw_parts(text as *const u8, len as usize);
    if let Ok(s) = std::str::from_utf8(slice) {
        e.fetch_all_songs();
        let _ = e.search_songs(s);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_search_result_count(ptr: *mut Engine) -> u32 {
    engine(ptr).all_songs.len() as u32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_search_result_at(
    ptr: *mut Engine,
    idx: u32,
    buf: *mut c_char,
    buf_size: u32,
) -> i32 {
    let e = engine(ptr);
    if let Some(song) = e.all_songs.get(idx as usize) {
        let s = format!("{} - {}", song.name, song.artists.join(", "));
        write_cstr(buf, buf_size, &s);
        0
    } else {
        -1
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_search_confirm(ptr: *mut Engine) {
    let e = engine(ptr);
    let cid = e.all_songs.first().map(|s| s.cid.clone());
    if let Some(cid) = cid {
        let _ = e.jump_to_song(&cid);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn msplayer_exit_search() {}

// --- Favorites --------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_is_loved(ptr: *mut Engine, cid: *const c_char) -> i32 {
    if cid.is_null() { return 0; }
    let cid_str = CStr::from_ptr(cid).to_string_lossy();
    engine(ptr).is_loved(&cid_str) as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_toggle_love(ptr: *mut Engine) {
    let e = engine(ptr);
    let name = e.current_song_name.clone();
    let cid = e.current_song_cid.clone();
    if let (Some(n), Some(c)) = (name, cid) {
        e.toggle_love(&c, &n, &[]);
    }
}

// --- Status Queries ---------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_is_playing(ptr: *mut Engine) -> i32 {
    engine(ptr).playing as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_is_buffering(ptr: *mut Engine) -> i32 {
    engine(ptr).buffering as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_is_global_mode(ptr: *mut Engine) -> i32 {
    engine(ptr).is_global_mode() as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_volume(ptr: *mut Engine) -> u32 {
    engine(ptr).volume as u32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_song_count(ptr: *mut Engine) -> u32 {
    engine(ptr).songs.len() as u32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_album_count(ptr: *mut Engine) -> u32 {
    engine(ptr).albums.len() as u32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_current_index(ptr: *mut Engine) -> u32 {
    engine(ptr).current_song_index.unwrap_or(0) as u32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_elapsed(ptr: *mut Engine) -> f64 {
    engine(ptr).elapsed_secs()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_duration(ptr: *mut Engine) -> f64 {
    engine(ptr).duration_secs().unwrap_or(0.0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_progress(ptr: *mut Engine) -> f64 {
    engine(ptr).progress.unwrap_or(0.0)
}

// --- Play Mode --------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_play_mode(ptr: *mut Engine) -> u32 {
    match engine(ptr).play_mode {
        PlayMode::AlbumList => 0,
        PlayMode::AlbumRandom => 1,
        PlayMode::GlobalList => 2,
        PlayMode::GlobalRandom => 3,
        PlayMode::Single => 4,
        PlayMode::LoveList => 5,
        PlayMode::LoveRandom => 6,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_set_play_mode(ptr: *mut Engine, mode: u32) {
    let e = engine(ptr);
    e.play_mode = match mode {
        0 => PlayMode::AlbumList,
        1 => PlayMode::AlbumRandom,
        2 => PlayMode::GlobalList,
        3 => PlayMode::GlobalRandom,
        4 => PlayMode::Single,
        5 => PlayMode::LoveList,
        6 => PlayMode::LoveRandom,
        _ => return,
    };
    e.rebuild_loved_list();
}

#[unsafe(no_mangle)]
pub extern "C" fn msplayer_mode_count() -> u32 { 7 }

// --- State Snapshot (JSON) --------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn msplayer_snapshot(
    ptr: *mut Engine,
    buf: *mut c_char,
    buf_size: u32,
) -> u32 {
    let e = engine(ptr);
    let json = serde_json::json!({
        "playing": e.playing,
        "volume": e.volume,
        "buffering": e.buffering,
        "elapsed": e.elapsed_secs(),
        "duration": e.duration_secs(),
        "progress": e.progress,
        "mode": msplayer_play_mode(ptr),
        "is_global": e.is_global_mode(),
        "album_name": e.album_name,
        "song_name": e.current_song_name,
        "song_count": e.songs.len(),
        "album_count": e.albums.len(),
    });
    write_cstr(buf, buf_size, &json.to_string())
}
