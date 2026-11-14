use super::EarsTextureSlot;
use super::builder::{Flip, QuadBuilder, Rotation, face};
use super::features::{
    EarAnchor, EarMode, Features, LegMode, Protrusions, TailMode,
    WingAnimationMode, WingMode,
};
use crate::model::BodyPart;

pub(super) fn draw_all(b: &mut QuadBuilder, features: &Features, slim: bool) {
    ears(b, features);
    tail(b, features);
    protrusions(b, features, slim);
    snout(b, features);
    chest(b, features);
    wings(b, features);
    cape(b, features);
    digitigrade_legs(b, features);
}

fn apply_ear_anchor(b: &mut QuadBuilder, anchor: EarAnchor) {
    match anchor {
        EarAnchor::Center => b.translate(0.0, 0.0, 4.0),
        EarAnchor::Back => b.translate(0.0, 0.0, 8.0),
        EarAnchor::Front => {}
    }
}

fn ears(b: &mut QuadBuilder, features: &Features) {
    let cw = Rotation::Cw;
    match features.ear_mode {
        EarMode::None => {}
        EarMode::Above | EarMode::Around => {
            b.push();
            b.anchor_to(BodyPart::Head);
            apply_ear_anchor(b, features.ear_anchor);
            b.push();
            b.translate(-4.0, -16.0, 0.0);
            b.front(face(24.0, 0.0, 16.0, 8.0));
            b.back(face(56.0, 28.0, 16.0, 8.0).rotated(cw));
            b.pop();
            if features.ear_mode == EarMode::Around {
                b.translate(-4.0, -8.0, 0.0);
                b.front(face(36.0, 16.0, 4.0, 8.0).rotated(cw));
                b.back(face(12.0, 16.0, 4.0, 8.0).rotated(cw));
                b.translate(12.0, 0.0, 0.0);
                b.front(face(36.0, 32.0, 4.0, 8.0).rotated(cw));
                b.back(face(12.0, 32.0, 4.0, 8.0).rotated(cw));
            }
            b.pop();
        }
        EarMode::Sides => {
            b.push();
            b.anchor_to(BodyPart::Head);
            apply_ear_anchor(b, features.ear_anchor);
            b.translate(-8.0, -8.0, 0.0);
            b.front(face(24.0, 0.0, 8.0, 8.0));
            b.back(face(56.0, 28.0, 8.0, 8.0).rotated(cw));
            b.translate(16.0, 0.0, 0.0);
            b.front(face(32.0, 0.0, 8.0, 8.0));
            b.back(face(56.0, 36.0, 8.0, 8.0).rotated(cw));
            b.pop();
        }
        EarMode::Behind => {
            b.push();
            b.anchor_to(BodyPart::Head);
            b.rotate(90.0, 0.0, 1.0, 0.0);
            b.translate(-16.0, -8.0, 0.0);
            b.front(face(24.0, 0.0, 8.0, 8.0));
            b.back(face(56.0, 28.0, 8.0, 8.0).rotated(cw));
            b.rotate(180.0, 0.0, 1.0, 0.0);
            b.translate(-8.0, 0.0, -8.0);
            b.front(face(32.0, 0.0, 8.0, 8.0));
            b.back(face(56.0, 36.0, 8.0, 8.0).rotated(cw));
            b.pop();
        }
        EarMode::Floppy => {
            b.push();
            b.anchor_to(BodyPart::Head);
            b.rotate(90.0, 0.0, 1.0, 0.0);
            b.translate(-8.0, -7.0, 0.0);
            b.rotate(-30.0, 1.0, 0.0, 0.0);
            b.front(face(24.0, 0.0, 8.0, 8.0));
            b.back(face(56.0, 28.0, 8.0, 8.0).rotated(cw));
            b.pop();

            b.push();
            b.anchor_to(BodyPart::Head);
            b.rotate(-90.0, 0.0, 1.0, 0.0);
            b.translate(0.0, -7.0, -8.0);
            b.rotate(-30.0, 1.0, 0.0, 0.0);
            b.front(face(32.0, 0.0, 8.0, 8.0));
            b.back(face(56.0, 36.0, 8.0, 8.0).rotated(cw));
            b.pop();
        }
        EarMode::Cross => {
            b.push();
            b.anchor_to(BodyPart::Head);
            apply_ear_anchor(b, features.ear_anchor);
            b.translate(4.0, -16.0, 0.0);
            b.push();
            b.rotate(45.0, 0.0, 1.0, 0.0);
            b.translate(-4.0, 0.0, 0.0);
            b.front(face(24.0, 0.0, 8.0, 8.0));
            b.back(face(56.0, 28.0, 8.0, 8.0).rotated(cw));
            b.pop();
            b.push();
            b.rotate(-45.0, 0.0, 1.0, 0.0);
            b.translate(-4.0, 0.0, 0.0);
            b.front(face(32.0, 0.0, 8.0, 8.0));
            b.back(face(56.0, 36.0, 8.0, 8.0).rotated(cw));
            b.pop();
            b.pop();
        }
        EarMode::Out => {
            b.push();
            b.anchor_to(BodyPart::Head);
            b.rotate(90.0, 0.0, 1.0, 0.0);
            match features.ear_anchor {
                EarAnchor::Back => b.translate(-16.0, -8.0, 0.0),
                EarAnchor::Center => b.translate(-8.0, -16.0, 0.0),
                EarAnchor::Front => b.translate(0.0, -8.0, 0.0),
            }
            b.front(face(24.0, 0.0, 8.0, 8.0));
            b.back(face(56.0, 28.0, 8.0, 8.0).rotated(cw));
            b.rotate(180.0, 0.0, 1.0, 0.0);
            b.translate(-8.0, 0.0, -8.0);
            b.front(face(32.0, 0.0, 8.0, 8.0));
            b.back(face(56.0, 36.0, 8.0, 8.0).rotated(cw));
            b.pop();
        }
        EarMode::Tall => {
            let angle = -6.0;
            b.push();
            b.anchor_to(BodyPart::Head);
            b.translate(0.0, -8.0, 0.0);
            apply_ear_anchor(b, features.ear_anchor);
            b.rotate(angle / 3.0, 1.0, 0.0, 0.0);
            b.translate(0.0, -4.0, 0.0);
            b.front(face(24.0, 0.0, 8.0, 4.0).rotated(cw));
            b.back(face(56.0, 40.0, 8.0, 4.0));
            b.rotate(angle, 1.0, 0.0, 0.0);
            b.translate(0.0, -4.0, 0.0);
            b.front(face(28.0, 0.0, 8.0, 4.0).rotated(cw));
            b.back(face(56.0, 36.0, 8.0, 4.0));
            b.rotate(angle / 2.0, 1.0, 0.0, 0.0);
            b.translate(0.0, -4.0, 0.0);
            b.front(face(32.0, 0.0, 8.0, 4.0).rotated(cw));
            b.back(face(56.0, 32.0, 8.0, 4.0));
            b.rotate(angle, 1.0, 0.0, 0.0);
            b.translate(0.0, -4.0, 0.0);
            b.front(face(36.0, 0.0, 8.0, 4.0).rotated(cw));
            b.back(face(56.0, 28.0, 8.0, 4.0));
            b.pop();
        }
        EarMode::TallCross => {
            b.push();
            b.anchor_to(BodyPart::Head);
            apply_ear_anchor(b, features.ear_anchor);
            b.translate(4.0, -24.0, 0.0);
            for angle in [45.0, -45.0] {
                b.push();
                b.rotate(angle, 0.0, 1.0, 0.0);
                b.translate(-4.0, 0.0, 0.0);
                b.front(face(24.0, 0.0, 8.0, 16.0).rotated(cw));
                b.back(face(56.0, 28.0, 8.0, 16.0));
                b.pop();
            }
            b.pop();
        }
    }
}

fn tail(b: &mut QuadBuilder, features: &Features) {
    let mode = features.tail_mode;
    let angle = match mode {
        TailMode::None => return,
        TailMode::Down => 30.0,
        TailMode::Back
        | TailMode::Cross
        | TailMode::CrossOverlap
        | TailMode::Star
        | TailMode::StarOverlap => {
            if features.tail_bends[0] != 0.0 {
                90.0
            } else {
                80.0
            }
        }
        TailMode::Up => 130.0,
        TailMode::Vertical => 0.0,
    };

    let vertical = mode == TailMode::Vertical;
    let base_angle = features.tail_bends[0];
    b.push();
    b.anchor_to(BodyPart::Torso);
    b.translate(0.0, -2.0, 4.0);
    b.rotate(angle, 1.0, 0.0, 0.0);
    if vertical {
        b.translate(4.0, 0.0, 0.0);
        b.rotate(90.0, 0.0, 0.0, 1.0);
        if base_angle < 0.0 {
            b.translate(4.0, 0.0, 0.0);
            b.rotate(base_angle, 0.0, 1.0, 0.0);
            b.translate(-4.0, 0.0, 0.0);
        }
        b.translate(-4.0, 0.0, 0.0);
        if base_angle > 0.0 {
            b.rotate(base_angle, 0.0, 1.0, 0.0);
        }
        b.rotate(90.0, 1.0, 0.0, 0.0);
    }

    let segments = features.tail_segments.max(1);
    let segment_height = (12 / segments) as f32;
    let mut angles = features.tail_bends;
    if vertical {
        angles[0] = 0.0;
    }
    let overlapping =
        matches!(mode, TailMode::CrossOverlap | TailMode::StarOverlap);
    let horizontal = Flip::Horizontal;

    for (index, angle) in angles.into_iter().take(segments as usize).enumerate()
    {
        let overlap = if index == 0 || !overlapping { 0.0 } else { 4.0 };
        let v = 16.0 + index as f32 * segment_height;
        b.rotate(angle, 1.0, 0.0, 0.0);
        b.double_sided(face(56.0, v, 8.0, segment_height).flipped(horizontal));

        let side_angles: &[f32] = match mode {
            TailMode::Cross | TailMode::CrossOverlap => &[90.0],
            TailMode::Star | TailMode::StarOverlap => &[45.0, 90.0, 135.0],
            _ => &[],
        };
        for &side_angle in side_angles {
            b.push();
            b.translate(4.0, 0.0, 0.0);
            b.rotate(side_angle, 0.0, 1.0, 0.0);
            b.translate(-4.0, -overlap, 0.0);
            b.double_sided(
                face(56.0, v - overlap, 8.0, segment_height + overlap)
                    .flipped(horizontal),
            );
            b.pop();
        }
        b.translate(0.0, segment_height, 0.0);
    }
    b.pop();
}

fn protrusions(b: &mut QuadBuilder, features: &Features, slim: bool) {
    let horizontal = Flip::Horizontal;
    if features.protrusions.has_claws() {
        let leg_offset = match features.leg_mode {
            LegMode::DigitigradeFull => -1.5,
            LegMode::DigitigradePartial => -1.0,
            LegMode::Plantigrade => 0.0,
        };
        b.push();
        b.anchor_to(BodyPart::LeftLeg);
        b.translate(0.0, 0.0, -4.0 + leg_offset);
        b.rotate(90.0, 1.0, 0.0, 0.0);
        b.double_sided(face(16.0, 48.0, 4.0, 4.0).flipped(horizontal));
        b.pop();

        b.push();
        b.anchor_to(BodyPart::RightLeg);
        b.translate(0.0, 0.0, -4.0 + leg_offset);
        b.rotate(90.0, 1.0, 0.0, 0.0);
        b.double_sided(face(0.0, 16.0, 4.0, 4.0).flipped(horizontal));
        b.pop();

        b.push();
        b.anchor_to(BodyPart::LeftArm);
        b.rotate(90.0, 0.0, 1.0, 0.0);
        b.translate(-4.0, 0.0, if slim { 3.0 } else { 4.0 });
        b.double_sided(
            face(44.0, 48.0, 4.0, 4.0)
                .rotated(Rotation::UpsideDown)
                .flipped(horizontal),
        );
        b.pop();

        b.push();
        b.anchor_to(BodyPart::RightArm);
        b.rotate(90.0, 0.0, 1.0, 0.0);
        b.translate(-4.0, 0.0, 0.0);
        b.double_sided(
            face(52.0, 16.0, 4.0, 4.0).rotated(Rotation::UpsideDown),
        );
        b.pop();
    }

    let horn_or_halo = face(56.0, 0.0, 8.0, 8.0);
    match features.protrusions {
        Protrusions::Horn | Protrusions::ClawsAndHorn => {
            b.push();
            b.anchor_to(BodyPart::Head);
            b.translate(0.0, -8.0, 0.0);
            b.rotate(25.0, 1.0, 0.0, 0.0);
            b.translate(0.0, -8.0, 0.0);
            b.double_sided(horn_or_halo);
            b.pop();
        }
        Protrusions::Halo | Protrusions::ClawsAndHalo => {
            b.push();
            b.anchor_to(BodyPart::Head);
            b.translate(0.0, -12.0, 0.0);
            b.rotate(90.0, 1.0, 0.0, 0.0);
            b.double_sided(horn_or_halo);
            b.pop();
        }
        Protrusions::DoubleHalo | Protrusions::ClawsAndDoubleHalo => {
            b.push();
            b.anchor_to(BodyPart::Head);
            b.translate(0.0, -10.0, 0.0);
            b.rotate(90.0, 1.0, 0.0, 0.0);
            for _ in 0..2 {
                b.translate(4.0, 4.0, 0.0);
                b.rotate(45.0, 0.0, 0.0, 1.0);
                b.translate(-4.0, -4.0, 2.0);
                b.double_sided(horn_or_halo);
            }
            b.pop();
        }
        Protrusions::None | Protrusions::Claws => {}
    }
}

fn digitigrade_leg(
    b: &mut QuadBuilder,
    u: f32,
    v: f32,
    grow: f32,
    bottom: bool,
    mend: bool,
) {
    let (width, height, depth) = (4.0, 12.0, 4.0);
    let double_grow = grow * 2.0;
    let skew = 1.0;
    let v_offset = if bottom { 10.0 } else { 4.0 };
    let v_side = v + v_offset;

    b.push();
    b.translate(width / 2.0, height / 2.0, depth / 2.0);
    b.scale(
        (width + double_grow) / width,
        (height + double_grow) / height,
        (depth + double_grow) / depth,
    );
    b.translate(-width / 2.0, -height / 2.0, -depth / 2.0);

    b.push();
    b.translate(0.0, if bottom { -6.0 } else { -12.0 }, 0.0);
    let sides: [(f32, [f32; 3], f32); 4] = [
        (u + 4.0, [0.0, 0.0, -skew], 6.0),
        (u + 8.0, [-skew, 0.0, 0.0], 6.0),
        (u + 12.0, [0.0, 0.0, skew], 5.0),
        (u, [skew, 0.0, 0.0], 6.0),
    ];
    for (index, (side_u, side_skew, mend_v)) in sides.into_iter().enumerate() {
        if index > 0 {
            b.rotate(-90.0, 0.0, 1.0, 0.0);
            b.translate(0.0, 0.0, -4.0);
        }
        if !mend && bottom {
            b.push();
            b.translate(0.0, 2.0, 0.0);
            b.front_skew(face(side_u, v_side + 2.0, 4.0, 4.0), side_skew);
            b.pop();
        } else {
            b.front_skew(face(side_u, v_side, 4.0, 6.0), side_skew);
        }
        if mend && !bottom {
            b.push();
            b.translate(0.0, 6.0, 0.0);
            b.front_skew(
                face(side_u, v_side + mend_v, 4.0, 2.0),
                side_skew.map(|component: f32| -component),
            );
            b.pop();
        }
    }
    b.pop();

    b.push();
    b.rotate(90.0, 1.0, 0.0, 0.0);
    let sole = face(u + 8.0, v, 4.0, 4.0).flipped(Flip::Vertical);
    b.push();
    if bottom {
        b.translate(0.0, -skew, 0.0);
        b.front(sole);
    } else {
        b.translate(0.0, skew, 12.0);
        b.back(sole);
    }
    b.pop();
    if mend && bottom {
        b.translate(0.0, 0.0, 6.0);
        b.front(face(u + 4.0, v_side, 4.0, 1.0));
        b.translate(0.0, 4.0, 0.0);
        b.back(face(u + 12.0, v_side, 4.0, 1.0));
    }
    b.pop();
    b.pop();
}

fn digitigrade_legs(b: &mut QuadBuilder, features: &Features) {
    if features.leg_mode == LegMode::Plantigrade {
        return;
    }

    let full = features.leg_mode == LegMode::DigitigradeFull;
    b.bind(EarsTextureSlot::DisplacedSkin);
    for (overlay, grow) in [(false, 0.0), (true, 0.25)] {
        let legs = [
            (BodyPart::LeftLeg, if overlay { 0.0 } else { 16.0 }, 48.0),
            (BodyPart::RightLeg, 0.0, if overlay { 32.0 } else { 16.0 }),
        ];
        for (part, u, v) in legs {
            b.push();
            b.anchor_to(part);
            if full {
                b.translate(0.0, 0.0, -0.5);
            }
            if overlay {
                b.translate(0.0, 0.5, 0.0);
            }
            if full {
                digitigrade_leg(b, u, v, grow, false, true);
            }
            digitigrade_leg(b, u, v, grow, true, !full);
            b.pop();
        }
    }
    b.bind(EarsTextureSlot::Skin);
}

fn snout(b: &mut QuadBuilder, features: &Features) {
    if features.snout_width == 0
        || features.snout_height == 0
        || features.snout_depth == 0
    {
        return;
    }
    let width = features.snout_width as f32;
    let height = features.snout_height as f32;
    let depth = features.snout_depth as f32;
    let offset = features.snout_offset as f32;
    let extra_rows = features.snout_depth - 1;

    b.push();
    b.anchor_to(BodyPart::Head);
    b.translate((8.0 - width) / 2.0, -(offset + height), -depth);
    b.double_sided(face(0.0, 2.0, width, height));

    b.push();
    b.rotate(-90.0, 1.0, 0.0, 0.0);
    b.translate(0.0, -1.0, 0.0);
    b.double_sided(face(0.0, 1.0, width, 1.0));
    for _ in 0..extra_rows {
        b.translate(0.0, -1.0, 0.0);
        b.double_sided(face(0.0, 0.0, width, 1.0));
    }
    b.pop();

    b.push();
    b.translate(0.0, height, 0.0);
    b.rotate(90.0, 1.0, 0.0, 0.0);
    b.double_sided(face(0.0, 2.0 + height, width, 1.0));
    for _ in 0..extra_rows {
        b.translate(0.0, 1.0, 0.0);
        b.double_sided(face(0.0, 3.0 + height, width, 1.0));
    }
    b.pop();

    b.push();
    b.rotate(90.0, 0.0, 1.0, 0.0);
    for side_offset in [0.0, width] {
        b.push();
        b.translate(-1.0, 0.0, side_offset);
        b.double_sided(face(7.0, 0.0, 1.0, height));
        for _ in 0..extra_rows {
            b.translate(-1.0, 0.0, 0.0);
            b.double_sided(face(7.0, 4.0, 1.0, height));
        }
        b.pop();
    }
    b.pop();
    b.pop();
}

fn chest(b: &mut QuadBuilder, features: &Features) {
    if features.chest_size <= 0.0 {
        return;
    }
    let horizontal = Flip::Horizontal;

    for overlay in [false, true] {
        b.push();
        b.anchor_to(BodyPart::Torso);
        b.translate(0.0, -10.0, 0.0);
        b.rotate(-features.chest_size * 45.0, 1.0, 0.0, 0.0);
        if overlay {
            b.push();
            b.translate(4.0, 2.0, 0.0);
            b.scale(8.5 / 8.0, 4.5 / 4.0, 1.0);
            b.translate(-4.0, -2.0, -0.25);
            b.double_sided(face(0.0, 48.0, 4.0, 4.0));
            b.translate(4.0, 0.0, 0.0);
            b.double_sided(face(12.0, 48.0, 4.0, 4.0));
            b.pop();
        } else {
            b.double_sided(face(20.0, 22.0, 8.0, 4.0));
        }

        b.push();
        b.translate(0.0, 4.0, 0.0);
        b.rotate(90.0, 1.0, 0.0, 0.0);
        if overlay {
            b.translate(0.0, 0.0, -0.25);
            b.double_sided(face(28.0, 48.0, 8.0, 4.0).grown(0.25));
        } else {
            b.double_sided(face(56.0, 44.0, 8.0, 4.0));
        }
        b.pop();

        b.push();
        b.rotate(90.0, 0.0, 1.0, 0.0);
        b.translate(-4.0, 0.0, 0.01);
        if overlay {
            b.translate(0.0, 0.0, -0.25);
            b.double_sided(face(48.0, 48.0, 4.0, 4.0).grown(0.25));
            b.translate(0.0, 0.0, 0.25);
        } else {
            b.double_sided(face(60.0, 48.0, 4.0, 4.0));
        }
        b.translate(0.0, 0.0, 7.98);
        b.rotate(180.0, 0.0, 1.0, 0.0);
        b.translate(-4.0, 0.0, 0.0);
        if overlay {
            b.translate(0.0, 0.0, -0.25);
            b.double_sided(
                face(48.0, 48.0, 4.0, 4.0).flipped(horizontal).grown(0.25),
            );
        } else {
            b.double_sided(face(60.0, 48.0, 4.0, 4.0).flipped(horizontal));
        }
        b.pop();
        b.pop();
    }
}

fn wings(b: &mut QuadBuilder, features: &Features) {
    let mode = features.wing_mode;
    if mode == WingMode::None {
        return;
    }

    let wiggle = if features.wing_animation_mode == WingAnimationMode::None {
        0.0
    } else {
        (8.0_f32 / 12.0).sin() * 2.0
    };
    let full_wing = face(0.0, 0.0, 20.0, 16.0);

    b.push();
    b.anchor_to(BodyPart::Torso);
    b.bind(EarsTextureSlot::Wing);
    b.translate(2.0, -14.0, 4.0);

    if matches!(mode, WingMode::SymmetricDual | WingMode::AsymmetricR) {
        b.push();
        b.rotate(-120.0 + wiggle, 0.0, 1.0, 0.0);
        b.double_sided(full_wing);
        b.pop();
    }
    if matches!(mode, WingMode::SymmetricDual | WingMode::AsymmetricL) {
        b.translate(4.0, 0.0, 0.0);
        b.push();
        b.rotate(-60.0 - wiggle, 0.0, 1.0, 0.0);
        b.double_sided(full_wing);
        b.pop();
    }
    if mode == WingMode::SymmetricSingle {
        b.translate(2.0, 0.0, 0.0);
        b.push();
        b.rotate(-90.0 + wiggle, 0.0, 1.0, 0.0);
        b.double_sided(full_wing);
        b.pop();
    }
    if mode == WingMode::AsymmetricDual {
        b.push();
        b.rotate(-120.0 + wiggle, 0.0, 1.0, 0.0);
        b.double_sided(face(0.0, 0.0, 10.0, 16.0));
        b.pop();
        b.translate(4.0, 0.0, 0.0);
        b.push();
        b.rotate(-60.0 - wiggle, 0.0, 1.0, 0.0);
        b.double_sided(face(10.0, 0.0, 10.0, 16.0));
        b.pop();
    }
    if mode == WingMode::Flat {
        b.translate(-8.0, 0.0, 0.75);
        b.double_sided(full_wing);
    }
    b.bind(EarsTextureSlot::Skin);
    b.pop();
}

fn cape(b: &mut QuadBuilder, features: &Features) {
    if !features.cape_enabled {
        return;
    }
    let horizontal = Flip::Horizontal;
    let vertical = Flip::Vertical;

    b.push();
    b.anchor_to(BodyPart::Torso);
    b.translate(4.0, -12.0, 5.0);
    b.rotate(6.0, 1.0, 0.0, 0.0);
    b.rotate(180.0, 0.0, 1.0, 0.0);
    b.bind(EarsTextureSlot::Cape);
    b.translate(-5.0, 0.0, 0.0);
    b.double_sided(face(0.0, 0.0, 10.0, 16.0));

    b.push();
    b.translate(10.0, 0.0, 1.0);
    b.rotate(90.0, 0.0, 1.0, 0.0);
    b.double_sided(face(9.0, 0.0, 1.0, 16.0).flipped(horizontal));
    b.rotate(90.0, 0.0, 1.0, 0.0);
    b.double_sided(face(10.0, 0.0, 10.0, 16.0));
    b.translate(10.0, 0.0, 1.0);
    b.rotate(90.0, 0.0, 1.0, 0.0);
    b.double_sided(face(0.0, 0.0, 1.0, 16.0).flipped(horizontal));
    b.pop();

    b.rotate(90.0, 1.0, 0.0, 0.0);
    b.double_sided(face(0.0, 0.0, 10.0, 1.0).flipped(vertical));
    b.translate(0.0, 0.0, -16.0);
    b.double_sided(face(0.0, 15.0, 10.0, 1.0).flipped(vertical));
    b.bind(EarsTextureSlot::Skin);
    b.pop();
}
