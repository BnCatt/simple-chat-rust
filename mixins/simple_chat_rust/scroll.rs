// The wheel over the voice chat menu scrolls the groups.
fn simple_chat_rust_scroll(game: &crate::game::Game<'_>, scroll: [f64; 2], canceled: &mut bool) {
    use simple_chat_rust_engine as vc;
    if game.native_mod_screen() != Some(vc::ID) {
        return;
    }
    *canceled = true;
    vc::menu_scroll(scroll[1]);
}
