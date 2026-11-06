// The main screen (V), as Simple Voice Chat's: the title, a row of icon
// buttons (microphone, voice chat, groups, settings) and Adjust Volumes, with
// the connection's state under the panel.
use super::{Screen, Ui};
use crate::skin::Icon;
use crate::text;
use crate::ui::{Action, View};

pub const WIDTH: f32 = 196.0;
pub const HEIGHT: f32 = 76.0;
const ICON: f32 = 20.0;
const GAP: f32 = 6.0;

pub fn layout(ui: &mut Ui<'_>, view: &View) {
    let [x, y, w, h] = ui.panel(WIDTH, HEIGHT, text::TITLE);
    let s = &view.settings;
    let mid = (ui.size[0] / 2.0).floor();
    let row = 4.0 * ICON + 3.0 * GAP;
    let left = (mid - row / 2.0).floor();
    let by = y + 22.0;
    let at = |i: f32| [left + i * (ICON + GAP), by, ICON, ICON];
    let mic = if s.muted { Icon::MicOff } else { Icon::Mic };
    ui.icon_button(at(0.0), mic, Some(Action::Mute), text::microphone_tip(s.muted));
    let speaker = if s.disabled { Icon::SpeakerOff } else { Icon::Speaker };
    ui.icon_button(at(1.0), speaker, Some(Action::Disable), text::voice_chat_tip(s.disabled));
    let groups = (view.connected && view.groups_enabled).then_some(Action::Open(Screen::Group));
    let groups_tip = match () {
        _ if !view.connected => text::NOT_CONNECTED,
        _ if !view.groups_enabled => text::GROUPS_OFF,
        _ => text::GROUPS_TIP,
    };
    ui.icon_button(at(2.0), Icon::Group, groups, groups_tip.into());
    ui.icon_button(at(3.0), Icon::Settings, Some(Action::Open(Screen::Settings)), text::SETTINGS_TIP.into());
    ui.button([mid - 75.0, y + 48.0, 150.0, 20.0], text::VOLUMES, Some(Action::Open(Screen::Volumes)));
    let (status, rgb) = &view.status;
    let status = ui.fit(status, ui.size[0] - 8.0);
    if y + h + 14.0 <= ui.size[1] {
        ui.centred(&status, x + w / 2.0, y + h + 6.0, *rgb, true);
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::{Screen, layout};
    use crate::ui::{Action, Screens};

    #[test]
    fn title_icon_row_and_volumes_in_the_middle_at_every_gui_size() {
        for size in [[320.0, 240.0], [960.0, 540.0], [427.0, 240.0]] {
            let (draws, widgets) = layout(&Screen::Main, &Screens::default(), &view(), size, [0.0, 0.0], &tw);
            let title = text_at(&draws, "Voice Chat").unwrap();
            let mid = (size[0] / 2.0).floor();
            assert!((title[0] + tw("Voice Chat") / 2.0 - mid).abs() <= 1.0, "{size:?}: {title:?}");
            let row: Vec<[f32; 4]> = [Action::Mute, Action::Disable, Action::Open(Screen::Group), Action::Open(Screen::Settings)]
                .iter()
                .map(|a| widget(&widgets, a).unwrap())
                .collect();
            // Side by side, evenly spaced, the same height, around the middle.
            let centres: Vec<f32> = row.iter().map(|r| centre(*r)[0]).collect();
            assert!(row.iter().all(|r| r[1] == row[0][1] && r[2] == 20.0 && r[3] == 20.0));
            assert_eq!(centres[1] - centres[0], centres[3] - centres[2]);
            assert_eq!(centres[2] - centres[1], centres[1] - centres[0]);
            assert!(((centres[0] + centres[3]) / 2.0 - mid).abs() <= 0.5, "{size:?}: {centres:?}");
            let volumes = widget(&widgets, &Action::Open(Screen::Volumes)).unwrap();
            assert!(volumes[1] >= row[0][1] + 20.0 && (centre(volumes)[0] - mid).abs() <= 0.5);
            assert!(on_screen(&widgets, size));
        }
    }

    #[test]
    fn groups_need_a_connection_and_a_tooltip_says_so() {
        let mut v = view();
        v.connected = false;
        let size = [320.0, 240.0];
        let (_, widgets) = layout(&Screen::Main, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        assert!(widget(&widgets, &Action::Open(Screen::Group)).is_none());
        // Hovering the mute button names the microphone's state.
        let (_, widgets) = layout(&Screen::Main, &Screens::default(), &view(), size, [0.0, 0.0], &tw);
        let mute = centre(widget(&widgets, &Action::Mute).unwrap());
        let (draws, _) = layout(&Screen::Main, &Screens::default(), &view(), size, mute, &tw);
        assert!(texts(&draws).contains(&"Microphone: On".to_owned()));
    }
}
