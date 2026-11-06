//! Every line the voice chat's screens show, in one place for a later
//! translation. English, as Horizon's own interface.

pub const TITLE: &str = "Voice Chat";
pub const SETTINGS: &str = "Voice Chat Settings";
pub const GROUPS: &str = "Groups";
pub const CREATE_GROUP: &str = "Create Group";
pub const VOLUMES: &str = "Adjust Volumes";
pub const SELECT_MICROPHONE: &str = "Select Microphone";
pub const SELECT_SPEAKER: &str = "Select Speaker";

pub const DONE: &str = "Done";
pub const BACK: &str = "Back";
pub const JOIN: &str = "Join";
pub const LEAVE: &str = "Leave";
pub const CANCEL: &str = "Cancel";
pub const CREATE: &str = "Create";
pub const NEW_GROUP: &str = "Create Group...";
pub const DEFAULT_DEVICE: &str = "Default";

pub const NAME: &str = "Name";
pub const PASSWORD: &str = "Password (optional)";
pub const PASSWORD_FOR: &str = "Password";

pub const NO_GROUPS: &str = "No groups yet";
pub const GROUPS_OFF: &str = "Groups are off on this server";
pub const NOT_CONNECTED: &str = "Voice chat isn't connected";
pub const NO_PLAYERS: &str = "No other players with voice chat";
pub const NO_DEVICES: &str = "No other devices found";

pub fn on_off(on: bool) -> &'static str {
    if on { "ON" } else { "OFF" }
}

pub fn microphone_tip(muted: bool) -> String {
    format!("Microphone: {}", if muted { "Muted" } else { "On" })
}

pub fn voice_chat_tip(disabled: bool) -> String {
    format!("Voice chat: {}", if disabled { "Off" } else { "On" })
}

pub const GROUPS_TIP: &str = "Groups";
pub const SETTINGS_TIP: &str = "Settings";

pub fn volume(percent: u32) -> String {
    format!("Voice Volume: {percent}%")
}

pub fn amplification(percent: u32) -> String {
    format!("Microphone Amplification: {percent}%")
}

pub fn threshold(db: i32) -> String {
    format!("Voice Activation Threshold: {db} dB")
}

pub fn activation(voice: bool) -> String {
    format!("Activation Type: {}", if voice { "Voice" } else { "Push To Talk" })
}

pub fn denoiser(on: bool) -> String {
    format!("Noise Suppression: {}", on_off(on))
}

pub const DENOISER_MISSING: &str = "Noise Suppression: N/A";

pub fn microphone(name: Option<&str>) -> String {
    format!("Microphone: {}", name.unwrap_or(DEFAULT_DEVICE))
}

pub fn speaker(name: Option<&str>) -> String {
    format!("Speaker: {}", name.unwrap_or(DEFAULT_DEVICE))
}

pub fn hide_icons(on: bool) -> String {
    format!("Hide Icons: {}", on_off(on))
}

pub fn chat_messages(on: bool) -> String {
    format!("Chat Messages: {}", on_off(on))
}

pub fn group_hud(on: bool) -> String {
    format!("Group HUD: {}", on_off(on))
}

pub fn group_names(on: bool) -> String {
    format!("Group Names: {}", on_off(on))
}

pub fn protocol(name: &str) -> String {
    format!("Protocol: {name}")
}

pub fn group_type(kind: i16) -> String {
    format!("Type: {}", KINDS[kind.rem_euclid(3) as usize])
}

pub const KINDS: [&str; 3] = ["Normal", "Open", "Isolated"];

/// What each group type means, for its button's tooltip.
pub const KIND_TIPS: [&str; 3] = [
    "Members hear each other and the players nearby",
    "Players nearby also hear the group",
    "Members hear only each other",
];

pub fn player_volume(name: &str, percent: u32) -> String {
    format!("{name}: {percent}%")
}

pub fn in_group(name: &str) -> String {
    format!("Group: {name}")
}

pub fn members(n: usize) -> String {
    format!("({n})")
}
