use bevy::{prelude::Component, reflect::Reflect};

#[derive(Debug, Clone, Reflect, PartialEq, Eq)]
pub enum Biome {
    Mountain,
    Plains,
    Forest,
    Desert,
    ShallowWater,
    DeepWater,
    Snow,
}

impl Biome {
    pub fn cost(&self) -> Option<u32> {
        match self {
            Biome::Mountain => None,
            Biome::Plains => Some(1),
            Biome::Forest => Some(5),
            Biome::Desert => Some(10),
            Biome::ShallowWater => Some(12),
            Biome::DeepWater => None,
            Biome::Snow => Some(13),
        }
    }

    pub fn from_elevation_and_moisture(elevation: f64, moisture: f64) -> Biome {
        if elevation < 0.0 {
            if moisture < 0.1 {
                Biome::DeepWater
            } else if moisture < 0.2 {
                Biome::ShallowWater
            } else {
                Biome::Plains
            }
        } else if elevation < 0.1 {
            if moisture < 0.33 {
                Biome::ShallowWater
            } else if moisture < 0.66 {
                Biome::Plains
            } else {
                Biome::Forest
            }
        } else if elevation < 0.2 {
            if moisture < 0.16 {
                Biome::ShallowWater
            } else if moisture < 0.33 {
                Biome::Plains
            } else if moisture < 0.66 {
                Biome::Forest
            } else {
                Biome::Mountain
            }
        } else if elevation < 0.3 {
            if moisture < 0.16 {
                Biome::Plains
            } else if moisture < 0.33 {
                Biome::Forest
            } else {
                Biome::Mountain
            }
        } else if elevation < 0.4 {
            if moisture < 0.16 {
                Biome::Forest
            } else {
                Biome::Mountain
            }
        } else {
            Biome::Mountain
        }
    }

    pub fn simple_biome(value: f64) -> Biome {
        if value < 0.1 {
            Biome::Plains
        } else if value < 0.2 {
            Biome::Forest
        } else if value < 0.3 {
            Biome::Desert
        } else if value < 0.4 {
            Biome::Mountain
        } else {
            Biome::Snow
        }
    }
}

#[derive(Debug, Clone, Reflect, PartialEq, Eq)]
pub enum TileResource {
    Wood,
    Stone,
    Iron,
    Nitre,
    Coal,
    Oil,
    Uranium,
    Tea,
    Marble,
    Salt,
    Copper,
    Diamond,
    Ivory,
    Banana,
    Wheat,
    Rice,
    Sugar,
    Spices,
}

// Gotten this way: TileResource::get_from_number(rng.gen_range(0..=25)),

impl TileResource {
    pub fn get_from_number(number: i32) -> Option<Self> {
        match number {
            0 => Some(Self::Wood),
            1 => Some(Self::Stone),
            2 => Some(Self::Iron),
            3 => Some(Self::Nitre),
            4 => Some(Self::Coal),
            5 => Some(Self::Oil),
            6 => Some(Self::Uranium),
            7 => Some(Self::Tea),
            8 => Some(Self::Marble),
            9 => Some(Self::Salt),
            10 => Some(Self::Copper),
            11 => Some(Self::Diamond),
            12 => Some(Self::Ivory),
            13 => Some(Self::Banana),
            14 => Some(Self::Wheat),
            15 => Some(Self::Rice),
            16 => Some(Self::Sugar),
            17 => Some(Self::Spices),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Reflect, PartialEq, Eq)]
pub struct TileAttributes {
    pub production: i32,
    pub science: i32,
    pub attractiveness: i32,
}

impl TileAttributes {
    pub fn production(&self) -> i32 {
        self.production
    }

    pub fn science(&self) -> i32 {
        self.science
    }

    pub fn attractiveness(&self) -> i32 {
        self.attractiveness
    }
}

impl Default for TileAttributes {
    fn default() -> Self {
        Self {
            production: 0,
            science: 0,
            attractiveness: 0,
        }
    }
}

#[derive(Debug, Clone, Component, Reflect, PartialEq, Eq)]
pub struct Tile {
    pub biome: Biome,
    pub attributes: TileAttributes,
    pub strategic_resource: Option<TileResource>,
    pub trade_resource: Option<TileResource>,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            biome: Biome::Plains,
            attributes: TileAttributes::default(),
            strategic_resource: None,
            trade_resource: None,
        }
    }
}

impl Tile {
    pub fn new(
        biome: Biome,
        production: i32,
        science: i32,
        attractiveness: i32,
        special_resource: Option<TileResource>,
        trade_resource: Option<TileResource>,
    ) -> Self {
        Self {
            biome,
            attributes: TileAttributes {
                production,
                science,
                attractiveness,
            },
            strategic_resource: special_resource,
            trade_resource,
        }
    }

    pub fn cost(&self) -> Option<u32> {
        self.biome.cost()
    }
}

#[derive(Component)]
pub struct HexPreviewMarker;

#[derive(Component)]
pub struct Cross;
