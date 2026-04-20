use crate::app::state::PlayMode;
use crate::playback::local_player::LocalPlayer;
use crate::playback::mpris_client::MprisClient;
use crate::playback::streaming_player::StreamingPlayer;

pub struct ModeManager {
    pub local: LocalPlayer,
    pub mpris: MprisClient,
    pub streaming: StreamingPlayer,
}

impl ModeManager {
    pub fn new() -> Self {
        Self {
            local: LocalPlayer::new(),
            mpris: MprisClient::new(),
            streaming: StreamingPlayer::new().expect("failed to create streaming player"),
        }
    }

    pub fn pause_other(&mut self, target: PlayMode) {
        match target {
            PlayMode::LocalPlayback => {
                let _ = self.mpris.pause();
                self.streaming.pause();
            }
            PlayMode::SystemMonitor => {
                let _ = self.local.pause();
                self.streaming.pause();
            }
            PlayMode::Streaming => {
                let _ = self.local.pause();
                let _ = self.mpris.pause();
            }
            PlayMode::Idle => {
                let _ = self.local.pause();
                self.streaming.pause();
            }
        }
    }
}
