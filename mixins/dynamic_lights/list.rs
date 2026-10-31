// Hook helper: native_mods
fn dynamic_lights_list(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    crate::game::mod_dynamic_lights_glue::on_list(mods);
}
