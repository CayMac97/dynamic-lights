// Glue between Horizon's `Game` and `mod_dynamic_lights` runtime state.
use crate::game::{Game, shield};
use crate::game::mod_dynamic_lights as m;
use crate::game::native_hooks::NativeMod;
use core::ops::ControlFlow;
use glam::DVec3;
use horizon_client::{
    blocks::Fluid,
    dynamic_light::{self, BURNING, Luminance, Source},
    extras::Corner,
    extras_hud::Corners,
    hud::HudBatch,
};

fn is_pos_in_water(game: &Game<'_>, pos: DVec3) -> bool {
    let x = pos.x.floor() as i32;
    let y = pos.y.floor() as i32;
    let z = pos.z.floor() as i32;
    let facts = game.res.facts().get(game.world.block(x, y, z));
    facts.fluid == Fluid::Water || facts.waterlogged
}

pub(super) fn on_gather(game: &mut Game<'_>, camera: DVec3, alpha: f64) -> ControlFlow<()> {
    if !game.native_on(m::ID) {
        return ControlFlow::Continue(());
    }
    let s = {
        let g = &*game;
        m::settings(|key| g.native_setting(m::ID, key))
    };

    let mut out = std::mem::take(&mut game.dynamic_lights);
    out.clear();
    if !s.enabled {
        m::set_stats(0, 0);
        game.dynamic_lights = out;
        return ControlFlow::Break(());
    }

    if game.luminance.is_none() {
        game.luminance = Some(Luminance::new(game.res.facts(), game.pack.assets.registry));
    }

    let mut sources = Vec::new();
    let mut player_level: u8 = 0;

    // 1. Local Player (main hand, offhand, and burning state)
    if let Some(p) = &game.player
        && !p.spectator
    {
        let feet = game
            .previous_position
            .lerp(p.position, game.clock.camera_fraction());
        let hand_pos = DVec3::new(feet.x, feet.y + 1.2, feet.z);
        let in_water = is_pos_in_water(game, hand_pos);
        let lum = game.luminance.as_ref().expect("luminance initialized");

        let mut level: u8 = 0;
        if s.held_items {
            if let Some(main_stack) = game.stack_in(game.selected) {
                let id = &*main_stack.item;
                level = level.max(m::item_effective_luminance(
                    id,
                    lum.of(id),
                    in_water,
                    s.water_sensitive,
                ));
            }
            if let Some(off_stack) = game.stack_in(shield::OFFHAND_SLOT) {
                let id = &*off_stack.item;
                level = level.max(m::item_effective_luminance(
                    id,
                    lum.of(id),
                    in_water,
                    s.water_sensitive,
                ));
            }
        }
        if s.entities && p.fire_ticks > 0 {
            level = level.max(BURNING);
        }
        player_level = level;
        if level > 0 {
            sources.push(Source {
                at: hand_pos,
                level,
            });
        }
    }

    // 2. Mobs (held items, glowing species, Creeper swell, and burning state)
    for mob in game.mobs.iter().filter(|mob| !mob.removed) {
        let at_vec3 = mob.previous.lerp(mob.position, alpha);
        let at = DVec3::new(
            at_vec3.x,
            at_vec3.y + f64::from(mob.height()) * 0.6,
            at_vec3.z,
        );
        let in_water = is_pos_in_water(game, at);
        let lum = game.luminance.as_ref().expect("luminance initialized");

        let mut level: u8 = 0;
        if s.held_items {
            if let Some(main_id) = mob.extra.main_hand.as_deref() {
                level = level.max(m::item_effective_luminance(
                    main_id,
                    lum.of(main_id),
                    in_water,
                    s.water_sensitive,
                ));
            }
            if let Some(off_id) = mob.extra.off_hand.as_deref() {
                level = level.max(m::item_effective_luminance(
                    off_id,
                    lum.of(off_id),
                    in_water,
                    s.water_sensitive,
                ));
            }
        }
        if s.entities {
            level = level.max(m::mob_intrinsic_luminance(
                &mob.kind.id,
                mob.swell,
                mob.fire_ticks,
            ));
        }
        if level > 0 {
            sources.push(Source { at, level });
        }
    }

    // 3. Dropped Items
    if s.dropped_items {
        for item in &game.dropped {
            let at_vec3 = item.previous.lerp(item.position, alpha);
            let at = DVec3::new(at_vec3.x, at_vec3.y + 0.25, at_vec3.z);
            let in_water = is_pos_in_water(game, at);
            let lum = game.luminance.as_ref().expect("luminance initialized");
            let id = &*item.stack.item;
            let level = m::item_effective_luminance(id, lum.of(id), in_water, s.water_sensitive)
                .max(if item.fire_ticks > 0 { BURNING } else { 0 });
            if level > 0 {
                sources.push(Source { at, level });
            }
        }
    }

    dynamic_light::select(&sources, camera, &mut out);
    m::set_stats(out.len(), player_level);
    game.dynamic_lights = out;
    ControlFlow::Break(())
}

pub(super) fn on_key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !pressed || !game.native_on(m::ID) || !game.native_playing() {
        return;
    }
    let toggle_key = game.native_mod_key(m::ID, m::KEY_TOGGLE, m::KEY_TOGGLE_DEFAULT);
    if key != toggle_key {
        return;
    }
    let mut s = {
        let g = &*game;
        m::settings(|k| g.native_setting(m::ID, k))
    };
    s.enabled = !s.enabled;
    m::set_settings(s);
    for (k, v) in s.stored() {
        game.native_set_setting(m::ID, k, &v);
    }
    let state_label = if s.enabled { "ON" } else { "OFF" };
    game.native_chat(&format!("[Dynamic Lights] Toggled {state_label}"));
}

pub(super) fn on_hud(game: &mut Game<'_>, batch: &mut HudBatch, corners: &mut Corners) {
    if !game.native_on(m::ID) {
        return;
    }
    let s = {
        let g = &*game;
        m::settings(|k| g.native_setting(m::ID, k))
    };
    if !s.show_hud || !game.native_corner_shown(Corner::TopLeft) {
        return;
    }
    let (count, player_lvl) = m::stats();
    let text = if s.enabled {
        format!("Dynamic Lights: ON ({count} src, Hand L{player_lvl})")
    } else {
        "Dynamic Lights: OFF".to_owned()
    };
    let color = if s.enabled { m::COLOR_ON } else { m::COLOR_OFF };
    let w = game.native_text_width(&text) + 8.0;
    let h = 14.0;
    let (x, y) = corners.place(Corner::TopLeft, w, h);
    game.native_fill(batch, [x, y, w, h], m::BACKGROUND);
    game.native_text(batch, &text, [x + 4.0, y + 3.0], color, true);
}

pub(super) fn on_list(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: vec![[
            m::KEY_TOGGLE.into(),
            m::KEY_TOGGLE_LABEL.into(),
            m::KEY_TOGGLE_DEFAULT.into(),
        ]],
    });
}

pub(super) fn on_settings(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id != m::ID {
        return;
    }
    let g = &*game;
    let settings = m::settings(|key| g.native_setting(m::ID, key));
    buttons.extend(settings.labels());
}

pub(super) fn on_click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    if mod_id != m::ID {
        return;
    }
    let mut settings = {
        let g = &*game;
        m::settings(|key| g.native_setting(m::ID, key))
    };
    settings.click(button);
    m::set_settings(settings);
    for (key, value) in settings.stored() {
        game.native_set_setting(m::ID, key, &value);
    }
}
