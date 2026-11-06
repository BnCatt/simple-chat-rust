//! The player's own icons: PNGs in the mod's data folder
//! (`horizon-mods/.horizon-mod-data/simple_chat_rust/icons/<name>.png`, the names of
//! [`Icon::file`]) drawn instead of the compiled-in ones, pixel by pixel. The
//! mod ships none: a player can put any icons there, for example the ones of
//! a resource pack they own. Any square size works; a file that can't be read
//! is named in the log once and the compiled-in icon is drawn.
use crate::skin::Icon;
use crate::ui::Draw;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

/// An icon's pixels, ARGB, row by row, and its side.
struct Pixels {
    side: usize,
    argb: Vec<u32>,
}

static LOADED: LazyLock<Mutex<HashMap<Icon, Option<Pixels>>>> = LazyLock::new(Mutex::default);

fn load(dir: &Path, i: Icon) -> Option<Pixels> {
    let path = dir.join("icons").join(format!("{}.png", i.file()));
    if !path.is_file() {
        return None;
    }
    match decode(&path) {
        Ok(p) => {
            log::info!(target: "horizon_voicechat", "Voice Chat: icon {} from {}", i.file(), path.display());
            Some(p)
        }
        Err(e) => {
            log::warn!(target: "horizon_voicechat", "Voice Chat: icon {} not read: {e}", path.display());
            None
        }
    }
}

fn decode(path: &Path) -> Result<Pixels, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size().ok_or("too large")?];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    let (w, h) = (info.width as usize, info.height as usize);
    if w == 0 || w != h || w > 256 {
        return Err(format!("{w}x{h}: not a square of at most 256"));
    }
    let channels = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        png::ColorType::Indexed => return Err("indexed colours left after expanding".into()),
    };
    let line = info.line_size;
    let mut argb = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let p = &buf[y * line + x * channels..][..channels];
            let (r, g, b, a) = match channels {
                4 => (p[0], p[1], p[2], p[3]),
                3 => (p[0], p[1], p[2], 255),
                2 => (p[0], p[0], p[0], p[1]),
                _ => (p[0], p[0], p[0], 255),
            };
            argb.push(u32::from(a) << 24 | u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b));
        }
    }
    Ok(Pixels { side: w, argb })
}

/// Part `src` (in 16x16 icon units) of the player's icon `i` drawn into `dst`
/// with fills; none when there is no such file.
pub fn fills(i: Icon, src: [f32; 4], dst: [f32; 4]) -> Option<Vec<Draw>> {
    let dir = crate::dylib::data_dir()?;
    let mut loaded = LOADED.lock().unwrap_or_else(|e| e.into_inner());
    let p = loaded.entry(i).or_insert_with(|| load(dir, i)).as_ref()?;
    Some(pixel_fills(p.side, &p.argb, src, dst))
}

fn pixel_fills(side: usize, argb: &[u32], src: [f32; 4], dst: [f32; 4]) -> Vec<Draw> {
    let k = side as f32 / 16.0;
    let (sx, sy) = (dst[2] / (src[2] * k).max(1.0), dst[3] / (src[3] * k).max(1.0));
    let (x0, y0) = ((src[0] * k).max(0.0) as usize, (src[1] * k).max(0.0) as usize);
    let x1 = (((src[0] + src[2]) * k) as usize).min(side);
    let y1 = (((src[1] + src[3]) * k) as usize).min(side);
    let mut out = Vec::new();
    for y in y0..y1 {
        let row = &argb[y * side..][..side];
        let mut x = x0;
        while x < x1 {
            let c = row[x];
            let mut end = x + 1;
            while end < x1 && row[end] == c {
                end += 1;
            }
            if c >> 24 != 0 {
                let px = dst[0] + (x - x0) as f32 * sx;
                let py = dst[1] + (y - y0) as f32 * sy;
                out.push(Draw::Fill([px, py, (end - x) as f32 * sx, sy], c));
            }
            x = end;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_32_pixel_icon_fills_a_16_unit_place_at_half_a_unit_a_pixel() {
        // Left half opaque red, right half clear.
        let side = 32;
        let argb: Vec<u32> = (0..side * side).map(|i| if i % side < 16 { 0xFFFF_0000 } else { 0 }).collect();
        let f = pixel_fills(side, &argb, [0.0, 0.0, 16.0, 16.0], [10.0, 20.0, 16.0, 16.0]);
        assert_eq!(f.len(), 32, "one run a row");
        assert!(f.iter().all(|d| matches!(d, Draw::Fill([10.0, _, 8.0, 0.5], 0xFFFF_0000))));
    }

    #[test]
    fn a_part_of_the_icon_is_cut_out() {
        let side = 16;
        let argb = vec![0xFF00_FF00; side * side];
        let f = pixel_fills(side, &argb, [4.0, 4.0, 8.0, 8.0], [0.0, 0.0, 8.0, 8.0]);
        assert_eq!(f.len(), 8);
        assert!(f.iter().all(|d| matches!(d, Draw::Fill([0.0, _, 8.0, 1.0], _))));
    }
}
