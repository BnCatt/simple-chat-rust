// Adjust Volumes: a slider for each other player with voice chat, 0 to 400 %
// as Simple Voice Chat's, kept per player.
use super::{Ui, scrollbar, window};
use crate::skin;
use crate::text;
use crate::ui::{Screens, Slider, View};

pub const PLAYER_MAX: f32 = 400.0;
const ROW: f32 = 24.0;
const MAX_ROWS: usize = 6;

pub fn player_value(percent: u32) -> f32 {
    percent as f32 / PLAYER_MAX
}

pub fn player_of(v: f32) -> u32 {
    (v.clamp(0.0, 1.0) * PLAYER_MAX).round() as u32
}

pub fn layout(ui: &mut Ui<'_>, state: &Screens, view: &View) {
    let rows = ((((ui.size[1] - 70.0) / ROW).floor().max(1.0)) as usize).min(MAX_ROWS);
    let [px, py, pw, ph] = ui.panel(260.0, 56.0 + rows as f32 * ROW, text::VOLUMES);
    let list = [px + 7.0, py + 20.0, pw - 14.0, rows as f32 * ROW + 2.0];
    ui.draws.extend(skin::inset(list));
    if view.players.is_empty() {
        ui.centred(text::NO_PLAYERS, list[0] + list[2] / 2.0, list[1] + list[3] / 2.0 - 4.0, 0xFF_FFFF, true);
    } else {
        let (first, shown) = window(state.scroll, view.players.len(), rows);
        let bar = view.players.len() > rows;
        let rw = list[2] - 6.0 - if bar { 6.0 } else { 0.0 };
        for (i, p) in view.players.iter().skip(first).take(shown).enumerate() {
            let rect = [list[0] + 3.0, list[1] + 3.0 + i as f32 * ROW, rw, 20.0];
            ui.slider(rect, player_value(p.volume), &text::player_volume(&p.name, p.volume), Slider::Player(p.id), Vec::new());
        }
        if bar {
            scrollbar(ui, [list[0] + list[2] - 7.0, list[1] + 1.0, 6.0, list[3] - 2.0], first, shown, view.players.len());
        }
    }
    ui.back([(px + pw / 2.0 - 75.0).floor(), py + ph - 28.0, 150.0, 20.0], text::BACK);
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::{Screen, layout};
    use super::*;
    use crate::ui::{Action, PlayerRow};

    #[test]
    fn a_slider_per_player_from_0_to_400() {
        let size = [320.0, 240.0];
        let mut v = view();
        v.players = vec![
            PlayerRow { id: 0xA, name: "Alex".into(), volume: 100 },
            PlayerRow { id: 0xB, name: "Bea".into(), volume: 200 },
        ];
        let (draws, widgets) = layout(&Screen::Volumes, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        assert!(texts(&draws).contains(&"Bea: 200%".to_owned()));
        let r = widgets.iter().find(|w| matches!(w.action, Action::Slide(Slider::Player(0xB), _))).unwrap().rect;
        let mut m = Screens::default();
        m.set_widgets(widgets.clone());
        let Some(Action::Slide(Slider::Player(id), value)) = m.press([r[0] + 4.0 + 0.25 * (r[2] - 8.0), r[1] + 5.0]) else {
            panic!()
        };
        assert_eq!((id, player_of(value)), (0xB, 100));
        assert!(on_screen(&widgets, size));
    }
}
