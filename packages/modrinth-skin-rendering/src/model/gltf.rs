use glam::Mat4;
use gltf::buffer::Data;
use gltf::mesh::Mode;

use super::{Mesh, Model, Node, Primitive};
use crate::error::{Error, Result};

impl Model {
    /// Takes `.gltf` JSON or `.glb` bytes with embedded buffers. Reads geometry and node
    /// transforms only.
    pub fn from_gltf(data: &[u8]) -> Result<Self> {
        let gltf::Gltf { document, blob } = gltf::Gltf::from_slice(data)?;
        let buffers = gltf::import_buffers(&document, None, blob)?;

        let meshes = document
            .meshes()
            .map(|mesh| {
                let mut primitives = Vec::new();
                for primitive in mesh.primitives() {
                    if primitive.mode() == Mode::Triangles
                        && let Some(primitive) =
                            read_primitive(&primitive, &buffers)?
                    {
                        primitives.push(primitive);
                    }
                }
                Ok(Mesh { primitives })
            })
            .collect::<Result<Vec<_>>>()?;

        let nodes: Vec<Node> = document
            .nodes()
            .map(|node| Node {
                name: node.name().unwrap_or_default().to_owned(),
                transform: Mat4::from_cols_array_2d(&node.transform().matrix()),
                children: node.children().map(|child| child.index()).collect(),
                mesh: node.mesh().map(|mesh| mesh.index()),
            })
            .collect();

        let scene = document
            .default_scene()
            .or_else(|| document.scenes().next())
            .ok_or(Error::InvalidModel("no scene"))?;
        let roots = scene.nodes().map(|node| node.index()).collect();

        Ok(Self {
            nodes,
            roots,
            meshes,
        })
    }
}

fn read_primitive(
    primitive: &gltf::Primitive<'_>,
    buffers: &[Data],
) -> Result<Option<Primitive>> {
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    if primitive.get(&gltf::Semantic::Positions).is_none() {
        return Ok(None);
    }

    let out_of_range = || Error::InvalidModel("accessor out of range");
    let positions: Vec<[f32; 3]> =
        reader.read_positions().ok_or_else(out_of_range)?.collect();
    let uvs: Vec<[f32; 2]> = match primitive.get(&gltf::Semantic::TexCoords(0))
    {
        Some(_) => reader
            .read_tex_coords(0)
            .ok_or_else(out_of_range)?
            .into_f32()
            .collect(),
        None => vec![[0.0; 2]; positions.len()],
    };
    let indices: Vec<u32> = match primitive.indices() {
        Some(_) => reader
            .read_indices()
            .ok_or_else(out_of_range)?
            .into_u32()
            .collect(),
        None => (0..positions.len() as u32).collect(),
    };

    if uvs.len() != positions.len() {
        return Err(Error::InvalidModel(
            "texture coordinate count differs from vertex count",
        ));
    }
    if indices
        .iter()
        .any(|&index| index as usize >= positions.len())
    {
        return Err(Error::InvalidModel("index out of range"));
    }

    Ok(Some(Primitive {
        positions,
        uvs,
        indices,
        material: primitive.material().name().map(str::to_owned),
    }))
}
