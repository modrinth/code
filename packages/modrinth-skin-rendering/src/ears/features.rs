use crate::texture::Texture;

macro_rules! feature_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[cfg_attr(
            feature = "serde",
            derive(serde::Serialize, serde::Deserialize),
            serde(rename_all = "snake_case")
        )]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            const ALL: &[Self] = &[$(Self::$variant),+];
        }
    };
}

feature_enum!(EarMode {
    None,
    Above,
    Sides,
    Behind,
    Around,
    Floppy,
    Cross,
    Out,
    Tall,
    TallCross,
});
feature_enum!(EarAnchor {
    Center,
    Front,
    Back
});
feature_enum!(TailMode {
    None,
    Down,
    Back,
    Up,
    Vertical,
    Cross,
    CrossOverlap,
    Star,
    StarOverlap,
});
feature_enum!(WingMode {
    None,
    SymmetricDual,
    SymmetricSingle,
    AsymmetricL,
    AsymmetricR,
    AsymmetricDual,
    Flat,
});
feature_enum!(WingAnimationMode {
    Normal,
    None,
    NoFlight
});
feature_enum!(LegMode {
    Plantigrade,
    DigitigradePartial,
    DigitigradeFull
});
feature_enum!(Protrusions {
    None,
    Horn,
    Claws,
    ClawsAndHorn,
    Halo,
    DoubleHalo,
    ClawsAndHalo,
    ClawsAndDoubleHalo,
});

impl Protrusions {
    pub const fn has_claws(self) -> bool {
        matches!(
            self,
            Self::Claws
                | Self::ClawsAndHorn
                | Self::ClawsAndHalo
                | Self::ClawsAndDoubleHalo
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Features {
    pub ear_mode: EarMode,
    pub ear_anchor: EarAnchor,
    pub protrusions: Protrusions,
    pub tail_mode: TailMode,
    pub tail_segments: u32,
    pub tail_bends: [f32; 4],
    pub snout_offset: u32,
    pub snout_width: u32,
    pub snout_height: u32,
    pub snout_depth: u32,
    pub chest_size: f32,
    pub wing_mode: WingMode,
    pub wing_animation_mode: WingAnimationMode,
    pub cape_enabled: bool,
    pub emissive: bool,
    pub leg_mode: LegMode,
    pub animate_tail: bool,
    pub swap_jacket_back_and_tail: bool,
}

const V0_MAGIC: u32 = 0x3f23d8;
const V1_MAGIC: u32 = 0xea2501;

const BLUE: u32 = 0x3f23d8;
const GREEN: u32 = 0x23d848;
const RED: u32 = 0xd82350;
const PURPLE: u32 = 0xb923d8;
const CYAN: u32 = 0x23d8c6;
const ORANGE: u32 = 0xd87823;
const PINK: u32 = 0xd823b7;
const PURPLE2: u32 = 0xd823ff;
const WHITE: u32 = 0xfefdf2;
const GRAY: u32 = 0x5e605a;

pub(super) fn rgb(pixel: [u8; 4]) -> u32 {
    (u32::from(pixel[0]) << 16)
        | (u32::from(pixel[1]) << 8)
        | u32::from(pixel[2])
}

fn config_pixel(texture: &Texture, index: u32) -> [u8; 4] {
    texture.get(index % 4, 32 + index / 4)
}

pub(super) struct BitReader<'a> {
    data: &'a [u8],
    bit_index: usize,
}

impl<'a> BitReader<'a> {
    pub(super) const fn new(data: &'a [u8]) -> Self {
        Self { data, bit_index: 0 }
    }

    pub(super) const fn remaining_bits(&self) -> usize {
        self.data.len() * 8 - self.bit_index
    }

    pub(super) fn read(&mut self, bits: u32) -> Option<u32> {
        if self.remaining_bits() < bits as usize {
            return None;
        }
        let mut result = 0;
        for _ in 0..bits {
            let byte = self.data[self.bit_index / 8];
            let shift = 7 - (self.bit_index % 8);
            result = (result << 1) | u32::from((byte >> shift) & 1);
            self.bit_index += 1;
        }
        Some(result)
    }

    fn read_bool(&mut self) -> Option<bool> {
        Some(self.read(1)? == 1)
    }

    fn read_unit(&mut self, bits: u32) -> Option<f32> {
        Some(self.read(bits)? as f32 / ((1 << bits) - 1) as f32)
    }

    fn read_signed_unit(&mut self, bits: u32) -> Option<f32> {
        let negative = self.read_bool()?;
        let value = self.read_unit(bits)?;
        Some(if negative { -value } else { value })
    }
}

fn lookup<T: Copy>(values: &[T], index: u32, fallback: T) -> T {
    values.get(index as usize).copied().unwrap_or(fallback)
}

fn parse_v1(texture: &Texture) -> Option<Features> {
    let mut bytes = Vec::with_capacity(45);
    for y in 0..4 {
        for x in 0..4 {
            if x == 0 && y == 0 {
                continue;
            }
            bytes.extend_from_slice(&texture.get(x, 32 + y)[..3]);
        }
    }

    let mut reader = BitReader::new(&bytes);
    let version = reader.read(8)?;

    let ears = reader.read(6)?;
    let (ear_mode, ear_anchor) = if ears == 0 {
        (EarMode::None, EarAnchor::Center)
    } else {
        (
            lookup(EarMode::ALL, (ears - 1) / 3 + 1, EarMode::None),
            lookup(EarAnchor::ALL, (ears - 1) % 3, EarAnchor::Center),
        )
    };
    let mut protrusions_index = reader.read(2)?;
    let tail_index = reader.read(3)?;
    let mut tail_mode = lookup(TailMode::ALL, tail_index, TailMode::None);
    let mut tail_segments = 0;
    let mut tail_bends = [0.0; 4];

    if tail_index != 0 {
        tail_segments = reader.read(2)? + 1;
        for bend in tail_bends.iter_mut().take(tail_segments as usize) {
            *bend = reader.read_signed_unit(6)? * 90.0;
        }
    }

    let snout_width = reader.read(3)?;
    let mut snout_height = 0;
    let mut snout_depth = 0;
    let mut snout_offset = 0;
    if snout_width > 0 {
        snout_height = reader.read(2)? + 1;
        snout_depth = reader.read(3)? + 1;
        snout_offset = reader.read(3)?.min(8 - snout_height);
    }

    let chest_size = reader.read_unit(5)?;
    let wing_mode = lookup(WingMode::ALL, reader.read(3)?, WingMode::None);
    let animate_wings = if wing_mode == WingMode::None {
        false
    } else {
        reader.read_bool()?
    };
    let cape_enabled = reader.read_bool()?;
    let emissive = reader.read_bool()?;
    let mut wing_animation_mode = if animate_wings {
        WingAnimationMode::Normal
    } else {
        WingAnimationMode::None
    };
    let mut leg_mode = LegMode::Plantigrade;
    let mut animate_tail = true;
    let mut swap_jacket_back_and_tail = false;

    if version >= 1 && tail_index == 7 {
        tail_mode =
            lookup(TailMode::ALL, tail_index + reader.read(3)?, TailMode::None);
    }
    if version >= 2 {
        leg_mode = lookup(LegMode::ALL, reader.read(3)?, LegMode::Plantigrade);
        if wing_mode != WingMode::None {
            wing_animation_mode = lookup(
                WingAnimationMode::ALL,
                reader.read(3)?,
                wing_animation_mode,
            );
        }
        animate_tail = reader.read_bool()?;
        swap_jacket_back_and_tail = reader.read_bool()?;
    }
    if version >= 3 {
        protrusions_index |= reader.read(2)? << 2;
    }

    Some(Features {
        ear_mode,
        ear_anchor,
        protrusions: lookup(
            Protrusions::ALL,
            protrusions_index,
            Protrusions::None,
        ),
        tail_mode,
        tail_segments,
        tail_bends,
        snout_offset,
        snout_width,
        snout_height,
        snout_depth,
        chest_size,
        wing_mode,
        wing_animation_mode,
        cape_enabled,
        emissive,
        leg_mode,
        animate_tail,
        swap_jacket_back_and_tail,
    })
}

fn pixel_value_to_unit(value: u8) -> f32 {
    if value == 0 {
        return 0.0;
    }
    let mut shifted = i32::from(value) - 128;
    shifted += if shifted < 0 { -1 } else { 1 };
    shifted as f32 / 128.0
}

fn by_pixel<T: Copy>(table: &[(u32, T)], pixel: [u8; 4], fallback: T) -> T {
    let color = rgb(pixel);
    table
        .iter()
        .find(|(key, _)| *key == color)
        .map_or(fallback, |(_, value)| *value)
}

fn parse_v0(texture: &Texture) -> Features {
    let ear_mode = by_pixel(
        &[
            (RED, EarMode::None),
            (BLUE, EarMode::Above),
            (GREEN, EarMode::Sides),
            (PURPLE, EarMode::Behind),
            (CYAN, EarMode::Around),
            (ORANGE, EarMode::Floppy),
            (PINK, EarMode::Cross),
            (PURPLE2, EarMode::Out),
            (WHITE, EarMode::Tall),
            (GRAY, EarMode::TallCross),
        ],
        config_pixel(texture, 1),
        EarMode::None,
    );
    let ear_anchor = by_pixel(
        &[
            (BLUE, EarAnchor::Center),
            (GREEN, EarAnchor::Front),
            (RED, EarAnchor::Back),
        ],
        config_pixel(texture, 2),
        EarAnchor::Center,
    );
    let protrusions = by_pixel(
        &[
            (BLUE, Protrusions::None),
            (RED, Protrusions::None),
            (GREEN, Protrusions::Claws),
            (PURPLE, Protrusions::Horn),
            (CYAN, Protrusions::ClawsAndHorn),
            (WHITE, Protrusions::Halo),
            (GRAY, Protrusions::DoubleHalo),
            (PURPLE2, Protrusions::ClawsAndHalo),
            (PINK, Protrusions::ClawsAndDoubleHalo),
        ],
        config_pixel(texture, 3),
        Protrusions::None,
    );
    let tail_mode = by_pixel(
        &[
            (RED, TailMode::None),
            (BLUE, TailMode::Down),
            (GREEN, TailMode::Back),
            (PURPLE, TailMode::Up),
            (ORANGE, TailMode::Vertical),
            (PINK, TailMode::Cross),
            (PURPLE2, TailMode::CrossOverlap),
            (WHITE, TailMode::Star),
            (GRAY, TailMode::StarOverlap),
        ],
        config_pixel(texture, 4),
        TailMode::None,
    );

    let tail_pixel = config_pixel(texture, 5);
    let mut tail_bends = [0.0; 4];
    let mut tail_segments = 0;
    if rgb(tail_pixel) != BLUE {
        tail_bends = [
            pixel_value_to_unit(255 - tail_pixel[3]) * 90.0,
            pixel_value_to_unit(tail_pixel[0]) * 90.0,
            pixel_value_to_unit(tail_pixel[1]) * 90.0,
            pixel_value_to_unit(tail_pixel[2]) * 90.0,
        ];
        tail_segments = 1;
        for (index, bend) in tail_bends.iter().enumerate().skip(1) {
            if *bend != 0.0 {
                tail_segments = index as u32 + 1;
            }
        }
    }

    let snout_pixel = config_pixel(texture, 6);
    let etc_pixel = config_pixel(texture, 7);
    let mut snout_width = 0;
    let mut snout_height = 0;
    let mut snout_depth = 0;
    let mut snout_offset = 0;
    if rgb(snout_pixel) != BLUE {
        snout_width = u32::from(snout_pixel[0]).min(7);
        snout_height = u32::from(snout_pixel[1]).min(4);
        snout_depth = u32::from(snout_pixel[2]).min(8);
        snout_offset = u32::from(etc_pixel[1]).min(8 - snout_height);
    }

    let etc_is_default = rgb(etc_pixel) == BLUE;
    let bitflags = rgb(config_pixel(texture, 12));
    let bitflags_are_default = bitflags == BLUE;

    Features {
        ear_mode,
        ear_anchor,
        protrusions,
        tail_mode,
        tail_segments,
        tail_bends,
        snout_offset,
        snout_width,
        snout_height,
        snout_depth,
        chest_size: if etc_is_default {
            0.0
        } else {
            (f32::from(etc_pixel[0]) / 128.0).min(1.0)
        },
        wing_mode: by_pixel(
            &[
                (BLUE, WingMode::None),
                (RED, WingMode::None),
                (PINK, WingMode::SymmetricDual),
                (GREEN, WingMode::SymmetricSingle),
                (CYAN, WingMode::AsymmetricL),
                (ORANGE, WingMode::AsymmetricR),
                (PURPLE, WingMode::AsymmetricDual),
                (PURPLE2, WingMode::Flat),
            ],
            config_pixel(texture, 8),
            WingMode::None,
        ),
        wing_animation_mode: by_pixel(
            &[
                (BLUE, WingAnimationMode::Normal),
                (RED, WingAnimationMode::None),
                (GREEN, WingAnimationMode::NoFlight),
            ],
            config_pixel(texture, 9),
            WingAnimationMode::Normal,
        ),
        cape_enabled: !etc_is_default && (etc_pixel[2] & 16) != 0,
        emissive: rgb(config_pixel(texture, 10)) == ORANGE,
        leg_mode: by_pixel(
            &[
                (BLUE, LegMode::Plantigrade),
                (GREEN, LegMode::DigitigradePartial),
                (PINK, LegMode::DigitigradeFull),
            ],
            config_pixel(texture, 11),
            LegMode::Plantigrade,
        ),
        animate_tail: bitflags_are_default || (bitflags & 1) != 0,
        swap_jacket_back_and_tail: !bitflags_are_default && (bitflags & 2) != 0,
    }
}

pub(super) fn parse(texture: &Texture) -> Option<Features> {
    match rgb(texture.get(0, 32)) {
        V0_MAGIC => Some(parse_v0(texture)),
        V1_MAGIC => parse_v1(texture),
        _ => None,
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    pub(in crate::ears) fn color(value: u32) -> [u8; 4] {
        [(value >> 16) as u8, (value >> 8) as u8, value as u8, 255]
    }

    pub(in crate::ears) fn v0_skin(config: &[(u32, [u8; 4])]) -> Texture {
        let mut texture = Texture::new(64, 64);
        for index in 0..16 {
            texture.set(index % 4, 32 + index / 4, color(BLUE));
        }
        for &(index, pixel) in config {
            texture.set(index % 4, 32 + index / 4, pixel);
        }
        texture
    }

    pub(in crate::ears) fn v1_skin(bits: &[(u32, u32)]) -> Texture {
        let mut stream = Vec::new();
        for &(value, width) in bits {
            for bit in (0..width).rev() {
                stream.push((value >> bit) & 1 == 1);
            }
        }
        stream.resize(45 * 8, false);
        let bytes: Vec<u8> = stream
            .chunks(8)
            .map(|byte| {
                byte.iter().fold(0, |acc, &bit| (acc << 1) | u8::from(bit))
            })
            .collect();

        let mut texture = Texture::new(64, 64);
        texture.set(0, 32, color(V1_MAGIC));
        for (index, rgb) in bytes.chunks(3).enumerate() {
            let position = index as u32 + 1;
            texture.set(
                position % 4,
                32 + position / 4,
                [rgb[0], rgb[1], rgb[2], 255],
            );
        }
        texture
    }

    #[test]
    fn skins_without_the_magic_pixel_have_no_features() {
        assert_eq!(parse(&Texture::new(64, 64)), None);
    }

    #[test]
    fn v0_defaults_are_plain_ears_above_and_a_downward_tail() {
        let features = parse(&v0_skin(&[])).unwrap();
        assert_eq!(features.ear_mode, EarMode::Above);
        assert_eq!(features.ear_anchor, EarAnchor::Center);
        assert_eq!(features.tail_mode, TailMode::Down);
        assert_eq!(features.tail_segments, 0);
        assert_eq!(features.snout_width, 0);
        assert_eq!(features.wing_mode, WingMode::None);
        assert_eq!(features.leg_mode, LegMode::Plantigrade);
        assert!(features.animate_tail);
        assert!(!features.cape_enabled);
        assert!(!features.emissive);
    }

    #[test]
    fn v0_reads_each_config_pixel() {
        let features = parse(&v0_skin(&[
            (1, color(PINK)),
            (2, color(RED)),
            (3, color(CYAN)),
            (4, color(GRAY)),
            (5, [255, 128, 0, 0]),
            (6, [3, 2, 9, 255]),
            (7, [64, 7, 16, 255]),
            (8, color(PURPLE2)),
            (10, color(ORANGE)),
            (11, color(PINK)),
            (12, [0, 0, 2, 255]),
        ]))
        .unwrap();

        assert_eq!(features.ear_mode, EarMode::Cross);
        assert_eq!(features.ear_anchor, EarAnchor::Back);
        assert_eq!(features.protrusions, Protrusions::ClawsAndHorn);
        assert_eq!(features.tail_mode, TailMode::StarOverlap);
        assert_eq!(features.tail_bends, [90.0, 90.0, 0.703125, 0.0]);
        assert_eq!(features.tail_segments, 3);
        assert_eq!(
            (
                features.snout_width,
                features.snout_height,
                features.snout_depth,
                features.snout_offset
            ),
            (3, 2, 8, 6)
        );
        assert_eq!(features.chest_size, 0.5);
        assert!(features.cape_enabled);
        assert_eq!(features.wing_mode, WingMode::Flat);
        assert!(features.emissive);
        assert_eq!(features.leg_mode, LegMode::DigitigradeFull);
        assert!(!features.animate_tail);
        assert!(features.swap_jacket_back_and_tail);
    }

    #[test]
    fn v1_reads_the_packed_bit_stream() {
        let features = parse(&v1_skin(&[
            (3, 8),
            (5, 6),
            (1, 2),
            (7, 3),
            (1, 2),
            (1, 1),
            (63, 6),
            (0, 1),
            (21, 6),
            (2, 3),
            (1, 2),
            (2, 3),
            (7, 3),
            (31, 5),
            (1, 3),
            (1, 1),
            (1, 1),
            (1, 1),
            (1, 3),
            (2, 3),
            (2, 3),
            (0, 1),
            (1, 1),
            (1, 2),
        ]))
        .unwrap();

        assert_eq!(features.ear_mode, EarMode::Sides);
        assert_eq!(features.ear_anchor, EarAnchor::Front);
        assert_eq!(features.protrusions, Protrusions::DoubleHalo);
        assert_eq!(features.tail_mode, TailMode::StarOverlap);
        assert_eq!(features.tail_segments, 2);
        assert_eq!(features.tail_bends[0], -90.0);
        assert!((features.tail_bends[1] - 30.0).abs() < 1e-4);
        assert_eq!(features.tail_bends[2..], [0.0, 0.0]);
        assert_eq!(
            (
                features.snout_width,
                features.snout_height,
                features.snout_depth,
                features.snout_offset
            ),
            (2, 2, 3, 6)
        );
        assert_eq!(features.chest_size, 1.0);
        assert_eq!(features.wing_mode, WingMode::SymmetricDual);
        assert_eq!(features.wing_animation_mode, WingAnimationMode::NoFlight);
        assert!(features.cape_enabled);
        assert!(features.emissive);
        assert_eq!(features.leg_mode, LegMode::DigitigradeFull);
        assert!(!features.animate_tail);
        assert!(features.swap_jacket_back_and_tail);
    }

    #[test]
    fn v1_ear_values_past_the_known_modes_mean_no_ears() {
        let features = parse(&v1_skin(&[(0, 8), (63, 6)])).unwrap();
        assert_eq!(features.ear_mode, EarMode::None);
    }
}
