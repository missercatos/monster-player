use std::io::Read;

const CHARS: &[u8] = b" .:-=+*#%@";
const CHARS_LEN: f32 = 9.0;

pub fn cover_to_ascii(cover_url: &str, cols: u32, rows: u32) -> Option<Vec<String>> {
    let mut resp = ureq::get(cover_url).call().ok()?;
    let mut data = Vec::new();
    resp.body_mut().as_reader().read_to_end(&mut data).ok()?;

    let img = image::load_from_memory(&data).ok()?;
    let gray = img.grayscale();
    let resized =
        image::imageops::resize(&gray, cols, rows, image::imageops::FilterType::Lanczos3);

    let mut lines = Vec::with_capacity(rows as usize);
    for y in 0..rows {
        let mut line = String::with_capacity(cols as usize);
        for x in 0..cols {
            let pixel = resized.get_pixel(x, y).0[0];
            let idx = (pixel as f32 / 255.0 * CHARS_LEN) as usize;
            line.push(CHARS[idx] as char);
        }
        lines.push(line);
    }
    Some(lines)
}
