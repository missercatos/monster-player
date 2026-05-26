use crate::api::types::SongDetail;
use crate::error::{Error, Result};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::io::{BufReader, Cursor, Read};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 音频播放器，封装 rodio 后端，管理播放状态与歌曲信息。
pub struct Player {
    /// rodio 音频输出槽，控制播放/暂停/音量/停止
    sink: Sink,
    /// 持有输出流以维持音频设备生命周期（未直接读取）
    _stream: OutputStream,
    /// 当前播放歌曲的元信息，在线程间共享
    current: Arc<Mutex<Option<SongDetail>>>,
    /// 当前播放段的起始时刻，用于计算实时进度
    start_time: Mutex<Option<Instant>>,
    /// 暂停前已累积的播放秒数
    paused_elapsed: Mutex<f64>,
    /// 当前曲目总时长（秒），从 WAV 解码时捕获
    duration: Mutex<Option<f64>>,
}

impl Player {
    /// 创建新的播放器实例，初始化音频输出流与 Sink。
    pub fn new() -> Result<Self> {
        let (stream, handle) = rodio::OutputStream::try_default()
            .map_err(|e| Error::Audio(e.to_string()))?;
        let sink = Sink::try_new(&handle).map_err(|e| Error::Audio(e.to_string()))?;
        Ok(Self {
            sink,
            _stream: stream,
            current: Arc::new(Mutex::new(None)),
            start_time: Mutex::new(None),
            paused_elapsed: Mutex::new(0.0),
            duration: Mutex::new(None),
        })
    }

    /// 下载 WAV 音频文件并送入播放队列。
    /// 会先停止当前播放，解码后记录总时长，重置播放进度。
    pub fn play_url(&self, url: &str) -> Result<()> {
        let mut resp = ureq::get(url).call()?;
        let mut data = Vec::new();
        resp.body_mut().as_reader().read_to_end(&mut data)?;

        let cursor = Cursor::new(data);
        let source = Decoder::new(BufReader::new(cursor))
            .map_err(|e| Error::Audio(format!("decode: {e}")))?;
        *self.duration.lock().unwrap() = source.total_duration().map(|d| d.as_secs_f64());

        self.sink.stop();
        self.sink.append(source);
        *self.start_time.lock().unwrap() = Some(Instant::now());
        *self.paused_elapsed.lock().unwrap() = 0.0;
        Ok(())
    }

    /// 将内存中的 WAV 字节数据解码并播放（缓冲播放入口）。
    /// 会先停止当前播放，解码后记录总时长，重置播放进度。
    pub fn play_bytes(&self, data: Vec<u8>) -> Result<()> {
        let cursor = Cursor::new(data);
        let source = Decoder::new(BufReader::new(cursor))
            .map_err(|e| Error::Audio(format!("decode: {e}")))?;
        *self.duration.lock().unwrap() = source.total_duration().map(|d| d.as_secs_f64());

        self.sink.stop();
        self.sink.append(source);
        *self.start_time.lock().unwrap() = Some(Instant::now());
        *self.paused_elapsed.lock().unwrap() = 0.0;
        Ok(())
    }

    /// 从指定秒数位置开始播放（支持进度跳转）。
    /// 跳过音频前 `start_secs` 秒的内容，其余逻辑与 `play_bytes` 一致。
    pub fn play_bytes_at(&self, data: Vec<u8>, start_secs: f64) -> Result<()> {
        use std::time::Duration;
        let cursor = Cursor::new(data);
        let source = Decoder::new(BufReader::new(cursor))
            .map_err(|e| Error::Audio(format!("decode: {e}")))?;
        *self.duration.lock().unwrap() = source.total_duration().map(|d| d.as_secs_f64());

        let source = source.skip_duration(Duration::from_secs_f64(start_secs));

        self.sink.stop();
        self.sink.append(source);
        *self.start_time.lock().unwrap() = Some(Instant::now());
        *self.paused_elapsed.lock().unwrap() = start_secs;
        Ok(())
    }

    /// 播放歌曲：先记录当前歌曲信息，再通过 `play_url` 下载并播放。
    pub fn play_song(&self, song: &SongDetail) -> Result<()> {
        *self.current.lock().unwrap() = Some(song.to_owned());
        self.play_url(&song.source_url)
    }

    /// 暂停播放，记录当前已播放时间。
    /// 将已流逝的秒数累加到 `paused_elapsed`，以便恢复时续接。
    pub fn pause(&self) {
        if !self.sink.is_paused() {
            if let Some(t) = *self.start_time.lock().unwrap() {
                *self.paused_elapsed.lock().unwrap() += t.elapsed().as_secs_f64();
            }
            *self.start_time.lock().unwrap() = None;
        }
        self.sink.pause();
    }

    /// 恢复播放：重新标记起始时刻，Sink 恢复输出。
    pub fn resume(&self) {
        self.sink.play();
        *self.start_time.lock().unwrap() = Some(Instant::now());
    }

    /// 切换 暂停/播放 状态。
    pub fn toggle(&self) {
        if self.sink.is_paused() {
            self.resume();
        } else {
            self.pause();
        }
    }

    /// 停止播放，重置计时。
    /// 清空 Sink 队列，清除起始时刻和已累积时间。
    pub fn stop(&self) {
        self.sink.stop();
        *self.start_time.lock().unwrap() = None;
        *self.paused_elapsed.lock().unwrap() = 0.0;
    }

    /// 设置输出音量 0.0~1.0，直接控制 rodio Sink。
    pub fn set_volume(&self, vol: f32) {
        self.sink.set_volume(vol);
    }

    /// 返回当前播放进度（秒），包含暂停累加逻辑。
    /// 正在播放时 = 暂停前累计 + 当前段已流逝；暂停时 = 暂停前累计。
    pub fn elapsed(&self) -> f64 {
        let base = *self.paused_elapsed.lock().unwrap();
        if let Some(t) = *self.start_time.lock().unwrap() {
            base + t.elapsed().as_secs_f64()
        } else {
            base
        }
    }

    /// 返回当前曲目总时长（秒），WAV 解码时捕获。
    /// 无歌曲返回 `None`。
    pub fn duration(&self) -> Option<f64> {
        *self.duration.lock().unwrap()
    }

    /// 返回当前是否处于暂停状态。
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    /// 返回当前播放队列是否为空（无待播放数据）。
    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    /// 返回当前播放歌曲的元信息。
    pub fn current_song(&self) -> Option<SongDetail> {
        self.current.lock().unwrap().to_owned()
    }
}

/// 默认实现：调用 `Player::new()` 并期望成功。
impl Default for Player {
    fn default() -> Self {
        Self::new().expect("failed to create audio player")
    }
}
