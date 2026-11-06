// Characters typed on the voice chat menu go into the focused field.
fn simple_chat_rust_chars(game: &crate::game::Game<'_>, c: char, canceled: &mut bool) {
    use simple_chat_rust_engine as vc;
    if game.native_mod_screen() != Some(vc::ID) {
        return;
    }
    *canceled = true;
    vc::menu_char(c);
}
