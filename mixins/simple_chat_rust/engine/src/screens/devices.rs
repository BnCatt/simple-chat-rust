// Choosing the microphone or the speakers: Default (the system's), then each
// device by name; the chosen one in green.
use super::{DeviceKind, ROW, Ui, scrollbar, window};
use crate::skin;
use crate::text;
use crate::ui::{Action, Draw, Screens, View, Widget};

const MAX_ROWS: usize = 8;

pub fn layout(ui: &mut Ui<'_>, state: &Screens, view: &View, kind: DeviceKind) {
    let rows = ((((ui.size[1] - 70.0) / ROW).floor().max(1.0)) as usize).min(MAX_ROWS);
    let title = match kind {
        DeviceKind::Input => text::SELECT_MICROPHONE,
        DeviceKind::Output => text::SELECT_SPEAKER,
    };
    let [px, py, pw, ph] = ui.panel(260.0, 56.0 + rows as f32 * ROW, title);
    let list = [px + 7.0, py + 20.0, pw - 14.0, rows as f32 * ROW + 2.0];
    ui.draws.extend(skin::inset(list));
    let chosen = match kind {
        DeviceKind::Input => view.settings.mic_device.as_deref(),
        DeviceKind::Output => view.settings.speaker_device.as_deref(),
    };
    // A device chosen before but gone now is not listed: Default is in use.
    let chosen = chosen.filter(|c| view.devices.iter().any(|d| d == c));
    let entries: Vec<Option<&str>> = std::iter::once(None).chain(view.devices.iter().map(|d| Some(d.as_str()))).collect();
    let (first, shown) = window(state.scroll, entries.len(), rows);
    let bar = entries.len() > rows;
    let rw = list[2] - 2.0 - if bar { 6.0 } else { 0.0 };
    for (i, entry) in entries.iter().skip(first).take(shown).enumerate() {
        let rect = [list[0] + 1.0, list[1] + 1.0 + i as f32 * ROW, rw, ROW];
        if ui.hover(rect) {
            ui.draws.push(Draw::Fill(rect, 0x40FF_FFFF));
        }
        let label = entry.unwrap_or(text::DEFAULT_DEVICE);
        let shown_label = ui.fit(label, rw - 8.0);
        let rgb = if *entry == chosen { 0x55_FF55 } else { 0xFF_FFFF };
        ui.text(&shown_label, [rect[0] + 4.0, rect[1] + 6.0], rgb, true);
        if shown_label != label {
            ui.tip(rect, vec![label.to_owned()]);
        }
        ui.widgets.push(Widget { rect, action: Action::Device(kind, entry.map(str::to_owned)) });
    }
    if bar {
        scrollbar(ui, [list[0] + list[2] - 7.0, list[1] + 1.0, 6.0, list[3] - 2.0], first, shown, entries.len());
    }
    if view.devices.is_empty() {
        ui.centred(text::NO_DEVICES, list[0] + list[2] / 2.0, list[1] + ROW + 6.0, 0xC0_C0C0, true);
    }
    ui.back([(px + pw / 2.0 - 75.0).floor(), py + ph - 28.0, 150.0, 20.0], text::BACK);
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::{Screen, layout};
    use super::*;

    fn colour(draws: &[Draw], want: &str) -> u32 {
        draws.iter().find_map(|d| match d {
            Draw::Text(t, _, rgb, _) if t == want => Some(*rgb),
            _ => None,
        }).unwrap()
    }

    #[test]
    fn default_first_and_the_chosen_one_green() {
        let size = [320.0, 240.0];
        let mut v = view();
        v.devices = vec!["USB Mic".into(), "Headset".into()];
        let screen = Screen::Devices(DeviceKind::Input);
        let (draws, widgets) = layout(&screen, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        let rows: Vec<&Action> = widgets.iter().map(|w| &w.action).filter(|a| matches!(a, Action::Device(..))).collect();
        assert_eq!(rows[0], &Action::Device(DeviceKind::Input, None));
        assert_eq!(rows[2], &Action::Device(DeviceKind::Input, Some("Headset".into())));
        assert_eq!(colour(&draws, "Default"), 0x55FF55);
        v.settings.mic_device = Some("Headset".into());
        let (draws, _) = layout(&screen, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        assert_eq!((colour(&draws, "Default"), colour(&draws, "Headset")), (0xFFFFFF, 0x55FF55));
        // A device that is gone: Default again.
        v.settings.mic_device = Some("Unplugged".into());
        let (draws, _) = layout(&screen, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        assert_eq!(colour(&draws, "Default"), 0x55FF55);
        assert!(on_screen(&widgets, size));
    }
}
