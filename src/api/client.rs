use crate::api::types::*;
use crate::error::Result;

pub struct Client {
    base_url: String,
    agent: ureq::Agent,
}

impl Client {
    pub fn new() -> Self {
        Self {
            base_url: "https://monster-siren.hypergryph.com".into(),
            agent: ureq::Agent::new_with_defaults(),
        }
    }

    pub fn albums(&self) -> Result<Vec<Album>> {
        let resp: ApiResponse<Vec<Album>> =
            self.agent.get(&format!("{}/api/albums", self.base_url)).call()?.body_mut().read_json()?;
        Ok(resp.data)
    }

    pub fn album_detail(&self, cid: &str) -> Result<AlbumDetail> {
        let resp: ApiResponse<AlbumDetail> = self
            .agent
            .get(&format!("{}/api/album/{}/detail", self.base_url, cid))
            .call()?
            .body_mut()
            .read_json()?;
        Ok(resp.data)
    }

    pub fn songs(&self) -> Result<Vec<Song>> {
        let resp: ApiResponse<SongsResponse> =
            self.agent.get(&format!("{}/api/songs", self.base_url)).call()?.body_mut().read_json()?;
        Ok(resp.data.list)
    }

    pub fn song_detail(&self, cid: &str) -> Result<SongDetail> {
        let resp: ApiResponse<SongDetail> =
            self.agent.get(&format!("{}/api/song/{}", self.base_url, cid)).call()?.body_mut().read_json()?;
        Ok(resp.data)
    }

    pub fn news(&self) -> Result<Vec<NewsItem>> {
        let resp: ApiResponse<NewsResponse> =
            self.agent.get(&format!("{}/api/news", self.base_url)).call()?.body_mut().read_json()?;
        Ok(resp.data.list)
    }

    pub fn search(&self, keyword: &str) -> Result<SearchResponse> {
        let resp: ApiResponse<SearchResponse> = self
            .agent
            .get(&format!("{}/api/search", self.base_url))
            .query("keyword", keyword)
            .call()?
            .body_mut()
            .read_json()?;
        Ok(resp.data)
    }
}
