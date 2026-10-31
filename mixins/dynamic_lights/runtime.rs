// Dynamic Lights runtime state & pure logic.
// Compiles both standalone (`cargo test`) and inside `horizon-client` via `horizon-mixins`.

use std::sync::Mutex;

pub const ID: &str = "dynamic_lights";
pub const NAME: &str = "Dynamic Lights";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "1:1 LambDynamicLights: handheld torches, lanterns, glowing items, dropped items, and glowing/burning entities illuminate their surroundings with water-sensitive extinguishing.";

pub const KEY_TOGGLE: &str = "toggle";
pub const KEY_TOGGLE_LABEL: &str = "Toggle Dynamic Lights";
pub const KEY_TOGGLE_DEFAULT: &str = "key.keyboard.l";

pub const BACKGROUND: u32 = 0x7800_0000;
pub const COLOR_ON: u32 = 0x00FF_CC44;
pub const COLOR_OFF: u32 = 0x00AA_AAAA;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub enabled: bool,
    pub held_items: bool,
    pub dropped_items: bool,
    pub entities: bool,
    pub water_sensitive: bool,
    pub show_hud: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            held_items: true,
            dropped_items: true,
            entities: true,
            water_sensitive: true,
            show_hud: false,
        }
    }
}

impl Settings {
    pub fn load<'a>(mut get: impl FnMut(&str) -> Option<String> + 'a) -> Self {
        let mut s = Self::default();
        if let Some(v) = get("enabled") {
            s.enabled = v != "false" && v != "0";
        }
        if let Some(v) = get("held_items") {
            s.held_items = v != "false" && v != "0";
        }
        if let Some(v) = get("dropped_items") {
            s.dropped_items = v != "false" && v != "0";
        }
        if let Some(v) = get("entities") {
            s.entities = v != "false" && v != "0";
        }
        if let Some(v) = get("water_sensitive") {
            s.water_sensitive = v != "false" && v != "0";
        }
        if let Some(v) = get("show_hud") {
            s.show_hud = v == "true" || v == "1";
        }
        s
    }

    pub fn stored(&self) -> [(&'static str, String); 6] {
        let b = |v: bool| if v { "true" } else { "false" }.to_owned();
        [
            ("enabled", b(self.enabled)),
            ("held_items", b(self.held_items)),
            ("dropped_items", b(self.dropped_items)),
            ("entities", b(self.entities)),
            ("water_sensitive", b(self.water_sensitive)),
            ("show_hud", b(self.show_hud)),
        ]
    }

    pub fn labels(&self) -> Vec<String> {
        let on_off = |v: bool| if v { "ON" } else { "OFF" };
        vec![
            format!("Dynamic Lights: {}", on_off(self.enabled)),
            format!("Held Item Lighting: {}", on_off(self.held_items)),
            format!("Dropped Item Lighting: {}", on_off(self.dropped_items)),
            format!("Glowing & Burning Entities: {}", on_off(self.entities)),
            format!("Water-Sensitive Check: {}", on_off(self.water_sensitive)),
            format!("Show HUD Badge: {}", on_off(self.show_hud)),
        ]
    }

    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.enabled = !self.enabled,
            1 => self.held_items = !self.held_items,
            2 => self.dropped_items = !self.dropped_items,
            3 => self.entities = !self.entities,
            4 => self.water_sensitive = !self.water_sensitive,
            5 => self.show_hud = !self.show_hud,
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RuntimeState {
    pub loaded: bool,
    pub settings: Settings,
    pub active_sources: usize,
    pub player_light_level: u8,
}

static STATE: Mutex<RuntimeState> = Mutex::new(RuntimeState {
    loaded: false,
    settings: Settings {
        enabled: true,
        held_items: true,
        dropped_items: true,
        entities: true,
        water_sensitive: true,
        show_hud: false,
    },
    active_sources: 0,
    player_light_level: 0,
});

pub fn settings<'a>(get: impl FnMut(&str) -> Option<String> + 'a) -> Settings {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    if !guard.loaded {
        guard.settings = Settings::load(get);
        guard.loaded = true;
    }
    guard.settings
}

pub fn set_settings(s: Settings) {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    guard.settings = s;
    guard.loaded = true;
}

pub fn set_stats(active_sources: usize, player_light_level: u8) {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    guard.active_sources = active_sources;
    guard.player_light_level = player_light_level;
}

pub fn stats() -> (usize, u8) {
    let guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    (guard.active_sources, guard.player_light_level)
}

/// Returns true if an item is water-sensitive in LambDynamicLights (extinguished underwater).
/// Waterproof blocks like lanterns, sea lanterns, glowstone, shroomlights, froglights,
/// jack o'lanterns, conduits, sea pickles, and end rods remain lit underwater.
pub fn is_water_sensitive_item(item_id: &str) -> bool {
    let short = item_id.strip_prefix("minecraft:").unwrap_or(item_id);
    matches!(
        short,
        "torch"
            | "soul_torch"
            | "redstone_torch"
            | "campfire"
            | "soul_campfire"
            | "lava_bucket"
            | "fire_charge"
            | "blaze_rod"
            | "blaze_powder"
    ) || short.ends_with("candle")
}

/// Extra LambDynamicLights item luminance values for non-block items that glow when held/dropped.
pub fn extra_item_luminance(item_id: &str) -> u8 {
    let short = item_id.strip_prefix("minecraft:").unwrap_or(item_id);
    match short {
        "lava_bucket" | "beacon" | "end_gateway" | "end_portal" => 15,
        "nether_star" => 12,
        "blaze_rod" | "glow_ink_sac" | "fire_charge" | "glow_berries" => 10,
        "glowstone_dust" | "spectral_arrow" | "prismarine_crystals" => 8,
        "blaze_powder" | "ender_eye" => 7,
        "magma_cream" | "totem_of_undying" | "experience_bottle" | "dragon_breath" => 6,
        _ => 0,
    }
}

/// Computes the effective dynamic light level (`0..=15`) of a held or dropped item,
/// respecting LambDynamicLights' water-sensitive extinguishing rule.
pub fn item_effective_luminance(
    item_id: &str,
    base_block_luminance: u8,
    in_water: bool,
    water_sensitive_enabled: bool,
) -> u8 {
    if in_water && water_sensitive_enabled && is_water_sensitive_item(item_id) {
        return 0;
    }
    base_block_luminance.max(extra_item_luminance(item_id)).min(15)
}

/// Computes the intrinsic dynamic light level (`0..=15`) of a mob entity based on its species,
/// Creeper swell timer, and burning state (`fire_ticks`).
pub fn mob_intrinsic_luminance(mob_id: &str, swell: i32, fire_ticks: i32) -> u8 {
    let short = mob_id.strip_prefix("minecraft:").unwrap_or(mob_id);
    let species_lum: u8 = match short {
        "blaze" | "glow_squid" => 10,
        "magma_cube" => 8,
        "allay" | "vex" => 7,
        "warden" => 5,
        _ => 0,
    };
    let swell_lum: u8 = if swell > 0 {
        (10 + (swell / 3)).clamp(10, 15) as u8
    } else {
        0
    };
    let fire_lum: u8 = if fire_ticks > 0 { 15 } else { 0 };
    species_lum.max(swell_lum).max(fire_lum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_sensitive_items_extinguish_underwater_while_waterproof_items_stay_lit() {
        // Torch (14) extinguishes underwater when water_sensitive = true
        assert_eq!(item_effective_luminance("minecraft:torch", 14, false, true), 14);
        assert_eq!(item_effective_luminance("minecraft:torch", 14, true, true), 0);
        // If water_sensitive setting is disabled, torch stays lit
        assert_eq!(item_effective_luminance("minecraft:torch", 14, true, false), 14);

        // Lantern (15) and Sea Lantern (15) stay lit underwater!
        assert_eq!(item_effective_luminance("minecraft:lantern", 15, true, true), 15);
        assert_eq!(
            item_effective_luminance("minecraft:sea_lantern", 15, true, true),
            15
        );

        // Non-block glowing items work and obey water sensitivity
        assert_eq!(
            item_effective_luminance("minecraft:blaze_rod", 0, false, true),
            10
        );
        assert_eq!(
            item_effective_luminance("minecraft:blaze_rod", 0, true, true),
            0
        );
        assert_eq!(
            item_effective_luminance("minecraft:glow_ink_sac", 0, true, true),
            10
        );
    }

    #[test]
    fn glowing_and_burning_mobs_emit_expected_light_levels() {
        assert_eq!(mob_intrinsic_luminance("minecraft:blaze", 0, 0), 10);
        assert_eq!(mob_intrinsic_luminance("minecraft:glow_squid", 0, 0), 10);
        assert_eq!(mob_intrinsic_luminance("minecraft:magma_cube", 0, 0), 8);
        assert_eq!(mob_intrinsic_luminance("minecraft:allay", 0, 0), 7);
        assert_eq!(mob_intrinsic_luminance("minecraft:zombie", 0, 0), 0);
        assert_eq!(mob_intrinsic_luminance("minecraft:zombie", 0, 20), 15);
        assert_eq!(mob_intrinsic_luminance("minecraft:creeper", 15, 0), 15);
    }

    #[test]
    fn settings_persistence_roundtrip() {
        let mut s = Settings::default();
        s.click(0);
        s.click(5);
        assert!(!s.enabled);
        assert!(s.show_hud);
        let map: std::collections::HashMap<_, _> = s.stored().into_iter().collect();
        let loaded = Settings::load(|k| map.get(k).cloned());
        assert_eq!(s, loaded);
    }
}
