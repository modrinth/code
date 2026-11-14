use super::{EarsQuad, EarsTextureSlot};
use crate::math::{Mat4, Vec3};
use crate::model::BodyPart;

const UV_TEXEL_INSET: f32 = 1.0 / 32.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Rotation {
    None,
    Cw,
    UpsideDown,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Flip {
    None,
    Horizontal,
    Vertical,
    Both,
}

impl Flip {
    const fn flipped_horizontally(self) -> Self {
        match self {
            Self::None => Self::Horizontal,
            Self::Horizontal => Self::None,
            Self::Vertical => Self::Both,
            Self::Both => Self::Vertical,
        }
    }
}

/// In texels.
#[derive(Clone, Copy)]
pub(super) struct Face {
    u: f32,
    v: f32,
    width: f32,
    height: f32,
    rotation: Rotation,
    flip: Flip,
    grow: f32,
}

pub(super) const fn face(u: f32, v: f32, width: f32, height: f32) -> Face {
    Face {
        u,
        v,
        width,
        height,
        rotation: Rotation::None,
        flip: Flip::None,
        grow: 0.0,
    }
}

impl Face {
    pub(super) const fn rotated(mut self, rotation: Rotation) -> Self {
        self.rotation = rotation;
        self
    }

    pub(super) const fn flipped(mut self, flip: Flip) -> Self {
        self.flip = flip;
        self
    }

    pub(super) const fn grown(mut self, grow: f32) -> Self {
        self.grow = grow;
        self
    }
}

#[derive(Clone, Copy)]
struct State {
    anchor: Option<BodyPart>,
    matrix: Mat4,
}

/// Takes coordinates as the Ears mod does, with X and Y negated relative to the output.
pub(super) struct QuadBuilder {
    stack: Vec<State>,
    state: State,
    texture: EarsTextureSlot,
    available: [bool; 4],
    pub(super) quads: Vec<EarsQuad>,
}

impl QuadBuilder {
    pub(super) fn new(available: [bool; 4]) -> Self {
        Self {
            stack: Vec::new(),
            state: State {
                anchor: None,
                matrix: Mat4::IDENTITY,
            },
            texture: EarsTextureSlot::Skin,
            available,
            quads: Vec::new(),
        }
    }

    pub(super) fn push(&mut self) {
        self.stack.push(self.state);
    }

    pub(super) fn pop(&mut self) {
        if let Some(state) = self.stack.pop() {
            self.state = state;
        }
    }

    pub(super) fn anchor_to(&mut self, part: BodyPart) {
        self.state = State {
            anchor: Some(part),
            matrix: Mat4::IDENTITY,
        };
    }

    pub(super) fn bind(&mut self, texture: EarsTextureSlot) {
        self.texture = texture;
    }

    pub(super) fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.state.matrix = self.state.matrix * Mat4::translation(-x, -y, z);
    }

    pub(super) fn rotate(&mut self, degrees: f32, x: f32, y: f32, z: f32) {
        self.state.matrix = self.state.matrix
            * Mat4::rotation_axis(Vec3::new(-x, -y, z), degrees.to_radians());
    }

    pub(super) fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.state.matrix = self.state.matrix * Mat4::scale(x, y, z);
    }

    pub(super) fn front(&mut self, face: Face) {
        self.quad(face, false, [0.0; 3]);
    }

    pub(super) fn front_skew(&mut self, face: Face, skew: [f32; 3]) {
        self.quad(face, false, skew);
    }

    pub(super) fn back(&mut self, face: Face) {
        self.quad(
            face.flipped(face.flip.flipped_horizontally()),
            true,
            [0.0; 3],
        );
    }

    pub(super) fn double_sided(&mut self, face: Face) {
        self.front(face);
        self.back(face.flipped(face.flip.flipped_horizontally()));
    }

    fn quad(&mut self, face: Face, back: bool, [xs, ys, zs]: [f32; 3]) {
        let Some(part) = self.state.anchor else {
            return;
        };
        if !self.available[self.texture as usize] {
            return;
        }

        let (texture_width, texture_height) = self.texture.size();
        let uvs = face_uvs(face, texture_width as f32, texture_height as f32);
        let Face {
            width,
            height,
            grow,
            ..
        } = face;

        let (positions, uvs) = if back {
            (
                [
                    [grow + xs, grow + ys, -zs],
                    [-width - grow + xs, grow, -zs],
                    [-width - grow - xs, -height - grow, zs],
                    [grow - xs, -height - grow + ys, zs],
                ],
                [uvs[3], uvs[2], uvs[1], uvs[0]],
            )
        } else {
            (
                [
                    [grow - xs, -height - grow + ys, zs],
                    [-width - grow - xs, -height - grow - ys, zs],
                    [-width - grow + xs, grow - ys, -zs],
                    [grow + xs, grow + ys, -zs],
                ],
                uvs,
            )
        };

        self.quads.push(EarsQuad {
            part,
            texture: self.texture,
            positions: positions.map(|position| {
                self.state
                    .matrix
                    .transform_point(position.into())
                    .to_array()
            }),
            uvs,
        });
    }
}

fn face_uvs(
    face: Face,
    texture_width: f32,
    texture_height: f32,
) -> [[f32; 2]; 4] {
    let transposed = face.rotation == Rotation::Cw;
    let (span_u, span_v) = if transposed {
        (face.height, face.width)
    } else {
        (face.width, face.height)
    };
    let mut min_u = (face.u + UV_TEXEL_INSET) / texture_width;
    let mut min_v = (face.v + UV_TEXEL_INSET) / texture_height;
    let mut max_u = (face.u + span_u - UV_TEXEL_INSET) / texture_width;
    let mut max_v = (face.v + span_v - UV_TEXEL_INSET) / texture_height;

    let flip = match (transposed, face.flip) {
        (true, Flip::Horizontal) => Flip::Vertical,
        (true, Flip::Vertical) => Flip::Horizontal,
        (_, flip) => flip,
    };
    if matches!(flip, Flip::Horizontal | Flip::Both) {
        std::mem::swap(&mut min_u, &mut max_u);
    }
    if matches!(flip, Flip::Vertical | Flip::Both) {
        std::mem::swap(&mut min_v, &mut max_v);
    }

    let mut uvs = [
        [min_u, max_v],
        [max_u, max_v],
        [max_u, min_v],
        [min_u, min_v],
    ];
    match face.rotation {
        Rotation::None => {}
        Rotation::Cw => uvs.rotate_right(1),
        Rotation::UpsideDown => uvs.rotate_left(2),
    }
    uvs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builder() -> QuadBuilder {
        QuadBuilder::new([true; 4])
    }

    fn texels(uvs: [[f32; 2]; 4]) -> [[f32; 2]; 4] {
        uvs.map(|[u, v]| [(u * 64.0).round(), (v * 64.0).round()])
    }

    #[test]
    fn nothing_is_drawn_before_anchoring() {
        let mut builder = builder();
        builder.front(face(0.0, 0.0, 8.0, 8.0));
        assert!(builder.quads.is_empty());
    }

    #[test]
    fn quads_for_unavailable_textures_are_skipped() {
        let mut builder = QuadBuilder::new([true, false, false, false]);
        builder.anchor_to(BodyPart::Torso);
        builder.bind(EarsTextureSlot::Wing);
        builder.front(face(0.0, 0.0, 20.0, 16.0));
        assert!(builder.quads.is_empty());
    }

    #[test]
    fn a_front_quad_hangs_down_and_towards_negative_x_from_the_origin() {
        let mut builder = builder();
        builder.anchor_to(BodyPart::Head);
        builder.front(face(24.0, 0.0, 16.0, 8.0));

        let quad = &builder.quads[0];
        assert_eq!(quad.part, BodyPart::Head);
        assert_eq!(
            quad.positions,
            [
                [0.0, -8.0, 0.0],
                [-16.0, -8.0, 0.0],
                [-16.0, 0.0, 0.0],
                [0.0, 0.0, 0.0]
            ]
        );
        assert_eq!(
            texels(quad.uvs),
            [[24.0, 8.0], [40.0, 8.0], [40.0, 0.0], [24.0, 0.0]]
        );
    }

    #[test]
    fn translation_negates_x_and_y() {
        let mut builder = builder();
        builder.anchor_to(BodyPart::Head);
        builder.translate(-4.0, -16.0, 4.0);
        builder.front(face(24.0, 0.0, 16.0, 8.0));
        assert_eq!(builder.quads[0].positions[3], [4.0, 16.0, 4.0]);
    }

    #[test]
    fn back_quads_reverse_winding_and_mirror_the_texture() {
        let mut builder = builder();
        builder.anchor_to(BodyPart::Head);
        builder.back(face(8.0, 0.0, 4.0, 2.0));

        let quad = &builder.quads[0];
        assert_eq!(
            quad.positions,
            [
                [0.0, 0.0, 0.0],
                [-4.0, 0.0, 0.0],
                [-4.0, -2.0, 0.0],
                [0.0, -2.0, 0.0]
            ]
        );
        assert_eq!(
            texels(quad.uvs),
            [[12.0, 0.0], [8.0, 0.0], [8.0, 2.0], [12.0, 2.0]]
        );
    }

    #[test]
    fn double_sided_faces_show_the_same_texels_from_both_sides() {
        let mut builder = builder();
        builder.anchor_to(BodyPart::Torso);
        builder.double_sided(face(56.0, 16.0, 8.0, 12.0));

        let [front, back] = &builder.quads[..] else {
            panic!("expected two quads");
        };
        for (position, uv) in back.positions.iter().zip(back.uvs) {
            let index = front
                .positions
                .iter()
                .position(|candidate| candidate == position)
                .unwrap();
            assert_eq!(texels([uv; 4])[0], texels([front.uvs[index]; 4])[0]);
        }
    }

    #[test]
    fn clockwise_rotation_transposes_the_sampled_rectangle() {
        let uvs = texels(face_uvs(
            face(56.0, 28.0, 16.0, 8.0).rotated(Rotation::Cw),
            64.0,
            64.0,
        ));
        assert_eq!(
            uvs,
            [[56.0, 28.0], [56.0, 44.0], [64.0, 44.0], [64.0, 28.0]]
        );
    }

    #[test]
    fn push_and_pop_restore_the_transform_and_anchor() {
        let mut builder = builder();
        builder.anchor_to(BodyPart::Head);
        builder.push();
        builder.anchor_to(BodyPart::Torso);
        builder.translate(1.0, 2.0, 3.0);
        builder.pop();
        builder.front(face(0.0, 0.0, 1.0, 1.0));
        assert_eq!(builder.quads[0].part, BodyPart::Head);
        assert_eq!(builder.quads[0].positions[3], [0.0, 0.0, 0.0]);
    }
}
