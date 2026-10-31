// Hook helper: native_settings_click
fn dynamic_lights_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    crate::game::mod_dynamic_lights_glue::on_click(game, mod_id, button);
}
