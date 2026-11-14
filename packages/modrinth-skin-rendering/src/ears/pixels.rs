use std::collections::{HashMap, HashSet};

use super::features::{BitReader, LegMode, rgb};
use crate::texture::{SKIN_SIZE, Texture};

type Region = (u32, u32, u32, u32);

const ALFALFA_MAGIC: u32 = 0xea1f_a1fa;
const ALFALFA_KEYS: [&str; 4] = ["END", "wing", "erase", "cape"];

const ALFALFA_REGIONS: [Region; 10] = [
    (8, 0, 24, 8),
    (0, 8, 8, 16),
    (16, 8, 32, 16),
    (4, 16, 12, 20),
    (20, 16, 36, 20),
    (44, 16, 52, 20),
    (0, 20, 56, 32),
    (20, 48, 28, 52),
    (36, 48, 44, 52),
    (16, 52, 48, 64),
];

const FORCED_OPAQUE: &[Region] = &[
    (8, 0, 24, 8),
    (0, 8, 32, 16),
    (4, 16, 12, 20),
    (20, 16, 36, 20),
    (44, 16, 52, 20),
    (0, 20, 56, 32),
    (20, 48, 28, 52),
    (36, 48, 44, 52),
    (16, 52, 48, 64),
];

const FORCED_OPAQUE_WITHOUT_LEG_BOTTOM: &[Region] = &[
    (8, 0, 24, 8),
    (0, 8, 32, 16),
    (4, 16, 8, 20),
    (20, 16, 36, 20),
    (44, 16, 52, 20),
    (0, 20, 16, 38),
    (16, 20, 56, 32),
    (20, 48, 24, 52),
    (36, 48, 44, 52),
    (16, 52, 32, 58),
    (32, 52, 48, 64),
];

const FORCED_OPAQUE_WITHOUT_LEGS: &[Region] = &[
    (8, 0, 24, 8),
    (0, 8, 32, 16),
    (4, 16, 8, 20),
    (20, 16, 36, 20),
    (44, 16, 52, 20),
    (16, 20, 56, 32),
    (20, 48, 24, 52),
    (36, 48, 44, 52),
    (32, 52, 48, 64),
];

const LEG_BOTTOM_HALF_REGIONS: [(Region, bool); 8] = [
    ((24, 48, 28, 52), true),
    ((16, 58, 32, 64), true),
    ((8, 16, 12, 20), true),
    ((0, 26, 16, 32), true),
    ((8, 32, 12, 36), false),
    ((0, 42, 16, 48), false),
    ((8, 48, 12, 52), false),
    ((0, 58, 16, 64), false),
];

const LEG_REGIONS: [(Region, bool); 8] = [
    ((20, 48, 28, 52), true),
    ((16, 52, 32, 64), true),
    ((4, 16, 12, 20), true),
    ((0, 20, 16, 32), true),
    ((4, 32, 12, 36), false),
    ((0, 36, 16, 48), false),
    ((4, 48, 12, 52), false),
    ((0, 52, 16, 64), false),
];

fn columns((x1, y1, x2, y2): Region) -> impl Iterator<Item = (u32, u32)> {
    (x1..x2).flat_map(move |x| (y1..y2).map(move |y| (x, y)))
}

/// Alfalfa is a key-value store packed into the alpha channel of opaque skin regions.
pub(super) fn parse_alfalfa(texture: &Texture) -> HashMap<String, Vec<u8>> {
    let mut bits = Vec::new();
    for (x, y) in ALFALFA_REGIONS.into_iter().flat_map(columns) {
        let alpha = texture.get(x, y)[3];
        if alpha == 0 {
            continue;
        }
        let value = 0x7f - (alpha & 0x7f);
        bits.extend((0..7).map(|bit| (value >> bit) & 1 == 1));
    }

    let mut bytes: Vec<u8> = bits
        .chunks(8)
        .map(|byte| {
            byte.iter()
                .enumerate()
                .fold(0, |acc, (bit, &set)| acc | (u8::from(set) << bit))
        })
        .collect();
    while bytes.last() == Some(&0) {
        bytes.pop();
    }
    bytes.reverse();

    read_alfalfa_entries(&bytes).unwrap_or_default()
}

fn read_alfalfa_entries(bytes: &[u8]) -> Option<HashMap<String, Vec<u8>>> {
    let mut remaining = bytes.iter().copied();
    let mut entries = HashMap::new();
    if bytes.is_empty() {
        return Some(entries);
    }

    let magic = u32::from_be_bytes([
        remaining.next()?,
        remaining.next()?,
        remaining.next()?,
        remaining.next()?,
    ]);
    if magic != ALFALFA_MAGIC || remaining.next()? != 1 {
        return Some(entries);
    }

    while let Some(first) = remaining.next() {
        let key = if first < 64 {
            ALFALFA_KEYS
                .get(first as usize)
                .map_or_else(|| format!("!unk{first}"), |key| (*key).to_owned())
        } else {
            let mut chars = vec![first];
            while chars[chars.len() - 1] & 0x80 == 0 {
                chars.push(remaining.next()?);
            }
            let last = chars.len() - 1;
            chars[last] &= 0x7f;
            chars.into_iter().map(char::from).collect()
        };
        if key == "END" {
            break;
        }

        let mut value = Vec::new();
        loop {
            let length = remaining.next()?;
            for _ in 0..length {
                value.push(remaining.next()?);
            }
            if length != 255 {
                break;
            }
        }
        entries.insert(key, value);
    }

    Some(entries)
}

pub(super) fn apply_erase_data(texture: &mut Texture, erase: &[u8]) {
    let mut reader = BitReader::new(erase);
    while reader.remaining_bits() >= 22 {
        let (Some(x), Some(y), Some(width), Some(height)) = (
            reader.read(6),
            reader.read(6),
            reader.read(5),
            reader.read(5),
        ) else {
            return;
        };
        let region = (
            x,
            y,
            (x + width + 1).min(SKIN_SIZE),
            (y + height + 1).min(SKIN_SIZE),
        );
        for (x, y) in columns(region) {
            texture.set(x, y, [0; 4]);
        }
    }
}

pub(super) fn force_opaque_regions(texture: &mut Texture, leg_mode: LegMode) {
    let regions = match leg_mode {
        LegMode::DigitigradeFull => FORCED_OPAQUE_WITHOUT_LEGS,
        LegMode::DigitigradePartial => FORCED_OPAQUE_WITHOUT_LEG_BOTTOM,
        LegMode::Plantigrade => FORCED_OPAQUE,
    };
    for (x, y) in regions.iter().copied().flat_map(columns) {
        let mut pixel = texture.get(x, y);
        pixel[3] = 255;
        texture.set(x, y, pixel);
    }
}

/// Moves the leg pixels that digitigrade legs redraw out of `texture` into a new one.
pub(super) fn displace_digitigrade_legs(
    texture: &mut Texture,
    leg_mode: LegMode,
) -> Option<Texture> {
    let regions = match leg_mode {
        LegMode::Plantigrade => return None,
        LegMode::DigitigradePartial => LEG_BOTTOM_HALF_REGIONS,
        LegMode::DigitigradeFull => LEG_REGIONS,
    };

    let mut displaced = Texture::new(SKIN_SIZE, SKIN_SIZE);
    for (region, force_opaque) in regions {
        for (x, y) in columns(region) {
            let mut pixel = texture.get(x, y);
            if force_opaque {
                pixel[3] = 255;
            }
            displaced.set(x, y, pixel);
            texture.set(x, y, [0; 4]);
        }
    }
    Some(displaced)
}

pub(super) fn swap_jacket_back_and_tail(texture: &mut Texture) {
    let mut tail = [[0; 4]; 8 * 12];
    for x in 0..8 {
        for y in 0..12 {
            tail[(y * 8 + x) as usize] = texture.get(56 + x, 16 + y);
            let jacket = texture.get(32 + x, 36 + (11 - y));
            texture.set(56 + x, 16 + y, jacket);
        }
    }
    for x in 0..8 {
        for y in 0..12 {
            texture.set(32 + x, 36 + y, tail[(y * 8 + x) as usize]);
        }
    }
}

pub(super) fn emissive_palette(texture: &Texture) -> HashSet<u32> {
    columns((52, 32, 56, 36))
        .map(|(x, y)| texture.get(x, y))
        .filter(|pixel| pixel[3] > 0)
        .map(rgb)
        .collect()
}

/// Returns `(base, emissive)`.
pub(super) fn split_emissive(
    texture: &Texture,
    palette: &HashSet<u32>,
) -> (Texture, Texture) {
    let mut base = texture.clone();
    let mut emissive = texture.clone();
    for (base, emissive) in
        base.pixels_mut().iter_mut().zip(emissive.pixels_mut())
    {
        if palette.contains(&rgb(*base)) {
            base[3] = 0;
        } else {
            emissive[3] = 0;
        }
    }
    (base, emissive)
}

/// A PNG that fails to decode gives a blank canvas.
pub(super) fn embedded_texture(
    png: &[u8],
    width: u32,
    height: u32,
    offset_y: u32,
) -> Texture {
    let mut canvas = Texture::new(width, height);
    if let Some(image) = decode_embedded(png) {
        for y in 0..image.height().min(height.saturating_sub(offset_y)) {
            for x in 0..image.width().min(width) {
                canvas.set(x, y + offset_y, image.get(x, y));
            }
        }
    }
    canvas
}

#[cfg(feature = "png")]
fn decode_embedded(png: &[u8]) -> Option<Texture> {
    Texture::from_png_with_limit(png, SKIN_SIZE).ok()
}

#[cfg(not(feature = "png"))]
fn decode_embedded(_png: &[u8]) -> Option<Texture> {
    None
}

pub(super) fn png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 24 || !data.starts_with(&[0x89, 0x50, 0x4e, 0x47]) {
        return None;
    }
    Some((
        u32::from_be_bytes(data[16..20].try_into().ok()?),
        u32::from_be_bytes(data[20..24].try_into().ok()?),
    ))
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    pub(in crate::ears) fn write_alfalfa(texture: &mut Texture, bytes: &[u8]) {
        let mut little_endian = bytes.to_vec();
        little_endian.reverse();
        let bits: Vec<bool> = little_endian
            .iter()
            .flat_map(|byte| (0..8).map(move |bit| (byte >> bit) & 1 == 1))
            .collect();
        let mut values = bits.chunks(7).map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .fold(0_u8, |acc, (bit, &set)| acc | (u8::from(set) << bit))
        });

        for (x, y) in ALFALFA_REGIONS.into_iter().flat_map(columns) {
            let value = values.next().unwrap_or(0);
            let mut pixel = texture.get(x, y);
            pixel[3] = 0x80 | (0x7f - value);
            texture.set(x, y, pixel);
        }
        assert!(values.next().is_none(), "alfalfa payload does not fit");
    }

    pub(in crate::ears) fn alfalfa_payload(entries: &[(u8, &[u8])]) -> Vec<u8> {
        let mut bytes = ALFALFA_MAGIC.to_be_bytes().to_vec();
        bytes.push(1);
        for (key, value) in entries {
            bytes.push(*key);
            let mut chunks = value.chunks(255).peekable();
            while let Some(chunk) = chunks.next() {
                bytes.push(chunk.len() as u8);
                bytes.extend_from_slice(chunk);
                if chunk.len() == 255 && chunks.peek().is_none() {
                    bytes.push(0);
                }
            }
            if value.is_empty() {
                bytes.push(0);
            }
        }
        bytes.push(0);
        bytes
    }

    fn opaque_skin() -> Texture {
        Texture::from_pixels(64, 64, vec![[9, 9, 9, 255]; 64 * 64]).unwrap()
    }

    #[test]
    fn a_fully_opaque_skin_has_no_alfalfa_data() {
        assert!(parse_alfalfa(&opaque_skin()).is_empty());
    }

    #[test]
    fn alfalfa_entries_round_trip_through_alpha_values() {
        let long_value: Vec<u8> = (0..300).map(|i| (i % 251) as u8).collect();
        let mut texture = opaque_skin();
        write_alfalfa(
            &mut texture,
            &alfalfa_payload(&[(2, &[1, 2, 3]), (1, &long_value)]),
        );

        let entries = parse_alfalfa(&texture);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries["erase"], vec![1, 2, 3]);
        assert_eq!(entries["wing"], long_value);
    }

    #[test]
    fn alfalfa_with_the_wrong_magic_is_ignored() {
        let mut payload = alfalfa_payload(&[(2, &[1])]);
        payload[1] ^= 0xff;
        let mut texture = opaque_skin();
        write_alfalfa(&mut texture, &payload);
        assert!(parse_alfalfa(&texture).is_empty());
    }

    #[test]
    fn truncated_alfalfa_yields_nothing() {
        let mut payload = alfalfa_payload(&[(2, &[1, 2, 3, 4])]);
        payload.truncate(payload.len() - 3);
        let mut texture = opaque_skin();
        write_alfalfa(&mut texture, &payload);
        assert!(parse_alfalfa(&texture).is_empty());
    }

    #[test]
    fn erase_rectangles_clear_pixels_and_clip_to_the_skin() {
        let mut texture = opaque_skin();
        let rectangles: [(u32, u32, u32, u32); 2] =
            [(2, 3, 1, 0), (62, 62, 7, 7)];
        let mut bits = Vec::new();
        for (x, y, width, height) in rectangles {
            for (value, size) in [(x, 6), (y, 6), (width, 5), (height, 5)] {
                bits.extend((0..size).rev().map(|bit| (value >> bit) & 1 == 1));
            }
        }
        let erase: Vec<u8> = bits
            .chunks(8)
            .map(|byte| {
                byte.iter().enumerate().fold(0, |acc, (index, &set)| {
                    acc | (u8::from(set) << (7 - index))
                })
            })
            .collect();

        apply_erase_data(&mut texture, &erase);
        assert_eq!(texture.get(2, 3), [0; 4]);
        assert_eq!(texture.get(3, 3), [0; 4]);
        assert_eq!(texture.get(4, 3)[3], 255);
        assert_eq!(texture.get(2, 4)[3], 255);
        assert_eq!(texture.get(63, 63), [0; 4]);
        assert_eq!(texture.get(61, 63)[3], 255);
    }

    #[test]
    fn full_digitigrade_legs_move_every_leg_pixel() {
        let mut texture =
            Texture::from_pixels(64, 64, vec![[7, 7, 7, 100]; 64 * 64])
                .unwrap();
        let displaced =
            displace_digitigrade_legs(&mut texture, LegMode::DigitigradeFull)
                .unwrap();

        assert_eq!(displaced.get(0, 20), [7, 7, 7, 255]);
        assert_eq!(displaced.get(0, 36), [7, 7, 7, 100]);
        assert_eq!(displaced.get(40, 20), [0; 4]);
        assert_eq!(texture.get(0, 20), [0; 4]);
        assert_eq!(texture.get(40, 20), [7, 7, 7, 100]);
        assert!(
            displace_digitigrade_legs(&mut texture, LegMode::Plantigrade)
                .is_none()
        );
    }

    #[test]
    fn swapping_moves_the_tail_and_flips_the_jacket_back() {
        let mut texture = Texture::new(64, 64);
        texture.set(56, 16, [1, 0, 0, 255]);
        texture.set(32, 47, [2, 0, 0, 255]);
        swap_jacket_back_and_tail(&mut texture);
        assert_eq!(texture.get(32, 36), [1, 0, 0, 255]);
        assert_eq!(texture.get(56, 16), [2, 0, 0, 255]);
    }

    #[test]
    fn emissive_split_keeps_each_pixel_in_exactly_one_texture() {
        let mut texture = opaque_skin();
        for (x, y) in columns((52, 32, 56, 36)) {
            texture.set(x, y, [0; 4]);
        }
        texture.set(52, 32, [200, 0, 0, 255]);
        texture.set(53, 32, [0, 200, 0, 0]);
        texture.set(10, 10, [200, 0, 0, 255]);

        let palette = emissive_palette(&texture);
        assert_eq!(palette, HashSet::from([0xc80000]));

        let (base, emissive) = split_emissive(&texture, &palette);
        assert_eq!(base.get(10, 10)[3], 0);
        assert_eq!(emissive.get(10, 10), [200, 0, 0, 255]);
        assert_eq!(base.get(11, 10), [9, 9, 9, 255]);
        assert_eq!(emissive.get(11, 10)[3], 0);
    }

    #[cfg(feature = "png")]
    #[test]
    fn legacy_wings_are_drawn_two_pixels_down() {
        let mut wing = Texture::new(12, 12);
        wing.set(0, 0, [5, 6, 7, 255]);
        let png = wing.to_png().unwrap();
        assert_eq!(png_dimensions(&png), Some((12, 12)));

        let canvas = embedded_texture(&png, 20, 16, 2);
        assert_eq!(canvas.get(0, 2), [5, 6, 7, 255]);
        assert_eq!(canvas.get(0, 0), [0; 4]);
        assert_eq!(embedded_texture(b"not a png", 20, 16, 0).get(0, 0), [0; 4]);
    }
}
