//! What the voice chat draws over the world, as Simple Voice Chat's HUD
//! does: one status icon in the bottom left corner, the group's members in
//! the top left with a speaker by whoever is talking, and problems (no
//! microphone, no libopus) as red lines.
use crate::skin::{self, Icon};
use crate::ui::Draw;

/// The voice chat as the status icon shows it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HudState {
    /// The voice connection is up.
    pub connected: bool,
    pub muted: bool,
    /// Voice chat turned off: nothing heard, nothing sent.
    pub disabled: bool,
    /// The microphone is being sent now.
    pub transmitting: bool,
    pub whispering: bool,
}

/// The icon's size and its distance from the screen's left and bottom edges.
pub const ICON: f32 = 16.0;
const MARGIN: f32 = 4.0;

/// The status icon in the bottom left of a `size` screen: none while
/// nothing is worth saying (connected, quiet, not muted).
pub fn status_icon(state: &HudState, size: [f32; 2]) -> Vec<Draw> {
    let icon = if state.disabled {
        Icon::SpeakerOff
    } else if !state.connected {
        Icon::Disconnected
    } else if state.muted {
        Icon::MicOff
    } else if state.transmitting && state.whispering {
        Icon::Whisper
    } else if state.transmitting {
        Icon::Mic
    } else {
        return Vec::new();
    };
    vec![skin::icon(icon, [MARGIN, size[1] - MARGIN - ICON, ICON, ICON])]
}

/// One line of the group list.
const ROW: f32 = 12.0;
const SMALL: f32 = 10.0;

/// The group's members from the top left (`members`: name, talking now),
/// 12 px apart, with a speaker by those talking. The list's own size
/// is [`group_list_size`].
pub fn group_list(members: &[(String, bool)], _size: [f32; 2], text_width: &dyn Fn(&str) -> f32) -> Vec<Draw> {
    let mut out = Vec::new();
    if members.is_empty() {
        return out;
    }
    let [w, h] = group_list_size(members, text_width);
    out.push(Draw::Fill([MARGIN - 2.0, MARGIN - 2.0, w, h], 0x6000_0000));
    for (i, (name, talking)) in members.iter().enumerate() {
        let y = MARGIN + i as f32 * ROW;
        if *talking {
            out.push(skin::icon(Icon::Speaker, [MARGIN, y, SMALL, SMALL]));
        }
        let rgb = if *talking { 0xFF_FFFF } else { 0xAA_AAAA };
        out.push(Draw::Text(name.clone(), [MARGIN + SMALL + 3.0, y + 1.0], rgb, true));
    }
    out
}

/// The width and height the group list takes, its background included.
pub fn group_list_size(members: &[(String, bool)], text_width: &dyn Fn(&str) -> f32) -> [f32; 2] {
    let widest = members.iter().map(|(n, _)| text_width(n)).fold(0.0, f32::max);
    [SMALL + 3.0 + widest + 4.0, members.len() as f32 * ROW + 2.0]
}

/// Red lines for what doesn't work, above the status icon.
pub fn problems(lines: &[String], size: [f32; 2], text_width: &dyn Fn(&str) -> f32) -> Vec<Draw> {
    let mut out = Vec::new();
    let bottom = size[1] - MARGIN - ICON - 4.0;
    for (i, line) in lines.iter().rev().enumerate() {
        let y = bottom - (i as f32 + 1.0) * ROW;
        let w = text_width(line) + 4.0;
        out.push(Draw::Fill([MARGIN - 2.0, y, w, ROW], 0x9000_0000));
        out.push(Draw::Text(line.clone(), [MARGIN, y + 2.0], 0xFF_5555, true));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn only_icon(d: &[Draw]) -> Option<(String, [f32; 4])> {
        match d {
            [Draw::Sprite { name, dst, .. }] => Some((name.clone(), *dst)),
            [] => None,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_status_icon_says_what_matters() {
        let size = [320.0, 240.0];
        let on = HudState { connected: true, ..HudState::default() };
        let talking = HudState { transmitting: true, ..on };
        assert_eq!(only_icon(&status_icon(&talking, size)), Some((Icon::Mic.sprite().into(), [4.0, 220.0, 16.0, 16.0])));
        let whisper = HudState { whispering: true, ..talking };
        assert_eq!(only_icon(&status_icon(&whisper, size)).unwrap().0, Icon::Whisper.sprite());
        let muted = HudState { muted: true, ..on };
        assert_eq!(only_icon(&status_icon(&muted, size)).unwrap().0, Icon::MicOff.sprite());
        let off = HudState { disabled: true, ..on };
        assert_eq!(only_icon(&status_icon(&off, size)).unwrap().0, Icon::SpeakerOff.sprite());
        let lost = HudState::default();
        assert_eq!(only_icon(&status_icon(&lost, size)).unwrap().0, Icon::Disconnected.sprite());
        assert_eq!(only_icon(&status_icon(&on, size)), None);
        // It follows the screen's bottom edge.
        assert_eq!(only_icon(&status_icon(&talking, [960.0, 540.0])).unwrap().1, [4.0, 520.0, 16.0, 16.0]);
    }

    #[test]
    fn the_group_list_marks_who_talks() {
        let tw = |t: &str| t.len() as f32 * 6.0;
        let members = vec![("Alex".to_owned(), false), ("Bea".to_owned(), true), ("Cem".to_owned(), false)];
        let d = group_list(&members, [320.0, 240.0], &tw);
        let names: Vec<(String, f32)> =
            d.iter().filter_map(|x| if let Draw::Text(t, at, ..) = x { Some((t.clone(), at[1])) } else { None }).collect();
        assert_eq!(names.iter().map(|n| n.0.as_str()).collect::<Vec<_>>(), ["Alex", "Bea", "Cem"]);
        assert_eq!(names[1].1 - names[0].1, 12.0);
        assert_eq!(names[2].1 - names[1].1, 12.0);
        let speakers: Vec<[f32; 4]> = d
            .iter()
            .filter_map(|x| match x {
                Draw::Sprite { name, dst, .. } if name == Icon::Speaker.sprite() => Some(*dst),
                _ => None,
            })
            .collect();
        assert_eq!(speakers.len(), 1);
        // On the second line.
        assert_eq!(speakers[0][1], 4.0 + 12.0);
        assert!(group_list(&[], [320.0, 240.0], &tw).is_empty());
    }
}
