use glam::{Mat4, Vec3};

use crate::ears::{Ears, EarsTextureSlot, QUAD_INDICES, TexturePair};
use crate::model::{
    BodyPart, CAPE_MATERIAL, Model, OVERLAY_MESH_SUFFIX, PIXELS_PER_UNIT,
};
use crate::texture::{SkinTexture, Texture};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Material {
    /// Back faces are drawn too, lit as if they faced the viewer.
    pub double_sided: bool,
    pub blend: bool,
    /// The texture color is added to the lit color.
    pub emissive: bool,
    /// Blended triangles with a higher order are drawn later.
    pub order: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    /// Counter-clockwise when seen from the front.
    pub positions: [Vec3; 3],
    pub uvs: [[f32; 2]; 3],
    /// An index into [`Scene::textures`].
    pub texture: usize,
    pub material: Material,
}

/// Triangles are in the model's root space.
#[derive(Clone, Debug, Default)]
pub struct Scene {
    pub textures: Vec<Texture>,
    pub triangles: Vec<Triangle>,
}

const BODY: Material = Material {
    double_sided: false,
    blend: false,
    emissive: false,
    order: 0,
};
const OVERLAY: Material = Material {
    double_sided: true,
    blend: true,
    emissive: false,
    order: 0,
};
const CAPE: Material = Material {
    double_sided: true,
    blend: false,
    emissive: false,
    order: 0,
};
const FEATURE: Material = Material {
    double_sided: false,
    blend: true,
    emissive: false,
    order: 0,
};

const fn glowing(order: u8) -> Material {
    Material {
        double_sided: false,
        blend: true,
        emissive: true,
        order,
    }
}

impl Scene {
    /// Uses the model's rest pose.
    pub fn build(
        model: &Model,
        skin: &SkinTexture,
        cape: Option<&Texture>,
    ) -> Self {
        let slim =
            model
                .base_mesh_bounds(BodyPart::RightArm)
                .is_some_and(|bounds| {
                    (bounds.max.x - bounds.min.x) * PIXELS_PER_UNIT < 3.5
                });
        let ears = Ears::build(skin, slim);

        let mut scene = Self::default();
        let (skin, skin_glow) = match &ears {
            Some(ears) => scene.add_pair(&ears.skin),
            None => (scene.add_texture(skin.normalized()), None),
        };
        let cape = cape.map(|cape| scene.add_texture(cape.clone()));

        let transforms = model.world_transforms();
        for (node, transform) in model.nodes.iter().zip(&transforms) {
            let Some(mesh) = node.mesh.and_then(|mesh| model.meshes.get(mesh))
            else {
                continue;
            };
            let overlay = node.name.ends_with(OVERLAY_MESH_SUFFIX);

            for primitive in &mesh.primitives {
                let is_cape =
                    primitive.material.as_deref() == Some(CAPE_MATERIAL);
                let (texture, material) = match (is_cape, cape) {
                    (true, Some(cape)) => (cape, CAPE),
                    (true, None) => continue,
                    (false, _) if overlay => (skin, OVERLAY),
                    (false, _) => (skin, BODY),
                };

                for corners in primitive.indices.chunks_exact(3) {
                    let corners = [corners[0], corners[1], corners[2]]
                        .map(|index| index as usize);
                    let triangle = Triangle {
                        positions: corners.map(|index| {
                            transform.transform_point3(
                                primitive.positions[index].into(),
                            )
                        }),
                        uvs: corners.map(|index| primitive.uvs[index]),
                        texture,
                        material,
                    };
                    scene.triangles.push(triangle);
                    if let (false, Some(glow)) = (is_cape, skin_glow) {
                        scene.triangles.push(Triangle {
                            texture: glow,
                            material: glowing(2),
                            ..triangle
                        });
                    }
                }
            }
        }

        if let Some(ears) = &ears {
            scene.add_ears(model, &transforms, ears, (skin, skin_glow));
        }
        scene
    }

    pub fn transform(&mut self, matrix: &Mat4) {
        for triangle in &mut self.triangles {
            triangle.positions = triangle
                .positions
                .map(|point| matrix.transform_point3(point));
        }
    }

    fn add_texture(&mut self, texture: Texture) -> usize {
        self.textures.push(texture);
        self.textures.len() - 1
    }

    fn add_pair(&mut self, pair: &TexturePair) -> (usize, Option<usize>) {
        (
            self.add_texture(pair.base.clone()),
            pair.emissive
                .as_ref()
                .map(|emissive| self.add_texture(emissive.clone())),
        )
    }

    fn add_ears(
        &mut self,
        model: &Model,
        transforms: &[Mat4],
        ears: &Ears,
        skin: (usize, Option<usize>),
    ) {
        let slots = [
            EarsTextureSlot::Skin,
            EarsTextureSlot::DisplacedSkin,
            EarsTextureSlot::Wing,
            EarsTextureSlot::Cape,
        ]
        .map(|slot| match slot {
            EarsTextureSlot::Skin => Some(skin),
            _ => ears.texture(slot).map(|pair| self.add_pair(pair)),
        });

        let unit = 1.0 / PIXELS_PER_UNIT;
        let anchors = BodyPart::ALL.map(|part| {
            let node = model.find_node(part.node_name())?;
            let bounds = model.base_mesh_bounds(part)?;
            Some(
                transforms[node]
                    * Mat4::from_translation(Vec3::new(
                        bounds.max.x,
                        bounds.min.y,
                        bounds.min.z,
                    ))
                    * Mat4::from_scale(Vec3::splat(unit)),
            )
        });

        for quad in &ears.quads {
            let Some(anchor) = anchors[quad.part as usize] else {
                continue;
            };
            let Some((texture, glow)) = slots[quad.texture as usize] else {
                continue;
            };
            let positions = quad
                .positions
                .map(|position| anchor.transform_point3(position.into()));

            for corners in QUAD_INDICES.chunks_exact(3) {
                let corners = [corners[0], corners[1], corners[2]];
                let triangle = Triangle {
                    positions: corners.map(|index| positions[index]),
                    uvs: corners.map(|index| quad.uvs[index]),
                    texture,
                    material: FEATURE,
                };
                self.triangles.push(triangle);
                if let Some(glow) = glow {
                    self.triangles.push(Triangle {
                        texture: glow,
                        material: glowing(1),
                        ..triangle
                    });
                }
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::model::{Mesh, Node, Primitive};

    fn cuboid(min: [f32; 3], max: [f32; 3], material: Option<&str>) -> Mesh {
        let corner = |x: bool, y: bool, z: bool| {
            [
                if x { max[0] } else { min[0] },
                if y { max[1] } else { min[1] },
                if z { max[2] } else { min[2] },
            ]
        };
        let faces = [
            [
                (true, false, false),
                (false, false, false),
                (false, true, false),
                (true, true, false),
            ],
            [
                (false, false, true),
                (true, false, true),
                (true, true, true),
                (false, true, true),
            ],
            [
                (false, false, false),
                (true, false, false),
                (true, false, true),
                (false, false, true),
            ],
            [
                (false, true, true),
                (true, true, true),
                (true, true, false),
                (false, true, false),
            ],
            [
                (false, false, false),
                (false, false, true),
                (false, true, true),
                (false, true, false),
            ],
            [
                (true, false, true),
                (true, false, false),
                (true, true, false),
                (true, true, true),
            ],
        ];
        let mut primitive = Primitive {
            material: material.map(str::to_owned),
            ..Primitive::default()
        };
        for face in faces {
            let base = primitive.positions.len() as u32;
            for (x, y, z) in face {
                primitive.positions.push(corner(x, y, z));
                primitive.uvs.push([0.2, 0.2]);
            }
            primitive
                .indices
                .extend([0, 1, 2, 0, 2, 3].map(|index| base + index));
        }
        Mesh {
            primitives: vec![primitive],
        }
    }

    pub(crate) fn test_model() -> Model {
        let parts: [(&str, [f32; 3], [f32; 3]); 6] = [
            ("Head", [-0.25, 1.5, -0.25], [0.25, 2.0, 0.25]),
            ("Body", [-0.25, 0.75, -0.125], [0.25, 1.5, 0.125]),
            ("Right_Arm", [0.25, 0.75, -0.125], [0.5, 1.5, 0.125]),
            ("Left_Arm", [-0.5, 0.75, -0.125], [-0.25, 1.5, 0.125]),
            ("Right_Leg", [0.0, 0.0, -0.125], [0.25, 0.75, 0.125]),
            ("Left_Leg", [-0.25, 0.0, -0.125], [0.0, 0.75, 0.125]),
        ];

        let mut model = Model {
            nodes: vec![Node {
                name: "Root".into(),
                ..Node::default()
            }],
            roots: vec![0],
            meshes: Vec::new(),
        };
        fn add(
            model: &mut Model,
            parent: usize,
            name: String,
            mesh: Option<Mesh>,
        ) -> usize {
            let mesh = mesh.map(|mesh| {
                model.meshes.push(mesh);
                model.meshes.len() - 1
            });
            model.nodes.push(Node {
                name,
                mesh,
                ..Node::default()
            });
            let index = model.nodes.len() - 1;
            model.nodes[parent].children.push(index);
            index
        }

        for (name, min, max) in parts {
            let part = add(&mut model, 0, name.to_owned(), None);
            add(
                &mut model,
                part,
                format!("{name}_2"),
                Some(cuboid(min, max, None)),
            );
            let grow = 1.0 / 32.0;
            add(
                &mut model,
                part,
                format!("{name}_Layer"),
                Some(cuboid(
                    min.map(|value| value - grow),
                    max.map(|value| value + grow),
                    None,
                )),
            );
        }
        add(
            &mut model,
            0,
            "Cape".to_owned(),
            Some(cuboid([-0.3, 0.5, 0.13], [0.3, 1.5, 0.19], Some("cape"))),
        );
        model
    }

    fn opaque_skin() -> SkinTexture {
        SkinTexture::new(
            Texture::from_pixels(64, 64, vec![[90, 90, 90, 255]; 64 * 64])
                .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn the_cape_is_only_drawn_when_a_cape_texture_is_given() {
        let model = test_model();
        let skin = opaque_skin();

        let without = Scene::build(&model, &skin, None);
        assert_eq!(without.triangles.len(), 12 * 12);
        assert_eq!(without.textures.len(), 1);

        let cape = Texture::new(64, 32);
        let with = Scene::build(&model, &skin, Some(&cape));
        assert_eq!(with.triangles.len(), 13 * 12);
        let cape_triangles: Vec<_> =
            with.triangles.iter().filter(|t| t.texture == 1).collect();
        assert_eq!(cape_triangles.len(), 12);
        assert!(cape_triangles.iter().all(|t| t.material == CAPE));
    }

    #[test]
    fn overlay_meshes_blend_and_base_meshes_do_not() {
        let scene = Scene::build(&test_model(), &opaque_skin(), None);
        let blended =
            scene.triangles.iter().filter(|t| t.material.blend).count();
        assert_eq!(blended, 6 * 12);
    }

    #[test]
    fn ears_quads_are_placed_from_the_body_part_anchor() {
        let skin =
            SkinTexture::new(crate::ears::test_support::default_ears_skin())
                .unwrap();
        let model = test_model();

        let plain = Scene::build(&model, &opaque_skin(), None);
        let scene = Scene::build(&model, &skin, None);
        assert_eq!(scene.triangles.len(), plain.triangles.len() + 8);

        let ear = scene.triangles[plain.triangles.len()];
        assert_eq!(ear.material, FEATURE);
        let top = ear.positions.iter().map(|p| p.y).fold(f32::MIN, f32::max);
        let left = ear.positions.iter().map(|p| p.x).fold(f32::MAX, f32::min);
        let right = ear.positions.iter().map(|p| p.x).fold(f32::MIN, f32::max);
        assert!((top - 2.5).abs() < 1e-5);
        assert!((left + 0.5).abs() < 1e-5);
        assert!((right - 0.5).abs() < 1e-5);
        assert!(ear.positions.iter().all(|p| p.z.abs() < 1e-5));
    }

    #[test]
    fn transforming_a_scene_moves_its_triangles() {
        let mut scene = Scene::build(&test_model(), &opaque_skin(), None);
        let before = scene.triangles[0].positions[0];
        scene.transform(&Mat4::from_translation(Vec3::Y));
        assert_eq!(scene.triangles[0].positions[0], before + Vec3::Y);
    }
}
