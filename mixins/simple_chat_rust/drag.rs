// The pointer moving with the left button down on the voice chat's screen:
// the slider pressed follows it.
fn simple_chat_rust_drag(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    use simple_chat_rust_engine as vc;
    if game.native_mod_screen() != Some(vc::ID) {
        return;
    }
    *canceled = true;
    if button == 0 {
        vc::menu_drag(screen.mouse);
    }
}
