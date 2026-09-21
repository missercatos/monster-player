//! 塞壬唱片 API 参考 CLI。
//!
//! 拉取真实响应并输出 JSON fixture，供跨语言契约测试使用。
//! 也可通过 `--base-url` 指向未来的 Go 网关实现，做响应对照验证。

mod client;
mod error;
mod types;

use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::client::Client;
use crate::error::Result;

/// 塞壬唱片 API 参考 CLI
#[derive(Parser)]
#[command(
    name = "siren",
    version,
    about = "Monster Siren Records API reference CLI (fixtures & contract validation)"
)]
struct Cli {
    /// API base URL; point it to a gateway/proxy implementation to compare responses
    #[arg(long, global = true, default_value = client::DEFAULT_BASE_URL)]
    base_url: String,

    /// Write output to a file instead of stdout
    #[arg(short, long, global = true, value_name = "FILE")]
    output: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// GET /api/albums — all albums
    Albums,
    /// GET /api/album/{cid}/detail — album detail with song list
    Album {
        /// Album cid
        cid: String,
    },
    /// GET /api/songs — all songs
    Songs,
    /// GET /api/song/{cid} — song detail (audio source url / lyric url)
    Song {
        /// Song cid
        cid: String,
    },
    /// GET /api/news — news list
    News,
    /// GET /api/search?keyword= — search albums and news
    Search {
        /// Search keyword
        keyword: String,
    },
    /// Download the LRC lyrics for a song
    Lyrics {
        /// Song cid
        cid: String,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let client = Client::with_base_url(cli.base_url.as_str());

    let content = match &cli.command {
        Command::Albums => pretty(&client.albums()?)?,
        Command::Album { cid } => pretty(&client.album_detail(cid)?)?,
        Command::Songs => pretty(&client.songs()?)?,
        Command::Song { cid } => pretty(&client.song_detail(cid)?)?,
        Command::News => pretty(&client.news()?)?,
        Command::Search { keyword } => pretty(&client.search(keyword)?)?,
        Command::Lyrics { cid } => client.lyrics(cid)?,
    };

    match &cli.output {
        Some(path) => std::fs::write(path, content)?,
        None => {
            let mut stdout = std::io::stdout();
            stdout.write_all(content.as_bytes())?;
            stdout.write_all(b"\n")?;
        }
    }
    Ok(())
}

/// 序列化为缩进 JSON。
fn pretty<T: serde::Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)?)
}
