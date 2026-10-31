// Hook helper: native_settings
fn dynamic_lights_settings(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    buttons: &mut Vec<String>,
) {
    crate::game::mod_dynamic_lights_glue::on_settings(game, mod_id, buttons);
}
