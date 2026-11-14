use base64::Engine;
use serde::Deserialize;

use super::{Mesh, Model, Node, Primitive};
use crate::error::{Error, Result};
use crate::math::Mat4;

#[derive(Deserialize)]
struct Document {
    #[serde(default)]
    scene: usize,
    #[serde(default)]
    scenes: Vec<Scene>,
    #[serde(default)]
    nodes: Vec<GltfNode>,
    #[serde(default)]
    meshes: Vec<GltfMesh>,
    #[serde(default)]
    materials: Vec<Material>,
    #[serde(default)]
    accessors: Vec<Accessor>,
    #[serde(default, rename = "bufferViews")]
    buffer_views: Vec<BufferView>,
    #[serde(default)]
    buffers: Vec<Buffer>,
}

#[derive(Deserialize)]
struct Scene {
    #[serde(default)]
    nodes: Vec<usize>,
}

#[derive(Deserialize)]
struct GltfNode {
    #[serde(default)]
    name: String,
    #[serde(default)]
    children: Vec<usize>,
    mesh: Option<usize>,
    matrix: Option<[f32; 16]>,
    translation: Option<[f32; 3]>,
    rotation: Option<[f32; 4]>,
    scale: Option<[f32; 3]>,
}

#[derive(Deserialize)]
struct GltfMesh {
    #[serde(default)]
    primitives: Vec<GltfPrimitive>,
}

#[derive(Deserialize)]
struct GltfPrimitive {
    attributes: Attributes,
    indices: Option<usize>,
    material: Option<usize>,
    mode: Option<u32>,
}

#[derive(Deserialize)]
struct Attributes {
    #[serde(rename = "POSITION")]
    position: Option<usize>,
    #[serde(rename = "TEXCOORD_0")]
    texcoord: Option<usize>,
}

#[derive(Deserialize)]
struct Material {
    name: Option<String>,
}

#[derive(Deserialize)]
struct Accessor {
    #[serde(rename = "bufferView")]
    buffer_view: Option<usize>,
    #[serde(default, rename = "byteOffset")]
    byte_offset: usize,
    #[serde(rename = "componentType")]
    component_type: u32,
    #[serde(default)]
    normalized: bool,
    count: usize,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct BufferView {
    buffer: usize,
    #[serde(default, rename = "byteOffset")]
    byte_offset: usize,
    #[serde(rename = "byteStride")]
    byte_stride: Option<usize>,
}

#[derive(Deserialize)]
struct Buffer {
    uri: Option<String>,
}

const TRIANGLES: u32 = 4;
const GLB_MAGIC: &[u8; 4] = b"glTF";
const GLB_JSON_CHUNK: u32 = 0x4E4F_534A;
const GLB_BINARY_CHUNK: u32 = 0x004E_4942;

impl Model {
    /// Takes `.gltf` JSON or `.glb` bytes with embedded buffers. Reads geometry and node
    /// transforms only.
    pub fn from_gltf(data: &[u8]) -> Result<Self> {
        Self::from_gltf_with_buffers(data, |_| None)
    }

    /// `load_buffer` resolves buffer URIs that are not data URIs.
    pub fn from_gltf_with_buffers(
        data: &[u8],
        mut load_buffer: impl FnMut(&str) -> Option<Vec<u8>>,
    ) -> Result<Self> {
        let (json, mut glb_binary) = split_glb(data)?;
        let document: Document = serde_json::from_slice(json)?;

        let mut buffers = Vec::with_capacity(document.buffers.len());
        for buffer in &document.buffers {
            buffers.push(match &buffer.uri {
                None => glb_binary.take().ok_or_else(|| {
                    Error::Gltf("buffer has no URI and no GLB chunk".into())
                })?,
                Some(uri) => match decode_data_uri(uri) {
                    Some(decoded) => decoded?,
                    None => load_buffer(uri).ok_or_else(|| {
                        Error::Gltf(format!("external buffer `{uri}`"))
                    })?,
                },
            });
        }

        let reader = AccessorReader {
            document: &document,
            buffers: &buffers,
        };

        let mut meshes = Vec::with_capacity(document.meshes.len());
        for mesh in &document.meshes {
            let mut primitives = Vec::new();
            for primitive in &mesh.primitives {
                if primitive.mode.unwrap_or(TRIANGLES) != TRIANGLES {
                    continue;
                }
                let Some(position) = primitive.attributes.position else {
                    continue;
                };
                let positions: Vec<[f32; 3]> = reader.read(position)?;
                let uvs: Vec<[f32; 2]> = match primitive.attributes.texcoord {
                    Some(texcoord) => reader.read(texcoord)?,
                    None => vec![[0.0; 2]; positions.len()],
                };
                if uvs.len() != positions.len() {
                    return Err(Error::Gltf(
                        "texture coordinate count differs from vertex count"
                            .into(),
                    ));
                }
                let indices = match primitive.indices {
                    Some(indices) => reader.read_indices(indices)?,
                    None => (0..positions.len() as u32).collect(),
                };
                if indices.iter().any(|&i| i as usize >= positions.len()) {
                    return Err(Error::Gltf("index out of range".into()));
                }
                primitives.push(Primitive {
                    positions,
                    uvs,
                    indices,
                    material: primitive
                        .material
                        .and_then(|index| document.materials.get(index))
                        .and_then(|material| material.name.clone()),
                });
            }
            meshes.push(Mesh { primitives });
        }

        let node_count = document.nodes.len();
        let nodes = document
            .nodes
            .iter()
            .map(|node| Node {
                name: node.name.clone(),
                transform: node.matrix.map(Mat4).unwrap_or_else(|| {
                    Mat4::from_trs(
                        node.translation.unwrap_or([0.0; 3]),
                        node.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]),
                        node.scale.unwrap_or([1.0; 3]),
                    )
                }),
                children: node
                    .children
                    .iter()
                    .copied()
                    .filter(|&child| child < node_count)
                    .collect(),
                mesh: node.mesh.filter(|&mesh| mesh < meshes.len()),
            })
            .collect();

        let roots = match document.scenes.get(document.scene) {
            Some(scene) => scene
                .nodes
                .iter()
                .copied()
                .filter(|&root| root < node_count)
                .collect(),
            None => {
                let mut is_child = vec![false; node_count];
                for node in &document.nodes {
                    for &child in &node.children {
                        if let Some(flag) = is_child.get_mut(child) {
                            *flag = true;
                        }
                    }
                }
                (0..node_count).filter(|&index| !is_child[index]).collect()
            }
        };

        Ok(Self {
            nodes,
            roots,
            meshes,
        })
    }
}

fn split_glb(data: &[u8]) -> Result<(&[u8], Option<Vec<u8>>)> {
    if !data.starts_with(GLB_MAGIC) {
        return Ok((data, None));
    }

    let truncated = || Error::Gltf("truncated GLB".into());
    let read_u32 = |offset: usize| -> Result<u32> {
        let bytes = data.get(offset..offset + 4).ok_or_else(truncated)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    };

    let mut json = None;
    let mut binary = None;
    let mut offset = 12;
    while offset + 8 <= data.len() {
        let length = read_u32(offset)? as usize;
        let kind = read_u32(offset + 4)?;
        let chunk = data
            .get(offset + 8..offset + 8 + length)
            .ok_or_else(truncated)?;
        match kind {
            GLB_JSON_CHUNK if json.is_none() => json = Some(chunk),
            GLB_BINARY_CHUNK if binary.is_none() => {
                binary = Some(chunk.to_vec());
            }
            _ => {}
        }
        offset += 8 + length.next_multiple_of(4);
    }

    Ok((json.ok_or_else(truncated)?, binary))
}

fn decode_data_uri(uri: &str) -> Option<Result<Vec<u8>>> {
    let rest = uri.strip_prefix("data:")?;
    let (header, payload) = rest.split_once(',')?;
    if !header.ends_with(";base64") {
        return Some(Err(Error::Gltf("unsupported data URI encoding".into())));
    }
    Some(
        base64::engine::general_purpose::STANDARD
            .decode(payload)
            .map_err(|error| {
                Error::Gltf(format!("bad base64 buffer: {error}"))
            }),
    )
}

struct AccessorReader<'a> {
    document: &'a Document,
    buffers: &'a [Vec<u8>],
}

impl AccessorReader<'_> {
    fn read<const N: usize>(&self, index: usize) -> Result<Vec<[f32; N]>> {
        let (accessor, components) = self.accessor(index)?;
        if components != N {
            return Err(Error::Gltf(format!(
                "accessor {index} has {components} components, expected {N}"
            )));
        }
        let mut out = Vec::with_capacity(accessor.count);
        self.for_each_element(accessor, components, |element| {
            let mut value = [0.0; N];
            for (slot, component) in value.iter_mut().zip(element) {
                *slot = *component;
            }
            out.push(value);
        })?;
        Ok(out)
    }

    fn read_indices(&self, index: usize) -> Result<Vec<u32>> {
        let (accessor, components) = self.accessor(index)?;
        if components != 1 || accessor.normalized {
            return Err(Error::Gltf(format!(
                "accessor {index} is not an index accessor"
            )));
        }
        let mut out = Vec::with_capacity(accessor.count);
        self.for_each_element(accessor, 1, |element| {
            out.push(element[0] as u32);
        })?;
        Ok(out)
    }

    fn accessor(&self, index: usize) -> Result<(&Accessor, usize)> {
        let accessor =
            self.document.accessors.get(index).ok_or_else(|| {
                Error::Gltf(format!("missing accessor {index}"))
            })?;
        let components = match accessor.kind.as_str() {
            "SCALAR" => 1,
            "VEC2" => 2,
            "VEC3" => 3,
            "VEC4" => 4,
            other => {
                return Err(Error::Gltf(format!(
                    "unsupported accessor type {other}"
                )));
            }
        };
        Ok((accessor, components))
    }

    fn for_each_element(
        &self,
        accessor: &Accessor,
        components: usize,
        mut visit: impl FnMut(&[f32]),
    ) -> Result<()> {
        let out_of_range = || Error::Gltf("accessor out of range".into());
        let view_index = accessor.buffer_view.ok_or_else(|| {
            Error::Gltf("sparse accessors are not supported".into())
        })?;
        let view = self
            .document
            .buffer_views
            .get(view_index)
            .ok_or_else(out_of_range)?;
        let buffer = self.buffers.get(view.buffer).ok_or_else(out_of_range)?;

        let component_size = match accessor.component_type {
            5120 | 5121 => 1,
            5122 | 5123 => 2,
            5125 | 5126 => 4,
            other => {
                return Err(Error::Gltf(format!(
                    "unsupported component type {other}"
                )));
            }
        };
        let element_size = component_size * components;
        let stride = view.byte_stride.unwrap_or(element_size).max(element_size);
        let start = view.byte_offset.saturating_add(accessor.byte_offset);

        let mut element = [0.0_f32; 4];
        for item in 0..accessor.count {
            let offset = start.saturating_add(item.saturating_mul(stride));
            let bytes = buffer
                .get(offset..offset.saturating_add(element_size))
                .ok_or_else(out_of_range)?;
            for (slot, raw) in
                element.iter_mut().zip(bytes.chunks_exact(component_size))
            {
                *slot = decode_component(
                    raw,
                    accessor.component_type,
                    accessor.normalized,
                );
            }
            visit(&element[..components]);
        }
        Ok(())
    }
}

fn decode_component(raw: &[u8], component_type: u32, normalized: bool) -> f32 {
    match component_type {
        5120 => {
            let value = f32::from(raw[0] as i8);
            if normalized {
                (value / 127.0).max(-1.0)
            } else {
                value
            }
        }
        5121 => {
            let value = f32::from(raw[0]);
            if normalized { value / 255.0 } else { value }
        }
        5122 => {
            let value = f32::from(i16::from_le_bytes([raw[0], raw[1]]));
            if normalized {
                (value / 32767.0).max(-1.0)
            } else {
                value
            }
        }
        5123 => {
            let value = f32::from(u16::from_le_bytes([raw[0], raw[1]]));
            if normalized { value / 65535.0 } else { value }
        }
        5125 => u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as f32,
        _ => f32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec3;

    fn triangle_gltf() -> String {
        let mut buffer = Vec::new();
        for value in [0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        for value in [0.0_f32, 0.0, 1.0, 0.0, 0.0, 1.0] {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        for index in [0_u16, 1, 2] {
            buffer.extend_from_slice(&index.to_le_bytes());
        }
        let encoded = base64::engine::general_purpose::STANDARD.encode(&buffer);

        format!(
            r#"{{
                "scene": 0,
                "scenes": [{{ "nodes": [0] }}],
                "nodes": [
                    {{ "name": "Root", "children": [1], "translation": [0, 2, 0] }},
                    {{ "name": "Part", "mesh": 0, "rotation": [0, 1, 0, 0] }}
                ],
                "meshes": [{{ "primitives": [{{
                    "attributes": {{ "POSITION": 0, "TEXCOORD_0": 1 }},
                    "indices": 2,
                    "material": 0
                }}] }}],
                "materials": [{{ "name": "cape" }}],
                "accessors": [
                    {{ "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3" }},
                    {{ "bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC2" }},
                    {{ "bufferView": 2, "componentType": 5123, "count": 3, "type": "SCALAR" }}
                ],
                "bufferViews": [
                    {{ "buffer": 0, "byteOffset": 0 }},
                    {{ "buffer": 0, "byteOffset": 36 }},
                    {{ "buffer": 0, "byteOffset": 60 }}
                ],
                "buffers": [{{ "uri": "data:application/octet-stream;base64,{encoded}" }}],
                "animations": [{{ "name": "ignored" }}]
            }}"#
        )
    }

    #[test]
    fn loads_geometry_hierarchy_and_material_names() {
        let model = Model::from_gltf(triangle_gltf().as_bytes()).unwrap();
        assert_eq!(model.roots, vec![0]);
        assert_eq!(model.nodes.len(), 2);

        let primitive = &model.meshes[0].primitives[0];
        assert_eq!(primitive.positions[1], [1.0, 0.0, 0.0]);
        assert_eq!(primitive.uvs[2], [0.0, 1.0]);
        assert_eq!(primitive.indices, vec![0, 1, 2]);
        assert_eq!(primitive.material.as_deref(), Some("cape"));

        let world = model.world_transforms()[1];
        let moved = world.transform_point(Vec3::new(1.0, 0.0, 0.0));
        assert!((moved - Vec3::new(-1.0, 2.0, 0.0)).length() < 1e-5);
    }

    #[test]
    fn external_buffers_need_a_loader() {
        let json = triangle_gltf();
        let start = json.find("data:").unwrap();
        let end = start + json[start..].find('"').unwrap();
        let external = format!("{}model.bin{}", &json[..start], &json[end..]);
        assert!(Model::from_gltf(external.as_bytes()).is_err());
    }

    #[test]
    fn rejects_accessors_that_run_past_their_buffer() {
        let json = triangle_gltf().replace(
            r#""componentType": 5123, "count": 3"#,
            r#""componentType": 5123, "count": 9"#,
        );
        assert!(Model::from_gltf(json.as_bytes()).is_err());
    }
}
