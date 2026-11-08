// What voice chat needs from the game that the native hook points don't give
// a client yet: the address the player joined, where the other players stand
// (each tick for the audio, each frame for the icons) and the camera the last
// frame was drawn from. Everything else goes through the
// hook points. This module is the mod's only reach into the game's own state,
// so a Horizon that moves these fields breaks the build here and nowhere else.
// (A `native_client_players` / `native_server_address` hook would retire it.)
//
// Added to `game::remote` (`horizon-mixin.toml`, `[[module]]`), whose private
// fields its children can read.

/// On a server and in its world: joined, and not reconfiguring.
pub(crate) fn on_server(game: &crate::game::Game<'_>) -> bool {
    game.remote
        .as_ref()
        .is_some_and(|r| r.joined && r.reconfiguring.is_none())
}

/// The address the player joined, as typed (`host` or `host:port`).
pub(crate) fn address(game: &crate::game::Game<'_>) -> Option<String> {
    game.remote
        .as_ref()
        .filter(|_| on_server(game))
        .map(|r| r.session.address.clone())
}

/// The other players in the level: UUID and feet, from one pass over the mobs.
pub(crate) fn players(game: &crate::game::Game<'_>) -> Vec<(u128, [f64; 3])> {
    let Some(r) = game.remote.as_ref() else {
        return Vec::new();
    };
    if r.level_players.is_empty() {
        return Vec::new();
    }
    game.mobs
        .iter()
        .filter_map(|m| {
            let uuid = r.level_players.get(&i32::try_from(m.id).ok()?)?;
            Some((*uuid, [m.position.x, m.position.y, m.position.z]))
        })
        .collect()
}

/// The other players as this frame draws them: UUID and eyes, between the last
/// tick's position and this one's by the world's partial tick, as the game
/// draws their models and name tags. (The tick's positions step 20 times a
/// second: an icon placed from them jumps along a walking player's name.)
/// The eyes are the name tag's attachment (the player's height, lower when
/// sneaking) less 0.18, where a standing player's eyes are, so the engine's
/// "above the eyes" offsets land where vanilla puts the tag.
pub(crate) fn frame_players(game: &crate::game::Game<'_>) -> Vec<(u128, [f64; 3])> {
    let Some(r) = game.remote.as_ref() else {
        return Vec::new();
    };
    if r.level_players.is_empty() {
        return Vec::new();
    }
    let alpha = game.clock.fraction();
    game.mobs
        .iter()
        .filter_map(|m| {
            let uuid = r.level_players.get(&i32::try_from(m.id).ok()?)?;
            let at = m.previous.lerp(m.position, alpha);
            let (_, height, _) = m.dims();
            Some((*uuid, [at.x, at.y + height - 0.18, at.z]))
        })
        .collect()
}

/// A player's name tag as the game draws it (`game/names.rs`): the shown
/// name's width in text pixels (`getDisplayName`: the team's prefix, the name
/// and the team's suffix, so a rank such as `[Admin] ` widens it), and whether
/// a below-name score goes under it, which puts the name a line up near by.
pub(crate) fn name_tag(game: &crate::game::Game<'_>, name: &str) -> (f32, bool) {
    use horizon_client::{scoreboard as sb, text::Text};
    let board = &game.score.board;
    let member = Text::literal(name);
    let shown = match board.players_team(name) {
        Some(team) => team.formatted_name(member),
        None => member,
    }
    .resolve_legacy();
    let width = game.gui.atlas.font.width_text(&shown);
    let score = board
        .display_objective(sb::SLOT_BELOW_NAME)
        .is_some_and(|o| board.score(name, &o.name).is_some());
    (width, score)
}

/// The camera the world was last drawn from: position, yaw, pitch and field of
/// view. Set after the HUD is built, so one frame old in the HUD hook; only a
/// stand-in for when `native_camera` (this frame's) is missing.
pub(crate) fn camera(game: &crate::game::Game<'_>) -> Option<([f64; 3], f32, f32, f32)> {
    game.locator_camera
}
