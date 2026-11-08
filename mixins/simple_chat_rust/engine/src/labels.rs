//! Labels and icons by players in the world: which voice chat group each is
//! in and the icon right of the name tag, as this frame's camera sees them.

/// The camera: its position, yaw and pitch in degrees, vertical field of view,
/// and the matrices the world was drawn with when the game gives them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub pos: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub matrices: Option<Matrices>,
}

/// The frame's camera-relative view matrix (with the view bobbing in it) and
/// its projection, column-major. Projecting with them puts a point exactly
/// where the world drew it, so an icon walks and bobs with the name tag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrices {
    pub view: [f32; 16],
    pub projection: [f32; 16],
}

/// `m * v` for a column-major 4×4 matrix.
fn mul(m: &[f32; 16], v: [f64; 4]) -> [f64; 4] {
    std::array::from_fn(|r| (0..4).map(|c| f64::from(m[c * 4 + r]) * v[c]).sum())
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
    let d = sub(point, camera.pos);
    if let Some(m) = &camera.matrices {
        let c = mul(&m.projection, mul(&m.view, [d[0], d[1], d[2], 1.0]));
        // w is the depth in front of the camera.
        if c[3] < 0.05 || size[1] <= 0.0 {
            return None;
        }
        let (nx, ny) = (c[0] / c[3], c[1] / c[3]);
        return Some([((nx + 1.0) / 2.0 * f64::from(size[0])) as f32, ((1.0 - ny) / 2.0 * f64::from(size[1])) as f32]);
    }
    let (f, r, u) = basis(camera.yaw, camera.pitch);
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

/// Where vanilla draws a player's name tag: `EntityRenderer` puts the text's
/// top this far above the eyes (the name tag attachment, 0.18 over the eyes
/// of a standing player, plus 0.5), one text pixel being this many blocks.
pub const NAME_TAG_ABOVE_EYES: f64 = 0.68;
pub const NAME_TAG_UNIT: f64 = 0.025;

/// `EntityRenderer.extractNameTags`: a below-name score shows within this
/// many blocks and lifts the name one line (`9 * 1.15` text pixels).
pub const BELOW_NAME_RANGE: f64 = 10.0;
pub const BELOW_NAME_LIFT: f64 = 9.0 * 1.15 * NAME_TAG_UNIT;
/// How many text pixels the icon's square starts above the name's top.
pub const ICON_RAISE: f32 = 0.5;

/// The icon by one player's name tag: eye position, the shown name's width in
/// text pixels (the team's prefix and suffix included, as `getDisplayName`),
/// whether a below-name score goes under it, and the icon.
#[derive(Clone, Debug, PartialEq)]
pub struct NameIcon {
    pub eye: [f64; 3],
    pub name_width: f32,
    pub below_name: bool,
    pub icon: crate::skin::Icon,
}

/// The icons right of `players`' name tags, where Simple Voice Chat draws
/// them (`RenderEvents.renderIcon`): `width / 2 + 2` text pixels right of
/// the tag's centre, 10 text pixels square, level with the name. The square
/// is laid out on the tag's own plane (the camera's right and up, as the game
/// turns the tag to face the camera) and its corners projected, so it stays
/// beside the tag however the camera turns, and shrinks with it. The
/// farthest first, so the nearest draws on top.
///
/// The square is then put on whole physical pixels (`scale` of them to a GUI
/// pixel) counted from the pixels the name itself covers: its left edge a
/// fixed number of pixels after the text's last column, its top from the
/// text's first row, one size at a given distance. The world draws the text by
/// pixel centres; a square placed between pixels instead lands a pixel off
/// the text every other frame and drops different texels of its 16 as it
/// moves, which shows as a wiggle by the name while the camera turns.
pub fn name_icons(camera: &Camera, size: [f32; 2], scale: f32, players: &[NameIcon]) -> Vec<crate::ui::Draw> {
    let k = f64::from(scale.max(1.0));
    let (_, right, up) = basis(camera.yaw, camera.pitch);
    let mut placed: Vec<(f64, crate::ui::Draw)> = players
        .iter()
        .filter_map(|p| {
            let mut anchor = [p.eye[0], p.eye[1] + NAME_TAG_ABOVE_EYES, p.eye[2]];
            let d = sub(anchor, camera.pos);
            let distance = dot(d, d).sqrt();
            if distance > RANGE {
                return None;
            }
            // Simple Voice Chat lifts the icon with the name when the score
            // line goes under it.
            if p.below_name && distance < BELOW_NAME_RANGE {
                anchor[1] += BELOW_NAME_LIFT;
            }
            // A point of the tag in text pixels: x right, y down from the
            // name's top centre (`Matrix4f.scale(0.025, -0.025, 0.025)` after
            // the camera's rotation).
            let on_tag = |x: f64, y: f64| {
                let k = |i: usize| anchor[i] + (right[i] * x - up[i] * y) * NAME_TAG_UNIT;
                [k(0), k(1), k(2)]
            };
            // Horizon draws the name from -width/2 to width/2
            // (`HudBatch::name_tag`); Simple Voice Chat's icon starts at
            // `width / 2 + 2` in whole text pixels.
            let half = f64::from(p.name_width) / 2.0;
            let left = half.floor() + 2.0;
            // What shows of the name ends a pixel before its advance: each
            // glyph's last column is the gap to the next.
            let shown_end = half - 1.0;
            // Level with the name: the icon's art (half a pixel into its
            // square) spans the text's 8 rows, shadow included.
            let top = -f64::from(ICON_RAISE);
            let edge = project(camera, on_tag(shown_end, 0.0), size)?;
            let far = project(camera, on_tag(shown_end + 10.0, 0.0), size)?;
            // Physical pixels to a text pixel (the tag faces the camera, so
            // the same across and down).
            let s = f64::from(far[0] - edge[0]) * k / 10.0;
            if s <= 0.0 {
                return None;
            }
            // The text's last column ends, and its first row starts, where
            // the pixel-centre rule puts them.
            let after_text = (f64::from(edge[0]) * k - 0.5).ceil();
            let text_top = (f64::from(edge[1]) * k - 0.5).ceil();
            let x = after_text + ((left - shown_end) * s).round();
            let y = text_top + (top * s).round();
            let side = (10.0 * s).round().max(1.0);
            let [x, y, side] = [x / k, y / k, side / k].map(|v| v as f32);
            let on_screen = x + side >= 0.0 && x <= size[0] && y + side >= 0.0 && y <= size[1];
            on_screen.then(|| (distance, crate::skin::icon(p.icon, [x, y, side, side])))
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

    fn one(d: &[Draw]) -> (String, [f32; 4]) {
        match d {
            [Draw::Sprite { name, dst, .. }] => (name.clone(), *dst),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_icon_sits_right_of_the_name_tag() {
        let camera = Camera { pos: [0.0, 65.62, 0.0], yaw: 0.0, pitch: 0.0, fov: 70.0, matrices: None };
        let size = [400.0, 200.0];
        let at = |z: f64, w: f32, icon: Icon| NameIcon { eye: [0.0, 65.62, z], name_width: w, below_name: false, icon };
        let (name, near) = one(&name_icons(&camera, size, 64.0, &[at(5.0, 30.0, Icon::Speaker)]));
        assert_eq!(name, Icon::Speaker.sprite());
        // The tag's top, 0.68 above the eyes, 5 blocks ahead.
        let top = project(&camera, [0.0, 65.62 + 0.68, 5.0], size).unwrap();
        let s = top[1] - project(&camera, [0.0, 65.62 + 0.705, 5.0], size).unwrap()[1];
        // 15 + 2 text pixels right of the middle, half a pixel up, 10 square.
        assert!((near[0] - (200.0 + 17.0 * s)).abs() < 0.01, "{near:?}");
        assert!((near[1] - (top[1] - 0.5 * s)).abs() < 0.01 && near[1] < 100.0, "{near:?}");
        assert!((near[2] - 10.0 * s).abs() < 0.01 && (near[2] - near[3]).abs() < 0.001, "{near:?}");
        // A longer name pushes it right; farther away it shrinks.
        let (_, wide) = one(&name_icons(&camera, size, 64.0, &[at(5.0, 60.0, Icon::Speaker)]));
        assert!(wide[0] > near[0] && (wide[2] - near[2]).abs() < 0.001);
        let (_, far) = one(&name_icons(&camera, size, 64.0, &[at(20.0, 30.0, Icon::Speaker)]));
        assert!(far[2] < near[2]);
        // Behind the camera or out of range: nothing.
        assert!(name_icons(&camera, size, 64.0, &[at(-5.0, 30.0, Icon::Speaker), at(80.0, 30.0, Icon::Speaker)]).is_empty());
        // The farther one first.
        let two = name_icons(&camera, size, 64.0, &[at(4.0, 30.0, Icon::Speaker), at(9.0, 30.0, Icon::Whisper)]);
        assert!(matches!(&two[0], Draw::Sprite { name, .. } if name == Icon::Whisper.sprite()));
    }

    #[test]
    fn the_icon_keeps_beside_the_tag_as_the_camera_turns() {
        // The tag's right edge and the icon's left edge, 2 text pixels apart
        // along the tag, wherever the tag is on screen and however the
        // camera is pitched (the old screen-space offset slid sideways).
        let size = [400.0, 200.0];
        let eye = [3.0, 66.0, 6.0];
        let w = 60.0f32;
        for (yaw, pitch) in [(0.0, 0.0), (-20.0, 0.0), (25.0, 10.0), (-10.0, 45.0), (15.0, -35.0), (30.0, 60.0)] {
            for framed in [false, true] {
                let mut camera = Camera { pos: [0.0, 65.62, 0.0], yaw, pitch, fov: 70.0, matrices: None };
                if framed {
                    camera.matrices = Some(matrices(yaw, pitch, 70.0, 2.0));
                }
                let Some(dst) = name_icons(&camera, size, 64.0, &[NameIcon { eye, name_width: w, below_name: false, icon: Icon::Speaker }])
                    .first()
                    .and_then(|d| if let Draw::Sprite { dst, .. } = d { Some(*dst) } else { None })
                else {
                    continue;
                };
                let (_, r, u) = basis(yaw, pitch);
                let anchor = [eye[0], eye[1] + NAME_TAG_ABOVE_EYES, eye[2]];
                let tag = |x: f64, y: f64| -> [f32; 2] {
                    let k = |i: usize| anchor[i] + (r[i] * x - u[i] * y) * NAME_TAG_UNIT;
                    project(&camera, [k(0), k(1), k(2)], size).unwrap()
                };
                let edge = tag(30.0, 0.0);
                let px = tag(31.0, 0.0)[0] - edge[0];
                assert!((dst[0] - (edge[0] + 2.0 * px)).abs() < 0.05, "{yaw} {pitch} {framed}: {dst:?} {edge:?} {px}");
                // Square on the tag: as wide as tall, 10 of its pixels.
                assert!((dst[2] - 10.0 * px).abs() < 0.05 && (dst[2] - dst[3]).abs() < 0.05, "{yaw} {pitch}: {dst:?}");
            }
        }
    }

    #[test]
    fn the_icon_moves_with_the_text_in_whole_pixels() {
        // The HUD draws in physical pixels (GUI pixels times the scale) and the
        // tag's text covers the pixels whose centres its glyphs reach. Sliding
        // the tag across the screen a fraction of a pixel at a time (the
        // camera strafing, so its size stays the same), the icon keeps whole
        // pixels, one size, and the same distance from the text's last column
        // and first row: it can't shimmer or wiggle against the name.
        let size = [640.0, 360.0];
        let scale = 3.0f32;
        let k = f64::from(scale);
        for w in [61.0f32, 98.0] {
            let mut seen = Vec::new();
            for step in 0..90 {
                let x = f64::from(step) * 0.0137;
                let camera = Camera {
                    pos: [x, 65.62, 0.0],
                    yaw: 0.0,
                    pitch: -6.0,
                    fov: 70.0,
                    matrices: Some(matrices(0.0, -6.0, 70.0, 16.0 / 9.0)),
                };
                let eye = [0.0, 65.62, 6.0];
                let (_, dst) = one(&name_icons(&camera, size, scale, &[NameIcon { eye, name_width: w, below_name: false, icon: Icon::Speaker }]));
                let px = dst.map(|v| f64::from(v) * k);
                for v in px {
                    assert!((v - v.round()).abs() < 1e-3, "w {w} step {step}: not whole pixels {px:?}");
                }
                // The name runs from -w/2 to w/2 (Horizon's HudBatch::name_tag), top at 0;
                // its last glyph's last column is empty (the advance's spacing), so
                // what shows ends at w/2 - 1.
                let (_, r, u) = basis(camera.yaw, camera.pitch);
                let anchor = [eye[0], eye[1] + NAME_TAG_ABOVE_EYES, eye[2]];
                let tag = |tx: f64, ty: f64| -> [f32; 2] {
                    let c = |i: usize| anchor[i] + (r[i] * tx - u[i] * ty) * NAME_TAG_UNIT;
                    project(&camera, [c(0), c(1), c(2)], size).unwrap()
                };
                let edge = tag(f64::from(w) / 2.0 - 1.0, 0.0);
                let last_column = (f64::from(edge[0]) * k - 0.5).ceil();
                let first_row = (f64::from(edge[1]) * k - 0.5).ceil();
                let rel = [(px[0] - last_column).round() as i64, (px[1] - first_row).round() as i64, px[2].round() as i64, px[3].round() as i64];
                if !seen.contains(&rel) {
                    seen.push(rel);
                }
            }
            assert_eq!(seen.len(), 1, "w {w}: the icon moved against the text: {seen:?}");
        }
    }

    #[test]
    fn a_rank_widens_the_tag_and_a_score_lifts_the_icon() {
        let camera = Camera { pos: [0.0, 65.62, 0.0], yaw: 0.0, pitch: 0.0, fov: 70.0, matrices: None };
        let size = [400.0, 200.0];
        let tag = |w: f32, below_name: bool, z: f64| NameIcon { eye: [0.0, 65.62, z], name_width: w, below_name, icon: Icon::Speaker };
        // "Steve" (30) and "[Yönetici] Steve" (90): the icon moves 30 text pixels right.
        let (_, plain) = one(&name_icons(&camera, size, 64.0, &[tag(30.0, false, 5.0)]));
        let (_, ranked) = one(&name_icons(&camera, size, 64.0, &[tag(90.0, false, 5.0)]));
        let s = plain[2] / 10.0;
        assert!((ranked[0] - plain[0] - 30.0 * s).abs() < 0.01, "{plain:?} {ranked:?}");
        // A below-name score lifts it one line near by, not beyond 10 blocks.
        let (_, lifted) = one(&name_icons(&camera, size, 64.0, &[tag(30.0, true, 5.0)]));
        assert!(plain[1] - lifted[1] > 9.0 * s, "{plain:?} {lifted:?}");
        let (_, far) = one(&name_icons(&camera, size, 64.0, &[tag(30.0, false, 20.0)]));
        let (_, far_scored) = one(&name_icons(&camera, size, 64.0, &[tag(30.0, true, 20.0)]));
        assert_eq!(far, far_scored);
    }

    /// The matrices Horizon builds (`camera::view_matrix`: rows right, up,
    /// back; `perspective_rh` with reversed Z) for a yaw, pitch and fov.
    fn matrices(yaw: f32, pitch: f32, fov: f32, aspect: f32) -> Matrices {
        let (f, r, u) = basis(yaw, pitch);
        let rows = [r, u, [-f[0], -f[1], -f[2]]];
        let mut view = [0.0f32; 16];
        for (i, row) in rows.iter().enumerate() {
            for (c, v) in row.iter().enumerate() {
                view[c * 4 + i] = *v as f32;
            }
        }
        view[15] = 1.0;
        let t = 1.0 / (fov.to_radians() / 2.0).tan();
        let (near, far) = (1000.0f32, 0.05f32);
        let mut projection = [0.0f32; 16];
        projection[0] = t / aspect;
        projection[5] = t;
        projection[10] = far / (near - far);
        projection[11] = -1.0;
        projection[14] = near * far / (near - far);
        Matrices { view, projection }
    }

    #[test]
    fn the_frame_matrices_project_as_the_angles_do() {
        let size = [400.0, 200.0];
        for (yaw, pitch) in [(0.0, 0.0), (37.0, -20.0), (-120.0, 45.0), (200.0, 80.0)] {
            let plain = Camera { pos: [3.0, 70.0, -4.0], yaw, pitch, fov: 70.0, matrices: None };
            let framed = Camera { matrices: Some(matrices(yaw, pitch, 70.0, 2.0)), ..plain };
            for point in [[5.0, 71.0, 6.0], [-8.0, 69.0, 2.0], [3.0, 75.0, -4.5], [10.0, 70.0, -12.0]] {
                let (a, b) = (project(&plain, point, size), project(&framed, point, size));
                match (a, b) {
                    (Some(a), Some(b)) => {
                        assert!((a[0] - b[0]).abs() < 0.05 && (a[1] - b[1]).abs() < 0.05, "{yaw} {pitch} {point:?}: {a:?} {b:?}")
                    }
                    (None, None) => {}
                    other => panic!("{yaw} {pitch} {point:?}: {other:?}"),
                }
            }
        }
        // The view bobbing in the view matrix moves the icon with the tag.
        let mut bobbed = matrices(0.0, 0.0, 70.0, 2.0);
        bobbed.view[13] = 0.05;
        let still = Camera { pos: [0.0, 65.62, 0.0], yaw: 0.0, pitch: 0.0, fov: 70.0, matrices: Some(matrices(0.0, 0.0, 70.0, 2.0)) };
        let bob = Camera { matrices: Some(bobbed), ..still };
        let tag = NameIcon { eye: [0.0, 65.62, 5.0], name_width: 30.0, below_name: false, icon: Icon::Speaker };
        let (_, a) = one(&name_icons(&still, size, 64.0, std::slice::from_ref(&tag)));
        let (_, b) = one(&name_icons(&bob, size, 64.0, std::slice::from_ref(&tag)));
        assert!(b[1] < a[1] - 1.0 && (a[0] - b[0]).abs() < 0.01, "{a:?} {b:?}");
    }

    #[test]
    fn a_player_ahead_gets_a_label_above_the_middle() {
        let camera = Camera { pos: [0.0, 65.62, 0.0], yaw: 0.0, pitch: 0.0, fov: 70.0, matrices: None };
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
