// Hook helper: native_hud
fn dynamic_lights_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    crate::game::mod_dynamic_lights_glue::on_hud(game, batch, corners);
}
