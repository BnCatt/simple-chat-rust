// A play packet from the server (the `client_packet` hook point): the voice
// chat's custom payloads go to the voice chat. Read-only: the game handles the
// packet as it would without the mod.
fn simple_chat_rust_packet(packet: &horizon_protocol::packets::play::Clientbound) {
    if let horizon_protocol::packets::play::Clientbound::CustomPayload(p) = packet {
        simple_chat_rust_engine::payload(&p.channel, &p.data);
    }
}
