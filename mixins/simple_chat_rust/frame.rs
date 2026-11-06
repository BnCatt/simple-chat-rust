// Off a server (the title screen, single player, a lost connection): the
// voice chat hangs up. And the menu closes when it asked to, and forgets a
// close the game did (Escape, another screen).
fn simple_chat_rust_frame(game: &mut crate::game::Game<'_>) {
    use simple_chat_rust_engine as vc;
    let on_server = crate::game::remote::simple_chat_rust_world::on_server(game);
    if !on_server || !game.native_on(vc::ID) {
        vc::tick(None, &|_| None);
    }
    if vc::menu_take_close() {
        game.native_mod_screen_close(vc::ID);
    }
    vc::menu_sync(game.native_mod_screen() == Some(vc::ID));
}
