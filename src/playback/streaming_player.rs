use anyhow::{anyhow, Result};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::io::Read;
use std::time::Duration;

pub struct StreamingPlayer {
    sink: Sink,
    _stream: OutputStream,
    current_url: Option<String>,
}

impl StreamingPlayer {
    pub fn new() -> Result<Self> {
        let (stream, handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&handle)?;
        Ok(Self {
            sink,
            _stream: stream,
            current_url: None,
        })
    }

    pub fn play_url(&mut self, url: &str) -> Result<()> {
        self.stop();
        
        // 使用 ureq 获取 HTTP 流
        let resp = ureq::get(url)
            .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
            .call()
            .map_err(|e| anyhow!("HTTP请求失败: {}", e))?;
        
        if resp.status() != 200 {
            return Err(anyhow!("HTTP错误: {}", resp.status()));
        }

        // 将响应体转换为 Read 对象
        let reader = resp.into_reader();
        
        // 尝试使用 rodio 的 Decoder 解码流
        // Decoder 需要 Seek，但 HTTP 流不支持 Seek。
        // 对于不支持 Seek 的流，我们可以尝试使用 Decoder::new_mp3 或类似的函数。
        // 但 rodio 的 Decoder::new 内部使用 symphonia，它需要 Seek。
        // 作为临时方案，我们将整个响应读入内存，然后解码。
        // TODO: 实现真正的流式解码
        let mut data = Vec::new();
        let mut reader = reader;
        reader.read_to_end(&mut data)?;
        
        let cursor = std::io::Cursor::new(data);
        let source = Decoder::new(cursor).map_err(|e| anyhow!("解码失败: {}", e))?;
        
        self.sink.append(source);
        self.sink.play();
        self.current_url = Some(url.to_string());
        Ok(())
    }
    
    pub fn stop(&mut self) {
        self.sink.stop();
        self.current_url = None;
    }
    
    pub fn pause(&mut self) {
        self.sink.pause();
    }
    
    pub fn play(&mut self) {
        self.sink.play();
    }
    
    pub fn toggle_play_pause(&mut self) {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }
    
    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume);
    }
    
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }
    
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
    
    pub fn empty(&self) -> bool {
        self.sink.empty()
    }
}