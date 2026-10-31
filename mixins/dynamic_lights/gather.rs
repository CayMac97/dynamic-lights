// Hook helper: gather_dynamic_lights (cancellable HEAD)
fn dynamic_lights_gather(
    game: &mut crate::game::Game<'_>,
    camera: glam::DVec3,
    alpha: f64,
) -> core::ops::ControlFlow<()> {
    crate::game::mod_dynamic_lights_glue::on_gather(game, camera, alpha)
}
