//! The screens' layouts, laid out each frame from the voice chat's
//! [`View`]: vanilla buttons, sliders and text fields (`skin`), on the
//! light grey panels a container screen has, as Simple Voice Chat's screens
//! look. Every position comes from the GUI size, so they follow the GUI scale.
mod devices;
mod group;
mod main;
mod settings;
mod volumes;

use crate::skin::{self, ButtonState, Icon};
use crate::ui::{Action, Draw, Screens, Slider, View, Widget};

pub use settings::{gain_of, threshold_of, volume_of};
pub use volumes::{PLAYER_MAX, player_of};

/// A sound device kind: microphones or speakers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceKind {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Main,
    Settings,
    Group,
    CreateGroup,
    Volumes,
    Devices(DeviceKind),
}

/// `screen` laid out on a `size` GUI with the pointer at `mouse`: what to
/// draw, and the widgets clicks find.
pub fn layout(
    screen: &Screen,
    state: &Screens,
    view: &View,
    size: [f32; 2],
    mouse: [f32; 2],
    text_width: &dyn Fn(&str) -> f32,
) -> (Vec<Draw>, Vec<Widget>) {
    let mut ui = Ui::new(size, mouse, text_width, state.dragging());
    // The world dimmed behind, as vanilla's in-game screens.
    ui.draws.push(Draw::Fill([0.0, 0.0, size[0], size[1]], 0xC010_1010));
    match screen {
        Screen::Main => main::layout(&mut ui, view),
        Screen::Settings => settings::layout(&mut ui, view),
        Screen::Group => group::list(&mut ui, state, view),
        Screen::CreateGroup => group::create(&mut ui, state),
        Screen::Volumes => volumes::layout(&mut ui, state, view),
        Screen::Devices(kind) => devices::layout(&mut ui, state, view, *kind),
    }
    ui.finish()
}

/// Draws and widgets being laid out, and the tooltip for whatever the
/// pointer is over (drawn last, over everything).
pub struct Ui<'a> {
    pub draws: Vec<Draw>,
    pub widgets: Vec<Widget>,
    pub size: [f32; 2],
    pub mouse: [f32; 2],
    text_width: &'a dyn Fn(&str) -> f32,
    dragging: Option<Slider>,
    tooltip: Vec<String>,
}

pub const ROW: f32 = 20.0;
const TEXT: u32 = 0xE0_E0E0;
const HINT: u32 = 0x70_7070;

impl<'a> Ui<'a> {
    pub fn new(size: [f32; 2], mouse: [f32; 2], text_width: &'a dyn Fn(&str) -> f32, dragging: Option<Slider>) -> Self {
        Self { draws: Vec::new(), widgets: Vec::new(), size, mouse, text_width, dragging, tooltip: Vec::new() }
    }

    pub fn finish(mut self) -> (Vec<Draw>, Vec<Widget>) {
        let tip = std::mem::take(&mut self.tooltip);
        self.draws.extend(skin::tooltip(&tip, self.mouse, self.size, self.text_width));
        (self.draws, self.widgets)
    }

    pub fn width(&self, text: &str) -> f32 {
        (self.text_width)(text)
    }

    pub fn fit(&self, text: &str, max: f32) -> String {
        skin::fit(text, max, self.text_width)
    }

    pub fn hover(&self, rect: [f32; 4]) -> bool {
        let [x, y, w, h] = rect;
        (x..x + w).contains(&self.mouse[0]) && (y..y + h).contains(&self.mouse[1])
    }

    pub fn tip(&mut self, rect: [f32; 4], lines: Vec<String>) {
        if self.hover(rect) && !lines.is_empty() {
            self.tooltip = lines;
        }
    }

    pub fn text(&mut self, text: &str, at: [f32; 2], rgb: u32, shadow: bool) {
        self.draws.push(Draw::Text(text.to_owned(), at, rgb, shadow));
    }

    /// `text` centred on `x`.
    pub fn centred(&mut self, text: &str, x: f32, y: f32, rgb: u32, shadow: bool) {
        let w = self.width(text);
        self.text(text, [(x - w / 2.0).floor(), y], rgb, shadow);
    }

    /// A panel `w`×`h` in the middle of the screen, titled: its rectangle.
    pub fn panel(&mut self, w: f32, h: f32, title: &str) -> [f32; 4] {
        let w = w.min(self.size[0] - 8.0).floor();
        let h = h.min(self.size[1] - 8.0).floor();
        let rect = [((self.size[0] - w) / 2.0).floor(), ((self.size[1] - h) / 2.0).floor(), w, h];
        self.draws.extend(skin::panel(rect));
        let title = self.fit(title, w - 16.0);
        self.centred(&title, rect[0] + w / 2.0, rect[1] + 7.0, skin::TITLE, false);
        rect
    }

    /// A vanilla button; `action` None draws it disabled.
    pub fn button(&mut self, rect: [f32; 4], label: &str, action: Option<Action>) {
        let state = match &action {
            None => ButtonState::Disabled,
            Some(_) if self.hover(rect) => ButtonState::Hover,
            Some(_) => ButtonState::Normal,
        };
        let label = self.fit(label, rect[2] - 8.0);
        let w = self.width(&label);
        self.draws.extend(skin::button(rect, state, &label, w));
        if let Some(action) = action {
            self.widgets.push(Widget { rect, action });
        }
    }

    /// A 20×20 button with an icon, and a tooltip.
    pub fn icon_button(&mut self, rect: [f32; 4], icon: Icon, action: Option<Action>, tip: String) {
        let state = match &action {
            None => ButtonState::Disabled,
            Some(_) if self.hover(rect) => ButtonState::Hover,
            Some(_) => ButtonState::Normal,
        };
        self.draws.extend(skin::icon_button(rect, state, icon));
        self.tip(rect, vec![tip]);
        if let Some(action) = action {
            self.widgets.push(Widget { rect, action });
        }
    }

    /// A slider at `value` (0..=1) with its label; `under` draws between the
    /// track and the handle (the threshold slider's level meter).
    pub fn slider(&mut self, rect: [f32; 4], value: f32, label: &str, slider: Slider, under: Vec<Draw>) {
        let dragging = self.dragging == Some(slider);
        let hover = self.hover(rect) || dragging;
        let track = if dragging { &skin::SLIDER_HIGHLIGHTED } else { &skin::SLIDER };
        self.draws.extend(skin::nine_slice(track, rect));
        self.draws.extend(under);
        self.draws.extend(skin::slider_handle(rect, value, hover));
        let label = self.fit(label, rect[2] - 8.0);
        let w = self.width(&label);
        let [x, y, rw, h] = rect;
        self.text(&label, [(x + (rw - w) / 2.0).floor(), y + ((h - 8.0) / 2.0).floor()], skin::LABEL, true);
        self.widgets.push(Widget { rect, action: Action::Slide(slider, value) });
    }

    /// A text field showing `value` (stars for a password), its hint when
    /// empty, a caret when focused.
    pub fn field(&mut self, rect: [f32; 4], value: &str, hint: &str, secret: bool, focus: bool, action: Action) {
        self.draws.extend(skin::text_field(rect, focus));
        let shown = if secret { "*".repeat(value.chars().count()) } else { value.to_owned() };
        let [x, y, w, h] = rect;
        let ty = y + ((h - 8.0) / 2.0).floor();
        if shown.is_empty() && !focus {
            let hint = self.fit(hint, w - 8.0);
            self.text(&hint, [x + 4.0, ty], HINT, false);
        } else {
            // The end of a long value, as a field scrolls to its caret.
            let mut chars: Vec<char> = shown.chars().collect();
            while !chars.is_empty() && self.width(&chars.iter().collect::<String>()) > w - 14.0 {
                chars.remove(0);
            }
            let caret = if focus { "_" } else { "" };
            self.text(&format!("{}{caret}", chars.iter().collect::<String>()), [x + 4.0, ty], TEXT, true);
        }
        self.widgets.push(Widget { rect, action });
    }

    /// A Done/Back button at the bottom of a panel or screen.
    pub fn back(&mut self, rect: [f32; 4], label: &str) {
        self.button(rect, label, Some(Action::Back));
    }
}

/// A list's visible window: the first row shown and how many, for `len`
/// rows with room for `fits`.
pub fn window(scroll: usize, len: usize, fits: usize) -> (usize, usize) {
    let first = scroll.min(len.saturating_sub(fits));
    (first, fits.min(len - first.min(len)))
}

/// A scrollbar along a list at `rect` showing `shown` of `len` rows from `first`.
pub fn scrollbar(ui: &mut Ui<'_>, rect: [f32; 4], first: usize, shown: usize, len: usize) {
    if len <= shown || len == 0 {
        return;
    }
    let [x, y, w, h] = rect;
    ui.draws.push(Draw::Fill(rect, 0xFF00_0000));
    let thumb = (h * shown as f32 / len as f32).max(8.0).floor();
    let at = y + ((h - thumb) * first as f32 / (len - shown) as f32).floor();
    ui.draws.push(Draw::Fill([x, at, w, thumb], 0xFF80_8080));
    ui.draws.push(Draw::Fill([x, at, w - 1.0, thumb - 1.0], 0xFFC0_C0C0));
}

#[cfg(test)]
pub(crate) mod testing {
    use super::*;
    use crate::ui::GroupRow;

    pub fn tw(t: &str) -> f32 {
        t.chars().count() as f32 * 6.0
    }

    pub fn view() -> View {
        View {
            status: ("Connected".into(), 0x55FF55),
            connected: true,
            groups_enabled: true,
            groups: vec![
                GroupRow { id: 1, name: "Builders".into(), has_password: false, members: vec!["Alex".into()], ours: false },
                GroupRow { id: 2, name: "Secret".into(), has_password: true, members: vec![], ours: false },
            ],
            ..View::default()
        }
    }

    pub fn texts(draws: &[Draw]) -> Vec<String> {
        draws.iter().filter_map(|d| if let Draw::Text(t, ..) = d { Some(t.clone()) } else { None }).collect()
    }

    pub fn text_at(draws: &[Draw], want: &str) -> Option<[f32; 2]> {
        draws.iter().find_map(|d| match d {
            Draw::Text(t, at, ..) if t == want => Some(*at),
            _ => None,
        })
    }

    pub fn widget(widgets: &[Widget], action: &Action) -> Option<[f32; 4]> {
        widgets.iter().find(|w| &w.action == action).map(|w| w.rect)
    }

    pub fn centre(rect: [f32; 4]) -> [f32; 2] {
        [rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0]
    }

    /// Every widget inside the screen.
    pub fn on_screen(widgets: &[Widget], size: [f32; 2]) -> bool {
        widgets.iter().all(|w| w.rect[0] >= 0.0 && w.rect[1] >= 0.0 && w.rect[0] + w.rect[2] <= size[0] && w.rect[1] + w.rect[3] <= size[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_window_never_runs_past_its_end() {
        assert_eq!(window(0, 25, 8), (0, 8));
        assert_eq!(window(3, 25, 8), (3, 8));
        assert_eq!(window(99, 25, 8), (17, 8));
        assert_eq!(window(5, 3, 8), (0, 3));
        assert_eq!(window(0, 0, 8), (0, 0));
    }
}
