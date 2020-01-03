use generated_enums::Ability
use {FromProto};

#[derive(Debug, Clone)]
pub struct AbilityData {
    available: bool,
    ability: Ability,
    link_name: String,
    link_index: u32,
    button_name: String,
    friendly_name: String,
    hotkey: String,
    remaps_to_ability: Option<Ability>,
    remaps_from_ability: Vec<Ability>,
    target: Option<AbilityTarget>,
    allow_minimap: bool,
    allow_autocast: bool,
    is_building: bool,
    footprint_radius: Option<f32>,
    is_instant_placement: bool,
    cast_range: f32,
}

impl AbilityData {
    /// Get the most generalized id of the ability.
    pub fn get_generalized_ability(&self) -> Ability {
        match self.remaps_to_ability {
            Some(remap) => remap,
            None => self.ability,
        }
    }

    /// Indicates whether the ability is available to the current mods/map.
    pub fn get_available(&self) -> bool {
        self.available
    }

    /// Stable ID for the ability.
    pub fn get_id(&self) -> Ability {
        self.ability
    }

    /// Catalog (game data xml) name of the ability.
    pub fn get_link_name(&self) -> &str {
        &self.link_name
    }

    /// Catalog (game data xml) index of the ability.
    pub fn get_link_index(&self) -> u32 {
        self.link_index
    }

    /// Name of the button for the command card.
    pub fn get_button_name(&self) -> &str {
        &self.button_name
    }
    /// In case the button name is not descriptive.
    pub fn get_friendly_name(&self) -> &str {
        &self.friendly_name
    }
    /// UI hotkey.
    pub fn get_hotkey(&self) -> &str {
        &self.hotkey
    }

    /// Other abilities that can remap to this generic ability.
    pub fn get_remap_abilities(&self) -> &[Ability] {
        &self.remaps_from_ability
    }

    /// Type of target that this ability uses.
    pub fn get_target(&self) -> Option<AbilityTarget> {
        self.target
    }
    /// Can be cast in the minimap (unimplemented).
    pub fn casts_in_minimap(&self) -> bool {
        self.allow_minimap
    }
    /// Autocast can be set.
    pub fn can_autocast(&self) -> bool {
        self.allow_autocast
    }
    /// Requires placement to construct a building.
    pub fn is_building(&self) -> bool {
        self.is_building
    }
    /// If the ability is placing a building, give the radius of the footprint.
    pub fn get_footprint_radius(&self) -> Option<f32> {
        self.footprint_radius
    }
    /// Placement next to an existing structure (an addon like a Tech Lab).
    pub fn is_instant_placement(&self) -> bool {
        self.is_instant_placement
    }
    /// Range unit can cast ability without needing to approach target.
    pub fn get_cast_range(&self) -> f32 {
        self.cast_range
    }
}

impl FromProto<AbilityData> for AbilityData {
    fn from_proto(mut data: AbilityData) -> Result<Self> {
        Ok(Self {
            available: data.get_available(),
            ability: Ability::from_proto(data.get_ability_id())?,
            link_name: data.take_link_name(),
            link_index: data.get_link_index(),
            button_name: data.take_button_name(),
            friendly_name: data.take_friendly_name(),
            hotkey: data.take_hotkey(),
            remaps_to_ability: {
                if data.has_remaps_to_ability_id() {
                    Some(Ability::from_proto(
                        data.get_remaps_to_ability_id(),
                    )?)
                } else {
                    None
                }
            },
            remaps_from_ability: vec![],
            target: match data.get_target() {
                data::AbilityData_Target::None => None,
                data::AbilityData_Target::Point => Some(AbilityTarget::Point),
                data::AbilityData_Target::Unit => Some(AbilityTarget::Unit),
                data::AbilityData_Target::PointOrUnit => {
                    Some(AbilityTarget::PointOrUnit)
                },
                data::AbilityData_Target::PointOrNone => {
                    Some(AbilityTarget::PointOrNone)
                },
            },
            allow_minimap: data.get_allow_minimap(),
            allow_autocast: data.get_allow_autocast(),
            is_building: data.get_is_building(),
            footprint_radius: {
                if data.get_is_building() && data.has_footprint_radius() {
                    Some(data.get_footprint_radius())
                } else {
                    None
                }
            },
            is_instant_placement: data.get_is_instant_placement(),
            cast_range: data.get_cast_range(),
        })
    }
}

struct UnitTypeData{

}

struct UpgradeData{

}

struct GameData{

}

