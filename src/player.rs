use crate::api::types::SongDetail;
use crate::error::{Error, Result};
use rodio::{Decoder, OutputStream, Sink, Source};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::io::{BufReader, Cursor, Read};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// PCM 采样拦截器：在 Source 迭代时将采样写入环形缓冲区供 FFT 线程读取
struct SpectrumTap {
    inner: Box<dyn Source<Item = f32> + Send>,
    buffer: Arc<Mutex<Vec<f32>>>,
}

impl Iterator for SpectrumTap {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.inner.next()?;
        if let Ok(mut buf) = self.buffer.lock() {
            if buf.len() < 4096 {
                buf.push(sample);
            }
        }
        Some(sample)
    }
}

impl Source for SpectrumTap {
    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.total_duration()
    }

    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
}

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
    /// FFT 采样缓冲区（SpectrumTap 写入，FFT 线程读取）
    spectrum_buffer: Arc<Mutex<Vec<f32>>>,
    /// FFT 计算结果（8 个频段，0.0-1.0）
    spectrum_result: Arc<Mutex<[f32; 8]>>,
}

impl Player {
    /// 创建新的播放器实例，初始化音频输出流与 Sink。
    pub fn new() -> Result<Self> {
        let (stream, handle) = rodio::OutputStream::try_default()
            .map_err(|e| Error::Audio(e.to_string()))?;
        let sink = Sink::try_new(&handle).map_err(|e| Error::Audio(e.to_string()))?;
        let spectrum_buffer = Arc::new(Mutex::new(Vec::new()));
        let spectrum_result = Arc::new(Mutex::new([0.0f32; 8]));

        // 启动 FFT 后台线程
        let buf_clone = spectrum_buffer.clone();
        let res_clone = spectrum_result.clone();
        std::thread::spawn(move || {
            fft_worker(buf_clone, res_clone);
        });

        Ok(Self {
            sink,
            _stream: stream,
            current: Arc::new(Mutex::new(None)),
            start_time: Mutex::new(None),
            paused_elapsed: Mutex::new(0.0),
            duration: Mutex::new(None),
            spectrum_buffer,
            spectrum_result,
        })
    }

    /// 下载 WAV 音频文件并送入播放队列。
    pub fn play_url(&self, url: &str) -> Result<()> {
        let mut resp = ureq::get(url).call()?;
        let mut data = Vec::new();
        resp.body_mut().as_reader().read_to_end(&mut data)?;

        let cursor = Cursor::new(data);
        let source = Decoder::new(BufReader::new(cursor))
            .map_err(|e| Error::Audio(format!("decode: {e}")))?;
        *self.duration.lock().unwrap() = source.total_duration().map(|d| d.as_secs_f64());

        let source = source.convert_samples::<f32>();
        let tap = SpectrumTap {
            inner: Box::new(source),
            buffer: self.spectrum_buffer.clone(),
        };

        self.sink.stop();
        // 清空采样缓冲
        if let Ok(mut buf) = self.spectrum_buffer.lock() {
            buf.clear();
        }
        self.sink.append(tap);
        *self.start_time.lock().unwrap() = Some(Instant::now());
        *self.paused_elapsed.lock().unwrap() = 0.0;
        Ok(())
    }

    /// 将内存中的 WAV 字节数据解码并播放（缓冲播放入口）。
    pub fn play_bytes(&self, data: Vec<u8>) -> Result<()> {
        let cursor = Cursor::new(data);
        let source = Decoder::new(BufReader::new(cursor))
            .map_err(|e| Error::Audio(format!("decode: {e}")))?;
        *self.duration.lock().unwrap() = source.total_duration().map(|d| d.as_secs_f64());

        let source = source.convert_samples::<f32>();
        let tap = SpectrumTap {
            inner: Box::new(source),
            buffer: self.spectrum_buffer.clone(),
        };

        self.sink.stop();
        if let Ok(mut buf) = self.spectrum_buffer.lock() {
            buf.clear();
        }
        self.sink.append(tap);
        *self.start_time.lock().unwrap() = Some(Instant::now());
        *self.paused_elapsed.lock().unwrap() = 0.0;
        Ok(())
    }

    /// 从指定秒数位置开始播放（支持进度跳转）。
    pub fn play_bytes_at(&self, data: Vec<u8>, start_secs: f64) -> Result<()> {
        use std::time::Duration;
        let cursor = Cursor::new(data);
        let source = Decoder::new(BufReader::new(cursor))
            .map_err(|e| Error::Audio(format!("decode: {e}")))?;
        *self.duration.lock().unwrap() = source.total_duration().map(|d| d.as_secs_f64());

        let source = source.skip_duration(Duration::from_secs_f64(start_secs));
        let source = source.convert_samples::<f32>();

        let tap = SpectrumTap {
            inner: Box::new(source),
            buffer: self.spectrum_buffer.clone(),
        };

        self.sink.stop();
        if let Ok(mut buf) = self.spectrum_buffer.lock() {
            buf.clear();
        }
        self.sink.append(tap);
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
    pub fn elapsed(&self) -> f64 {
        let base = self.paused_elapsed.lock().map_or(0.0, |v| *v);
        let start = self.start_time.lock().map_or(None, |v| *v);
        if let Some(t) = start {
            base + t.elapsed().as_secs_f64()
        } else {
            base
        }
    }

    /// 返回当前曲目总时长（秒），WAV 解码时捕获。
    pub fn duration(&self) -> Option<f64> {
        self.duration.lock().map_or(None, |v| *v)
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

    /// 返回当前音频频谱（8 个频段，0.0-1.0）
    pub fn spectrum(&self) -> [f32; 8] {
        *self.spectrum_result.lock().unwrap()
    }
}

/// FFT 后台工作线程：从环形缓冲读取采样，计算频谱，写入 spectrum_result
fn fft_worker(buffer: Arc<Mutex<Vec<f32>>>, result: Arc<Mutex<[f32; 8]>>) {
    let fft_size = 2048usize;
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(30));

        // 读取采样
        let samples = {
            let mut buf = match buffer.lock() {
                Ok(b) => b,
                Err(_) => continue,
            };
            if buf.len() < fft_size {
                continue;
            }
            let start = buf.len() - fft_size;
            let data: Vec<f32> = buf[start..].to_vec();
            buf.drain(0..start);
            data
        };

        // 应用汉宁窗
        let window: Vec<f32> = (0..fft_size)
            .map(|i| {
                let n = i as f32;
                let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * n / (fft_size as f32)).cos());
                samples[i] * w
            })
            .collect();

        // FFT 变换
        let mut buffer_f: Vec<Complex<f32>> = window
            .iter()
            .map(|&s| Complex { re: s, im: 0.0 })
            .collect();
        fft.process(&mut buffer_f);

        // 计算 8 个频段的幅度
        let bin_count = fft_size / 2;
        let bands = 8;
        let bins_per_band = bin_count / bands;
        let mut spectrum = [0.0f32; 8];

        for band in 0..bands {
            let start_bin = band * bins_per_band;
            let end_bin = (start_bin + bins_per_band).min(bin_count);
            let mut energy = 0.0f32;
            for k in start_bin..end_bin {
                let mag = (buffer_f[k].re * buffer_f[k].re + buffer_f[k].im * buffer_f[k].im).sqrt();
                energy += mag;
            }
            energy /= bins_per_band as f32;
            // 归一化到 0.0-1.0
            spectrum[band] = (energy * 3.0).min(1.0);
        }

        if let Ok(mut res) = result.lock() {
            *res = spectrum;
        }
    }
}

/// 默认实现：调用 `Player::new()` 并期望成功。
impl Default for Player {
    fn default() -> Self {
        Self::new().expect("failed to create audio player")
    }
}
