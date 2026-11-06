//! Labels above players in the world: which voice chat group each is in, as
//! the camera the last frame drew from sees them. The projection is the
//! waypoints mod's.

/// The camera: its position, yaw and pitch in degrees, vertical field of view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub pos: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// `Entity.calculateViewVector`: 0 yaw faces south, positive pitch down.
fn forward(yaw: f32, pitch: f32) -> [f64; 3] {
    let (y, p) = (f64::from(yaw).to_radians(), f64::from(pitch).to_radians());
    [-y.sin() * p.cos(), -p.sin(), y.cos() * p.cos()]
}

/// The camera's forward, right and up. Right comes from the yaw alone, so
/// looking straight up or down still has one.
fn basis(yaw: f32, pitch: f32) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let f = forward(yaw, pitch);
    let y = f64::from(yaw).to_radians();
    let r = [-y.cos(), 0.0, -y.sin()];
    (f, r, cross(r, f))
}

/// `point` on a `size` screen, as the world's perspective projection draws
/// it; None behind the camera.
pub fn project(camera: &Camera, point: [f64; 3], size: [f32; 2]) -> Option<[f32; 2]> {
    let (f, r, u) = basis(camera.yaw, camera.pitch);
    let d = sub(point, camera.pos);
    let z = dot(d, f);
    if z < 0.05 || size[1] <= 0.0 {
        return None;
    }
    let t = (f64::from(camera.fov).to_radians() / 2.0).tan();
    let aspect = f64::from(size[0]) / f64::from(size[1]);
    let nx = dot(d, r) / (z * t * aspect);
    let ny = dot(d, u) / (z * t);
    Some([((nx + 1.0) / 2.0 * f64::from(size[0])) as f32, ((1.0 - ny) / 2.0 * f64::from(size[1])) as f32])
}

/// Labels show for players this close.
pub const RANGE: f64 = 64.0;
/// Above the eyes, clear of the name tag.
const ABOVE_EYES: f64 = 0.95;

/// A label's text centred at `at` (GUI pixels), in `rgb`.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    pub text: String,
    pub at: [f32; 2],
    pub rgb: u32,
}

/// The labels for `players` (eye position, group name, whether it is the
/// player's own group), the farthest first so the nearest draws on top.
pub fn place(camera: &Camera, size: [f32; 2], players: &[([f64; 3], String, bool)]) -> Vec<Label> {
    let mut placed: Vec<(f64, Label)> = players
        .iter()
        .filter_map(|(eye, group, ours)| {
            let point = [eye[0], eye[1] + ABOVE_EYES, eye[2]];
            let d = sub(point, camera.pos);
            let distance = dot(d, d).sqrt();
            if distance > RANGE {
                return None;
            }
            let at = project(camera, point, size)?;
            let on_screen = (0.0..=size[0]).contains(&at[0]) && (0.0..=size[1]).contains(&at[1]);
            on_screen.then(|| {
                (distance, Label { text: format!("[{group}]"), at, rgb: if *ours { 0x55FF55 } else { 0xFFFF55 } })
            })
        })
        .collect();
    placed.sort_by(|a, b| b.0.total_cmp(&a.0));
    placed.into_iter().map(|(_, l)| l).collect()
}

/// The speaker over a talking player's head: this far above the eyes (over
/// the name tag, which spans about 0.68 to 0.9), higher still when the
/// group name is shown there; and this big in the world, in blocks.
pub const TALK_ABOVE_EYES: f64 = 1.1;
pub const TALK_ABOVE_LABEL: f64 = 0.3;
const TALK_SIZE: f64 = 0.35;

/// The speaker icons for `talking` players (where the icon goes: eye
/// position, and whether a group name is shown above them; whispering), as
/// Simple Voice Chat shows one over each player who talks: centred over the
/// head, as big as 0.35 blocks there (4 to 16 GUI pixels), the farthest
/// first so the nearest draws on top.
pub fn talk_icons(camera: &Camera, size: [f32; 2], talking: &[([f64; 3], bool, bool)]) -> Vec<crate::ui::Draw> {
    use crate::skin::{Icon, icon};
    let mut placed: Vec<(f64, crate::ui::Draw)> = talking
        .iter()
        .filter_map(|(eye, labelled, whisper)| {
            let above = TALK_ABOVE_EYES + if *labelled { TALK_ABOVE_LABEL } else { 0.0 };
            let point = [eye[0], eye[1] + above, eye[2]];
            let d = sub(point, camera.pos);
            let distance = dot(d, d).sqrt();
            if distance > RANGE {
                return None;
            }
            let at = project(camera, point, size)?;
            let top = project(camera, [point[0], point[1] + TALK_SIZE, point[2]], size)?;
            let side = (at[1] - top[1]).abs().clamp(4.0, 16.0).round();
            let half = side / 2.0;
            let on_screen = (-half..=size[0] + half).contains(&at[0]) && (-half..=size[1] + half).contains(&at[1]);
            let which = if *whisper { Icon::Whisper } else { Icon::Speaker };
            on_screen.then(|| (distance, icon(which, [at[0] - half, at[1] - half, side, side])))
        })
        .collect();
    placed.sort_by(|a, b| b.0.total_cmp(&a.0));
    placed.into_iter().map(|(_, d)| d).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::Icon;
    use crate::ui::Draw;

    #[test]
    fn a_speaker_over_whoever_talks_ahead() {
        let camera = Camera { pos: [0.0, 65.62, 0.0], yaw: 0.0, pitch: 0.0, fov: 70.0 };
        let size = [400.0, 200.0];
        let ahead = talk_icons(&camera, size, &[([0.0, 65.62, 5.0], false, false)]);
        let near = match ahead.as_slice() {
            [Draw::Sprite { name, dst, .. }] => {
                assert_eq!(name, Icon::Speaker.sprite());
                // Centred over the middle of the screen, above it.
                assert!((dst[0] + dst[2] / 2.0 - 200.0).abs() < 0.01 && dst[1] + dst[3] / 2.0 < 100.0, "{dst:?}");
                // 0.35 blocks at 5 blocks: about 10 px on a 200 px tall screen.
                assert!((9.0..=11.0).contains(&dst[2]) && dst[2] == dst[3], "{dst:?}");
                *dst
            }
            other => panic!("{other:?}"),
        };
        // Smaller farther away; higher with a group name under it.
        let far = talk_icons(&camera, size, &[([0.0, 65.62, 20.0], false, false)]);
        assert!(matches!(&far[..], [Draw::Sprite { dst, .. }] if dst[2] < near[2]));
        let labelled = talk_icons(&camera, size, &[([0.0, 65.62, 5.0], true, false)]);
        assert!(matches!(&labelled[..], [Draw::Sprite { dst, .. }] if dst[1] < near[1]));
        assert!(talk_icons(&camera, size, &[([0.0, 65.62, -5.0], false, false)]).is_empty());
        let whisper = talk_icons(&camera, size, &[([0.0, 65.62, 5.0], false, true)]);
        assert!(matches!(&whisper[..], [Draw::Sprite { name, .. }] if name == Icon::Whisper.sprite()));
        // The farther one first.
        let two = talk_icons(&camera, size, &[([0.0, 65.62, 4.0], false, false), ([1.0, 65.62, 9.0], false, true)]);
        assert!(matches!(&two[0], Draw::Sprite { name, .. } if name == Icon::Whisper.sprite()));
    }

    #[test]
    fn a_player_ahead_gets_a_label_above_the_middle() {
        let camera = Camera { pos: [0.0, 65.62, 0.0], yaw: 0.0, pitch: 0.0, fov: 70.0 };
        let size = [400.0, 200.0];
        let players = vec![
            ([0.0, 65.62, 10.0], "Builders".to_owned(), false),
            ([0.0, 65.62, 5.0], "Mine".to_owned(), true),
            ([0.0, 65.62, -10.0], "Behind".to_owned(), false),
            ([0.0, 65.62, 100.0], "Far".to_owned(), false),
        ];
        let labels = place(&camera, size, &players);
        assert_eq!(labels.len(), 2);
        // The farther one first, the nearer drawn over it.
        assert_eq!(labels[0].text, "[Builders]");
        assert_eq!((labels[1].text.as_str(), labels[1].rgb), ("[Mine]", 0x55FF55));
        assert!((labels[0].at[0] - 200.0).abs() < 0.01 && labels[0].at[1] < 100.0);
    }
}
