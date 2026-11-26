use glam::{Mat4, Vec3};

mod gltf;

#[derive(Clone, Debug, Default)]
pub struct Model {
    pub nodes: Vec<Node>,
    pub roots: Vec<usize>,
    pub meshes: Vec<Mesh>,
}

#[derive(Clone, Debug, Default)]
pub struct Node {
    pub name: String,
    pub transform: Mat4,
    pub children: Vec<usize>,
    pub mesh: Option<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub primitives: Vec<Primitive>,
}

#[derive(Clone, Debug, Default)]
pub struct Primitive {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub material: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum BodyPart {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

impl BodyPart {
    pub const ALL: [Self; 6] = [
        Self::Head,
        Self::Torso,
        Self::LeftArm,
        Self::RightArm,
        Self::LeftLeg,
        Self::RightLeg,
    ];

    pub const fn node_name(self) -> &'static str {
        match self {
            Self::Head => "Head",
            Self::Torso => "Body",
            Self::LeftArm => "Left_Arm",
            Self::RightArm => "Right_Arm",
            Self::LeftLeg => "Left_Leg",
            Self::RightLeg => "Right_Leg",
        }
    }
}

const BASE_MESH_SUFFIX: &str = "_2";
pub(crate) const OVERLAY_MESH_SUFFIX: &str = "_Layer";
pub(crate) const CAPE_MATERIAL: &str = "cape";
/// Skin pixels per model unit. A 32 pixel tall player that is two units tall uses 16.
pub(crate) const PIXELS_PER_UNIT: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
}

impl Model {
    pub fn find_node(&self, name: &str) -> Option<usize> {
        self.nodes.iter().position(|node| node.name == name)
    }

    /// Indexed like [`Model::nodes`].
    pub fn world_transforms(&self) -> Vec<Mat4> {
        let mut transforms = vec![Mat4::IDENTITY; self.nodes.len()];
        let mut pending: Vec<(usize, Mat4)> = self
            .roots
            .iter()
            .map(|&root| (root, Mat4::IDENTITY))
            .collect();
        let mut visited = vec![false; self.nodes.len()];

        while let Some((index, parent)) = pending.pop() {
            let Some(node) = self.nodes.get(index) else {
                continue;
            };
            if std::mem::replace(&mut visited[index], true) {
                continue;
            }
            let world = parent * node.transform;
            transforms[index] = world;
            pending.extend(node.children.iter().map(|&child| (child, world)));
        }

        transforms
    }

    /// Relative to the model root.
    pub fn node_position(&self, part: BodyPart) -> Option<Vec3> {
        let index = self.find_node(part.node_name())?;
        Some(self.world_transforms()[index].w_axis.truncate())
    }

    /// In the body part node's own space.
    pub fn base_mesh_bounds(&self, part: BodyPart) -> Option<Bounds> {
        let node = &self.nodes[self.find_node(part.node_name())?];
        let base_name = format!("{}{BASE_MESH_SUFFIX}", part.node_name());
        let mesh_node = node.children.iter().find_map(|&child| {
            self.nodes
                .get(child)
                .filter(|child| child.mesh.is_some() && child.name == base_name)
        })?;
        let mesh = self.meshes.get(mesh_node.mesh?)?;

        let mut bounds: Option<Bounds> = None;
        for position in mesh
            .primitives
            .iter()
            .flat_map(|primitive| &primitive.positions)
        {
            let point =
                mesh_node.transform.transform_point3((*position).into());
            bounds = Some(bounds.map_or(
                Bounds {
                    min: point,
                    max: point,
                },
                |bounds| Bounds {
                    min: bounds.min.min(point),
                    max: bounds.max.max(point),
                },
            ));
        }
        bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_transforms_accumulate_down_the_hierarchy() {
        let model = Model {
            nodes: vec![
                Node {
                    name: "Root".into(),
                    transform: Mat4::from_translation(Vec3::Y),
                    children: vec![1],
                    mesh: None,
                },
                Node {
                    name: "Head".into(),
                    transform: Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)),
                    children: vec![],
                    mesh: None,
                },
            ],
            roots: vec![0],
            meshes: vec![],
        };
        assert_eq!(
            model.node_position(BodyPart::Head),
            Some(Vec3::new(2.0, 1.0, 0.0))
        );
    }

    #[test]
    fn base_mesh_bounds_include_the_mesh_node_offset() {
        let model = Model {
            nodes: vec![
                Node {
                    name: "Head".into(),
                    transform: Mat4::IDENTITY,
                    children: vec![1],
                    mesh: None,
                },
                Node {
                    name: "Head_2".into(),
                    transform: Mat4::from_translation(Vec3::new(
                        0.0, -1.5, 0.0,
                    )),
                    children: vec![],
                    mesh: Some(0),
                },
            ],
            roots: vec![0],
            meshes: vec![Mesh {
                primitives: vec![Primitive {
                    positions: vec![[-0.25, 1.5, -0.25], [0.25, 2.0, 0.25]],
                    ..Primitive::default()
                }],
            }],
        };
        let bounds = model.base_mesh_bounds(BodyPart::Head).unwrap();
        assert_eq!(bounds.min, Vec3::new(-0.25, 0.0, -0.25));
        assert_eq!(bounds.max, Vec3::new(0.25, 0.5, 0.25));
    }
}
