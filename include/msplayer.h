/* msplayer.h — C-compatible FFI bindings for monster-player kernel */

#ifndef MSPLAYER_H
#define MSPLAYER_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* --- Opaque engine handle --- */
typedef struct MsplayerEngine MsplayerEngine;

/* --- Lifecycle --- */
MsplayerEngine* msplayer_new(void);
void            msplayer_free(MsplayerEngine* engine);
void            msplayer_update(MsplayerEngine* engine);

/* --- Playback --- */
void msplayer_play_at(MsplayerEngine* engine, uint32_t index);
void msplayer_toggle_pause(MsplayerEngine* engine);
void msplayer_restart(MsplayerEngine* engine);
void msplayer_seek_to(MsplayerEngine* engine, double progress);
void msplayer_seek_forward(MsplayerEngine* engine);
void msplayer_seek_backward(MsplayerEngine* engine);
void msplayer_volume_up(MsplayerEngine* engine);
void msplayer_volume_down(MsplayerEngine* engine);
void msplayer_cycle_mode(MsplayerEngine* engine);

/* --- Navigation --- */
void msplayer_next_album(MsplayerEngine* engine);
void msplayer_prev_album(MsplayerEngine* engine);
void msplayer_next_song(MsplayerEngine* engine);
void msplayer_prev_song(MsplayerEngine* engine);

/* --- Search --- */
void msplayer_enter_search(MsplayerEngine* engine);
void msplayer_search_input(MsplayerEngine* engine, const char* text, uint32_t len);
uint32_t msplayer_search_result_count(MsplayerEngine* engine);
int32_t  msplayer_search_result_at(MsplayerEngine* engine, uint32_t idx, char* buf, uint32_t buf_size);
void msplayer_search_confirm(MsplayerEngine* engine);
void msplayer_exit_search(MsplayerEngine* engine);

/* --- Favorites --- */
int32_t msplayer_is_loved(MsplayerEngine* engine, const char* cid);
void    msplayer_toggle_love(MsplayerEngine* engine);

/* --- Status Queries --- */
int32_t  msplayer_is_playing(MsplayerEngine* engine);
int32_t  msplayer_is_buffering(MsplayerEngine* engine);
int32_t  msplayer_is_global_mode(MsplayerEngine* engine);
uint32_t msplayer_volume(MsplayerEngine* engine);
uint32_t msplayer_song_count(MsplayerEngine* engine);
uint32_t msplayer_album_count(MsplayerEngine* engine);
uint32_t msplayer_current_index(MsplayerEngine* engine);
double   msplayer_elapsed(MsplayerEngine* engine);
double   msplayer_duration(MsplayerEngine* engine);
double   msplayer_progress(MsplayerEngine* engine);

/* --- State Snapshot (returns JSON into buf, returns bytes written) --- */
uint32_t msplayer_snapshot(MsplayerEngine* engine, char* buf, uint32_t buf_size);

/* --- Play Mode --- */
uint32_t msplayer_play_mode(MsplayerEngine* engine);
void     msplayer_set_play_mode(MsplayerEngine* engine, uint32_t mode);
uint32_t msplayer_mode_count(void);
/* mode values: 0=AlbumList 1=AlbumRandom 2=GlobalList 3=GlobalRandom 4=Single 5=LoveList 6=LoveRandom */

#ifdef __cplusplus
}
#endif

#endif /* MSPLAYER_H */
