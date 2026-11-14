//! [Ears mod](https://ears.unascribed.com) support.
//!
//! Adapted from Ears, Ears2 and the Ears Skin Manipulator
//! (<https://git.sleeping.town/exa.mods/Ears>, <https://git.sleeping.town/exa.mods/Ears2>),
//! copyright (c) 2020-2026 Exa Skye and contributors
//!
//! See `THIRD_PARTY.md`.

mod builder;
mod features;
mod parts;
mod pixels;

pub use features::{
    EarAnchor, EarMode, Features, LegMode, Protrusions, TailMode,
    WingAnimationMode, WingMode,
};

use crate::model::BodyPart;
use crate::texture::{SkinTexture, Texture};

/// Indices that split an [`EarsQuad`] into two triangles.
pub const QUAD_INDICES: [usize; 6] = [0, 1, 2, 0, 2, 3];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EarsTextureSlot {
    Skin,
    DisplacedSkin,
    Wing,
    Cape,
}

impl EarsTextureSlot {
    /// In texels.
    pub const fn size(self) -> (u32, u32) {
        match self {
            Self::Skin | Self::DisplacedSkin => (64, 64),
            Self::Wing | Self::Cape => (20, 16),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TexturePair {
    pub base: Texture,
    pub emissive: Option<Texture>,
}

/// Positions are in skin pixels, relative to the corner of the body part's base cuboid with the
/// largest X and the smallest Y and Z, in the body part's own space. Vertices wind
/// counter-clockwise seen from the front.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EarsQuad {
    pub part: BodyPart,
    pub texture: EarsTextureSlot,
    pub positions: [[f32; 3]; 4],
    pub uvs: [[f32; 2]; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ears {
    pub features: Features,
    /// Replaces the regular skin texture on the body.
    pub skin: TexturePair,
    pub displaced_skin: Option<TexturePair>,
    pub wing: Option<TexturePair>,
    pub cape: Option<TexturePair>,
    pub quads: Vec<EarsQuad>,
}

impl Ears {
    pub fn detect(skin: &SkinTexture) -> Option<Features> {
        if skin.is_legacy() {
            return None;
        }
        features::parse(skin.raw())
    }

    /// `None` if the skin has no Ears data. `slim` moves the left arm claw.
    pub fn build(skin: &SkinTexture, slim: bool) -> Option<Self> {
        let mut features = Self::detect(skin)?;
        let alfalfa = pixels::parse_alfalfa(skin.raw());

        let mut processed = skin.raw().clone();
        if let Some(erase) = alfalfa.get("erase") {
            pixels::apply_erase_data(&mut processed, erase);
        }
        let displaced = pixels::displace_digitigrade_legs(
            &mut processed,
            features.leg_mode,
        );
        if features.swap_jacket_back_and_tail {
            pixels::swap_jacket_back_and_tail(&mut processed);
        }
        pixels::force_opaque_regions(&mut processed, features.leg_mode);

        let palette = if features.emissive {
            pixels::emissive_palette(&processed)
        } else {
            std::collections::HashSet::new()
        };
        features.emissive = !palette.is_empty();
        let split = |texture: Texture| {
            if features.emissive {
                let (base, emissive) =
                    pixels::split_emissive(&texture, &palette);
                TexturePair {
                    base,
                    emissive: Some(emissive),
                }
            } else {
                TexturePair {
                    base: texture,
                    emissive: None,
                }
            }
        };

        let skin_textures = split(processed);
        let displaced_skin = displaced.map(split);

        let wing = alfalfa
            .get("wing")
            .filter(|_| features.wing_mode != WingMode::None)
            .and_then(|data| match pixels::png_dimensions(data)? {
                (20, 16) => Some(pixels::embedded_texture(data, 20, 16, 0)),
                (12, 12) => Some(pixels::embedded_texture(data, 20, 16, 2)),
                _ => None,
            })
            .map(split);
        if wing.is_none() {
            features.wing_mode = WingMode::None;
        }

        let cape = alfalfa
            .get("cape")
            .filter(|data| {
                features.cape_enabled
                    && pixels::png_dimensions(data) == Some((20, 16))
            })
            .map(|data| TexturePair {
                base: pixels::embedded_texture(data, 20, 16, 0),
                emissive: None,
            });
        features.cape_enabled = cape.is_some();

        let mut builder = builder::QuadBuilder::new([
            true,
            displaced_skin.is_some(),
            wing.is_some(),
            cape.is_some(),
        ]);
        parts::draw_all(&mut builder, &features, slim);

        Some(Self {
            features,
            skin: skin_textures,
            displaced_skin,
            wing,
            cape,
            quads: builder.quads,
        })
    }

    pub const fn texture(&self, slot: EarsTextureSlot) -> Option<&TexturePair> {
        match slot {
            EarsTextureSlot::Skin => Some(&self.skin),
            EarsTextureSlot::DisplacedSkin => self.displaced_skin.as_ref(),
            EarsTextureSlot::Wing => self.wing.as_ref(),
            EarsTextureSlot::Cape => self.cape.as_ref(),
        }
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use crate::texture::Texture;

    /// Blank skin with the default Ears configuration (ears above, tail down).
    pub(crate) fn default_ears_skin() -> Texture {
        super::features::tests::v0_skin(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::features::tests::{color, v0_skin, v1_skin};
    use super::*;

    const PINK: u32 = 0xd823b7;
    const ORANGE: u32 = 0xd87823;

    fn skin(texture: Texture) -> SkinTexture {
        SkinTexture::new(texture).unwrap()
    }

    #[test]
    fn plain_and_legacy_skins_have_no_ears() {
        assert!(Ears::build(&skin(Texture::new(64, 64)), false).is_none());
        assert!(Ears::build(&skin(Texture::new(64, 32)), false).is_none());
    }

    #[test]
    fn default_features_draw_ears_on_the_head_and_a_tail_on_the_torso() {
        let ears = Ears::build(&skin(v0_skin(&[])), false).unwrap();
        let on = |part| ears.quads.iter().filter(|q| q.part == part).count();
        assert_eq!(on(BodyPart::Head), 2);
        assert_eq!(on(BodyPart::Torso), 2);
        assert_eq!(ears.quads.len(), 4);
        assert!(
            ears.quads
                .iter()
                .all(|q| q.texture == EarsTextureSlot::Skin)
        );
        assert!(ears.skin.emissive.is_none());
        assert!(ears.displaced_skin.is_none());
    }

    #[test]
    fn ears_above_sit_on_top_of_the_head() {
        let ears = Ears::build(&skin(v0_skin(&[])), false).unwrap();
        let front = &ears.quads[0];
        let ys: Vec<f32> = front.positions.iter().map(|p| p[1]).collect();
        let xs: Vec<f32> = front.positions.iter().map(|p| p[0]).collect();
        assert_eq!(ys, [8.0, 8.0, 16.0, 16.0]);
        assert_eq!(xs, [4.0, -12.0, -12.0, 4.0]);
        assert!(front.positions.iter().all(|p| p[2] == 4.0));
    }

    #[test]
    fn the_body_texture_is_made_opaque_only_where_ears_requires_it() {
        let mut texture = v0_skin(&[]);
        texture.set(8, 8, [1, 2, 3, 0]);
        texture.set(40, 8, [4, 5, 6, 0]);
        let ears = Ears::build(&skin(texture), false).unwrap();
        assert_eq!(ears.skin.base.get(8, 8), [1, 2, 3, 255]);
        assert_eq!(ears.skin.base.get(40, 8), [4, 5, 6, 0]);
    }

    #[test]
    fn digitigrade_legs_use_the_displaced_texture() {
        let ears =
            Ears::build(&skin(v0_skin(&[(11, color(PINK))])), false).unwrap();
        assert_eq!(ears.features.leg_mode, LegMode::DigitigradeFull);
        assert!(ears.displaced_skin.is_some());
        assert!(
            ears.quads
                .iter()
                .any(|q| q.texture == EarsTextureSlot::DisplacedSkin
                    && q.part == BodyPart::LeftLeg)
        );
        assert_eq!(ears.skin.base.get(0, 20)[3], 0);
    }

    #[test]
    fn emissive_is_dropped_when_the_palette_is_empty() {
        let ears =
            Ears::build(&skin(v0_skin(&[(10, color(ORANGE))])), false).unwrap();
        assert!(!ears.features.emissive);
        assert!(ears.skin.emissive.is_none());
    }

    #[test]
    fn emissive_pixels_move_to_their_own_texture() {
        let mut texture = v0_skin(&[(10, color(ORANGE))]);
        texture.set(52, 32, [250, 250, 0, 255]);
        texture.set(9, 9, [250, 250, 0, 255]);
        let ears = Ears::build(&skin(texture), false).unwrap();
        assert!(ears.features.emissive);
        assert_eq!(ears.skin.base.get(9, 9)[3], 0);
        assert_eq!(
            ears.skin.emissive.as_ref().unwrap().get(9, 9),
            [250, 250, 0, 255]
        );
    }

    #[test]
    fn wings_without_an_embedded_texture_are_disabled() {
        let ears =
            Ears::build(&skin(v0_skin(&[(8, color(PINK))])), false).unwrap();
        assert_eq!(ears.features.wing_mode, WingMode::None);
        assert!(ears.wing.is_none());
    }

    #[cfg(feature = "png")]
    #[test]
    fn embedded_wings_are_drawn_from_alfalfa_data() {
        use super::pixels::tests::{alfalfa_payload, write_alfalfa};
        const GREEN: u32 = 0x23d848;

        let mut wing = Texture::new(20, 16);
        wing.set(3, 4, [10, 20, 30, 255]);
        let png = wing.to_png().unwrap();

        let mut texture = v0_skin(&[(8, color(GREEN))]);
        for pixel in texture.pixels_mut() {
            pixel[3] = 255;
        }
        write_alfalfa(&mut texture, &alfalfa_payload(&[(1, &png)]));

        let ears = Ears::build(&skin(texture), false).unwrap();
        assert_eq!(ears.features.wing_mode, WingMode::SymmetricSingle);
        assert_eq!(
            ears.wing.as_ref().unwrap().base.get(3, 4),
            [10, 20, 30, 255]
        );
        let wing_quads: Vec<_> = ears
            .quads
            .iter()
            .filter(|q| q.texture == EarsTextureSlot::Wing)
            .collect();
        assert_eq!(wing_quads.len(), 2);
        assert!(wing_quads.iter().all(|q| q.part == BodyPart::Torso));
    }

    #[test]
    fn slim_arms_move_the_left_claw_inwards() {
        let claws = |slim| {
            let texture = v1_skin(&[(3, 8), (0, 6), (2, 2)]);
            let ears = Ears::build(&skin(texture), slim).unwrap();
            assert_eq!(ears.features.protrusions, Protrusions::Claws);
            ears.quads
                .into_iter()
                .find(|q| q.part == BodyPart::LeftArm)
                .unwrap()
        };
        assert_ne!(claws(true).positions, claws(false).positions);
    }
}
