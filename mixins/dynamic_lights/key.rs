// Hook helper: native_key
fn dynamic_lights_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::mod_dynamic_lights_glue::on_key(game, key, pressed);
}
