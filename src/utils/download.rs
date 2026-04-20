use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

pub fn download_to_temp_file(url: &str) -> Result<NamedTempFile> {
    let resp = ureq::get(url)
        .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .call()
        .map_err(|e| anyhow!("HTTP请求失败: {}", e))?;
    
    if resp.status() != 200 {
        return Err(anyhow!("HTTP错误: {}", resp.status()));
    }

    let mut reader = resp.into_reader();
    let mut temp_file = tempfile::NamedTempFile::new()?;
    std::io::copy(&mut reader, &mut temp_file)?;
    Ok(temp_file)
}

pub fn download_to_file(url: &str, path: PathBuf) -> Result<()> {
    let resp = ureq::get(url)
        .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .call()
        .map_err(|e| anyhow!("HTTP请求失败: {}", e))?;
    
    if resp.status() != 200 {
        return Err(anyhow!("HTTP错误: {}", resp.status()));
    }

    let mut reader = resp.into_reader();
    let mut file = File::create(path)?;
    std::io::copy(&mut reader, &mut file)?;
    Ok(())
}