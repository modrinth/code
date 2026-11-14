use crate::math::{Mat4, Vec3};

#[cfg(feature = "gltf")]
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
}

/// The node, mesh and material names the crate looks for in a [`Model`].
#[derive(Clone, Debug)]
pub struct ModelConventions {
    pub head: String,
    pub torso: String,
    pub left_arm: String,
    pub right_arm: String,
    pub left_leg: String,
    pub right_leg: String,
    pub base_mesh_suffix: String,
    pub overlay_mesh_suffix: String,
    pub cape_material: String,
    /// Skin pixels per model unit. A 32 pixel tall player that is two units tall uses 16.
    pub pixels_per_unit: f32,
}

impl Default for ModelConventions {
    fn default() -> Self {
        Self {
            head: "Head".into(),
            torso: "Body".into(),
            left_arm: "Left_Arm".into(),
            right_arm: "Right_Arm".into(),
            left_leg: "Left_Leg".into(),
            right_leg: "Right_Leg".into(),
            base_mesh_suffix: "_2".into(),
            overlay_mesh_suffix: "_Layer".into(),
            cape_material: "cape".into(),
            pixels_per_unit: 16.0,
        }
    }
}

impl ModelConventions {
    pub fn body_part_node(&self, part: BodyPart) -> &str {
        match part {
            BodyPart::Head => &self.head,
            BodyPart::Torso => &self.torso,
            BodyPart::LeftArm => &self.left_arm,
            BodyPart::RightArm => &self.right_arm,
            BodyPart::LeftLeg => &self.left_leg,
            BodyPart::RightLeg => &self.right_leg,
        }
    }
}

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
    pub fn node_position(&self, name: &str) -> Option<Vec3> {
        let index = self.find_node(name)?;
        Some(self.world_transforms()[index].translation_part())
    }

    /// In the body part node's own space.
    pub fn base_mesh_bounds(
        &self,
        part: BodyPart,
        conventions: &ModelConventions,
    ) -> Option<Bounds> {
        let node_name = conventions.body_part_node(part);
        let node = &self.nodes[self.find_node(node_name)?];
        let base_name = format!("{node_name}{}", conventions.base_mesh_suffix);

        let direct = node.children.iter().copied().find(|&child| {
            self.nodes.get(child).is_some_and(|child| {
                child.mesh.is_some() && child.name == base_name
            })
        });
        let mesh_node = direct.or_else(|| {
            self.descendants(node).into_iter().find(|&index| {
                let candidate = &self.nodes[index];
                candidate.mesh.is_some()
                    && candidate.name.ends_with(&conventions.base_mesh_suffix)
            })
        })?;

        let mesh_node = &self.nodes[mesh_node];
        let mesh = self.meshes.get(mesh_node.mesh?)?;
        let mut bounds: Option<Bounds> = None;
        for position in mesh
            .primitives
            .iter()
            .flat_map(|primitive| &primitive.positions)
        {
            let point = mesh_node.transform.transform_point((*position).into());
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

    fn descendants(&self, node: &Node) -> Vec<usize> {
        let mut found = Vec::new();
        let mut pending: Vec<usize> =
            node.children.iter().rev().copied().collect();
        while let Some(index) = pending.pop() {
            if found.contains(&index) {
                continue;
            }
            let Some(child) = self.nodes.get(index) else {
                continue;
            };
            found.push(index);
            pending.extend(child.children.iter().rev().copied());
        }
        found
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
                    name: "root".into(),
                    transform: Mat4::translation(0.0, 1.0, 0.0),
                    children: vec![1],
                    mesh: None,
                },
                Node {
                    name: "child".into(),
                    transform: Mat4::translation(2.0, 0.0, 0.0),
                    children: vec![],
                    mesh: None,
                },
            ],
            roots: vec![0],
            meshes: vec![],
        };
        assert_eq!(
            model.node_position("child"),
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
                    transform: Mat4::translation(0.0, -1.5, 0.0),
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
        let bounds = model
            .base_mesh_bounds(BodyPart::Head, &ModelConventions::default())
            .unwrap();
        assert_eq!(bounds.min, Vec3::new(-0.25, 0.0, -0.25));
        assert_eq!(bounds.max, Vec3::new(0.25, 0.5, 0.25));
    }
}
