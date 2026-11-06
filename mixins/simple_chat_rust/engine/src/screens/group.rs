// The group screens: the server's groups (a lock on those with a password,
// the member count, Join or Leave; the members in a tooltip), and making a
// new one (name, password, type).
use super::{ROW, Screen, Ui, scrollbar, window};
use crate::skin::{self, Icon};
use crate::text;
use crate::ui::{Action, Field, Screens, View};

pub const WIDTH: f32 = 260.0;
pub const MAX_ROWS: usize = 8;

/// How many group rows fit on a `height` screen.
pub fn rows_that_fit(height: f32) -> usize {
    (((height - 78.0) / ROW).floor().max(1.0) as usize).min(MAX_ROWS)
}

pub fn list(ui: &mut Ui<'_>, state: &Screens, view: &View) {
    let rows = rows_that_fit(ui.size[1]);
    let ours = view.groups.iter().find(|g| g.ours);
    let title = ours.map_or_else(|| text::GROUPS.to_owned(), |g| text::in_group(&g.name));
    let [px, py, pw, ph] = ui.panel(WIDTH, 70.0 + rows as f32 * ROW, &title);
    let list = [px + 7.0, py + 20.0, pw - 14.0, rows as f32 * ROW + 2.0];
    ui.draws.extend(skin::inset(list));
    let bottom = py + ph - 28.0;
    let usable = view.connected && view.groups_enabled;
    if !usable || view.groups.is_empty() {
        let why = match () {
            _ if !view.connected => text::NOT_CONNECTED,
            _ if !view.groups_enabled => text::GROUPS_OFF,
            _ => text::NO_GROUPS,
        };
        ui.centred(why, list[0] + list[2] / 2.0, list[1] + list[3] / 2.0 - 4.0, 0xFF_FFFF, true);
    } else {
        let (first, shown) = window(state.scroll, view.groups.len(), rows);
        let bar = view.groups.len() > rows;
        let rw = list[2] - 2.0 - if bar { 6.0 } else { 0.0 };
        for (i, g) in view.groups.iter().skip(first).take(shown).enumerate() {
            let (x, y) = (list[0] + 1.0, list[1] + 1.0 + i as f32 * ROW);
            if state.joining == Some(g.id) {
                let fw = rw - 4.0 - 40.0 - 44.0;
                let focused = state.focus == Some(Field::JoinPassword);
                ui.field([x + 2.0, y + 2.0, fw, 16.0], &state.join_password, text::PASSWORD_FOR, true, focused, Action::Focus(Field::JoinPassword));
                ui.button([x + fw + 4.0, y + 2.0, 38.0, 16.0], text::JOIN, Some(Action::JoinWithPassword));
                ui.button([x + fw + 44.0, y + 2.0, 42.0, 16.0], text::CANCEL, Some(Action::CancelJoin));
                continue;
            }
            if g.has_password {
                ui.draws.push(skin::icon(Icon::Lock, [x + 2.0, y + 2.0, 16.0, 16.0]));
            }
            let count = text::members(g.members.len());
            let cw = ui.width(&count);
            let button = [x + rw - 44.0, y + 2.0, 42.0, 16.0];
            let room = button[0] - 4.0 - cw - 4.0 - (x + 20.0);
            let name = ui.fit(&g.name, room);
            ui.text(&name, [x + 20.0, y + 6.0], if g.ours { 0x55_FF55 } else { 0xFF_FFFF }, true);
            ui.text(&count, [button[0] - 4.0 - cw, y + 6.0], 0xC0_C0C0, true);
            let mut tip = vec![g.name.clone()];
            tip.extend(g.members.iter().cloned());
            ui.tip([x, y, button[0] - x - 2.0, ROW], tip);
            if g.ours {
                ui.button(button, text::LEAVE, Some(Action::Leave));
            } else {
                ui.button(button, text::JOIN, Some(Action::Join(g.id)));
            }
        }
        if bar {
            scrollbar(ui, [list[0] + list[2] - 7.0, list[1] + 1.0, 6.0, list[3] - 2.0], first, shown, view.groups.len());
        }
    }
    if let Some((m, rgb)) = &state.message {
        let m = ui.fit(m, pw - 16.0);
        ui.centred(&m, px + pw / 2.0, bottom - 11.0, *rgb, true);
    }
    let half = ((pw - 22.0) / 2.0).floor();
    ui.button([px + 8.0, bottom, half, 20.0], text::NEW_GROUP, usable.then_some(Action::Open(Screen::CreateGroup)));
    ui.back([px + pw - 8.0 - half, bottom, half, 20.0], text::BACK);
}

pub fn create(ui: &mut Ui<'_>, state: &Screens) {
    let [px, py, pw, ph] = ui.panel(220.0, 154.0, text::CREATE_GROUP);
    let (x, w) = (px + 10.0, pw - 20.0);
    let mut y = py + 20.0;
    ui.text(text::NAME, [x, y], skin::TITLE, false);
    y += 10.0;
    ui.field([x, y, w, 20.0], &state.name, "", false, state.focus == Some(Field::Name), Action::Focus(Field::Name));
    y += 24.0;
    ui.text(text::PASSWORD, [x, y], skin::TITLE, false);
    y += 10.0;
    ui.field([x, y, w, 20.0], &state.password, "", true, state.focus == Some(Field::Password), Action::Focus(Field::Password));
    y += 26.0;
    let kind = [x, y, w, 20.0];
    ui.button(kind, &text::group_type(state.kind), Some(Action::Kind));
    ui.tip(kind, vec![text::KIND_TIPS[state.kind.rem_euclid(3) as usize].into()]);
    let bottom = py + ph - 28.0;
    if let Some((m, rgb)) = &state.message {
        let m = ui.fit(m, pw - 16.0);
        ui.centred(&m, px + pw / 2.0, bottom - 11.0, *rgb, true);
    }
    let half = ((pw - 22.0) / 2.0).floor();
    ui.button([px + 8.0, bottom, half, 20.0], text::CREATE, Some(Action::Create));
    ui.back([px + pw - 8.0 - half, bottom, half, 20.0], text::BACK);
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::super::{Screen, layout};
    use super::*;
    use crate::ui::GroupRow;

    fn groups(n: usize) -> View {
        let mut v = view();
        v.groups = (0..n)
            .map(|i| GroupRow { id: i as u128 + 1, name: format!("Group {i}"), has_password: false, members: vec![], ours: false })
            .collect();
        v
    }

    #[test]
    fn a_long_name_is_cut_inside_its_row() {
        let size = [320.0, 240.0];
        let mut v = view();
        v.groups[0].name = "A".repeat(32);
        let (draws, widgets) = layout(&Screen::Group, &Screens::default(), &v, size, [0.0, 0.0], &tw);
        let cut = texts(&draws).into_iter().find(|t| t.starts_with("AAA")).unwrap();
        assert!(cut.ends_with('…') && cut.chars().count() < 32, "{cut}");
        let at = text_at(&draws, &cut).unwrap();
        let join = widget(&widgets, &Action::Join(1)).unwrap();
        assert!(at[0] + tw(&cut) < join[0], "the name runs under Join");
        assert!(on_screen(&widgets, size));
    }

    #[test]
    fn twenty_five_groups_show_eight_and_the_wheel_scrolls() {
        let size = [320.0, 240.0];
        let v = groups(25);
        let joins = |state: &Screens| {
            let (_, widgets) = layout(&Screen::Group, state, &v, size, [0.0, 0.0], &tw);
            widgets.iter().filter_map(|w| if let Action::Join(id) = w.action { Some(id) } else { None }).collect::<Vec<_>>()
        };
        let mut state = Screens::default();
        assert_eq!(joins(&state).len(), 8);
        for _ in 0..3 {
            state.scroll_by(-1);
        }
        assert_eq!(state.scroll, 3);
        assert_eq!(joins(&state)[0], 4);
        // Never past the last group.
        for _ in 0..50 {
            state.scroll_by(-1);
        }
        assert_eq!(*joins(&state).last().unwrap(), 25);
    }

    #[test]
    fn a_group_being_joined_asks_for_its_password() {
        let size = [320.0, 240.0];
        let mut state = Screens::default();
        state.joining = Some(2);
        state.focus = Some(Field::JoinPassword);
        let (_, widgets) = layout(&Screen::Group, &state, &view(), size, [0.0, 0.0], &tw);
        assert!(widget(&widgets, &Action::Focus(Field::JoinPassword)).is_some());
        assert!(widget(&widgets, &Action::JoinWithPassword).is_some());
        assert!(widget(&widgets, &Action::Join(2)).is_none());
    }

    #[test]
    fn the_create_screen_has_its_fields_and_type() {
        let size = [320.0, 240.0];
        let (draws, widgets) = layout(&Screen::CreateGroup, &Screens::default(), &view(), size, [0.0, 0.0], &tw);
        for a in [Action::Focus(Field::Name), Action::Focus(Field::Password), Action::Kind, Action::Create, Action::Back] {
            assert!(widget(&widgets, &a).is_some(), "{a:?}");
        }
        assert!(texts(&draws).contains(&"Type: Normal".to_owned()));
        assert!(on_screen(&widgets, size));
    }
}
