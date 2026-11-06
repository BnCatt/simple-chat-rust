// The mod's settings page: activation, microphone, volumes and the protocol.
fn simple_chat_rust_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use simple_chat_rust_engine as vc;
    if mod_id != vc::ID {
        return;
    }
    {
        let game = &*game;
        vc::load(&|k| game.native_setting(vc::ID, k));
    }
    buttons.extend(vc::settings_buttons());
}
