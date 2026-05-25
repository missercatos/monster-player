use crate::api::types::SongDetail;
use crate::error::{Error, Result};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::io::{BufReader, Cursor, Read};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct Player {
    sink: Sink,
    _stream: OutputStream,
    current: Arc<Mutex<Option<SongDetail>>>,
    start_time: Mutex<Option<Instant>>,
    paused_elapsed: Mutex<f64>,
    duration: Mutex<Option<f64>>,
}

impl Player {
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

    pub fn play_song(&self, song: &SongDetail) -> Result<()> {
        *self.current.lock().unwrap() = Some(song.to_owned());
        self.play_url(&song.source_url)
    }

    pub fn pause(&self) {
        if !self.sink.is_paused() {
            if let Some(t) = *self.start_time.lock().unwrap() {
                *self.paused_elapsed.lock().unwrap() += t.elapsed().as_secs_f64();
            }
            *self.start_time.lock().unwrap() = None;
        }
        self.sink.pause();
    }

    pub fn resume(&self) {
        self.sink.play();
        *self.start_time.lock().unwrap() = Some(Instant::now());
    }

    pub fn toggle(&self) {
        if self.sink.is_paused() {
            self.resume();
        } else {
            self.pause();
        }
    }

    pub fn stop(&self) {
        self.sink.stop();
        *self.start_time.lock().unwrap() = None;
        *self.paused_elapsed.lock().unwrap() = 0.0;
    }

    pub fn set_volume(&self, vol: f32) {
        self.sink.set_volume(vol);
    }

    pub fn elapsed(&self) -> f64 {
        let base = *self.paused_elapsed.lock().unwrap();
        if let Some(t) = *self.start_time.lock().unwrap() {
            base + t.elapsed().as_secs_f64()
        } else {
            base
        }
    }

    pub fn duration(&self) -> Option<f64> {
        *self.duration.lock().unwrap()
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    pub fn current_song(&self) -> Option<SongDetail> {
        self.current.lock().unwrap().to_owned()
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new().expect("failed to create audio player")
    }
}
