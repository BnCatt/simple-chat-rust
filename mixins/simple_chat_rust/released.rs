// The left button coming up on the voice chat's screen: the slider dragged
// keeps its value, stored now.
fn simple_chat_rust_released(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    use simple_chat_rust_engine as vc;
    let _ = screen;
    if game.native_mod_screen() != Some(vc::ID) {
        return;
    }
    *canceled = true;
    if button == 0 {
        vc::menu_release();
        for (k, v) in vc::take_store() {
            game.native_set_setting(vc::ID, &k, &v);
        }
    }
}
