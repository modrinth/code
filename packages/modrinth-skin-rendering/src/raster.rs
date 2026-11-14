use std::collections::HashMap;
use std::sync::LazyLock;

use crate::math::{Mat4, Vec3};
use crate::render::Lighting;
use crate::scene::{Scene, Triangle};
use crate::texture::Texture;

const ALPHA_TEST: u8 = 26;

static SRGB_TO_LINEAR: LazyLock<[f32; 256]> = LazyLock::new(|| {
    std::array::from_fn(|value| {
        let c = value as f32 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    })
});

fn linear_to_srgb(value: f32) -> f32 {
    let c = value.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[derive(Clone, Copy)]
struct Vertex {
    x: f32,
    y: f32,
    depth: f32,
    inverse_w: f32,
    u_over_w: f32,
    v_over_w: f32,
}

/// Color is premultiplied by alpha.
pub(crate) struct Framebuffer {
    width: usize,
    height: usize,
    color: Vec<[u8; 4]>,
    depth: Vec<f32>,
}

type ShadeTable = [f32; 256];

fn shade_table(light: f32) -> ShadeTable {
    std::array::from_fn(|value| linear_to_srgb(SRGB_TO_LINEAR[value] * light))
}

impl Framebuffer {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        let size = width as usize * height as usize;
        Self {
            width: width as usize,
            height: height as usize,
            color: vec![[0; 4]; size],
            depth: vec![f32::INFINITY; size],
        }
    }

    /// Opaque triangles are drawn first, then blended ones from back to front.
    pub(crate) fn draw(
        &mut self,
        scene: &Scene,
        view_projection: &Mat4,
        lighting: &Lighting,
    ) {
        let light_direction = lighting.direction.normalize();
        let mut shade_tables: HashMap<u32, ShadeTable> = HashMap::new();
        let mut shade_for = |prepared: &Prepared, emissive: bool| {
            let diffuse = lighting.ambient
                + lighting.directional
                    * prepared.normal.dot(light_direction).max(0.0);
            let light = diffuse + if emissive { 1.0 } else { 0.0 };
            *shade_tables
                .entry(light.to_bits())
                .or_insert_with(|| shade_table(light))
        };

        let mut blended = Vec::new();
        for triangle in &scene.triangles {
            let Some(texture) = scene.textures.get(triangle.texture) else {
                continue;
            };
            let Some(prepared) = self.prepare(triangle, view_projection) else {
                continue;
            };
            if triangle.material.blend {
                let depth =
                    prepared.vertices.iter().map(|v| v.depth).sum::<f32>();
                blended.push((triangle, texture, prepared, depth));
            } else {
                let shade = shade_for(&prepared, triangle.material.emissive);
                self.fill(triangle, texture, &prepared, &shade);
            }
        }

        blended.sort_by(|a, b| {
            a.0.material
                .order
                .cmp(&b.0.material.order)
                .then(b.3.total_cmp(&a.3))
        });
        for (triangle, texture, prepared, _) in blended {
            let shade = shade_for(&prepared, triangle.material.emissive);
            self.fill(triangle, texture, &prepared, &shade);
        }
    }

    fn prepare(
        &self,
        triangle: &Triangle,
        view_projection: &Mat4,
    ) -> Option<Prepared> {
        let mut vertices = [Vertex {
            x: 0.0,
            y: 0.0,
            depth: 0.0,
            inverse_w: 0.0,
            u_over_w: 0.0,
            v_over_w: 0.0,
        }; 3];
        for (vertex, (position, uv)) in vertices
            .iter_mut()
            .zip(triangle.positions.iter().zip(triangle.uvs))
        {
            let [x, y, z, w] = view_projection.transform_point4(*position);
            if w <= 1e-6 || z < -w {
                return None;
            }
            let inverse_w = 1.0 / w;
            *vertex = Vertex {
                x: (x * inverse_w * 0.5 + 0.5) * self.width as f32,
                y: (0.5 - y * inverse_w * 0.5) * self.height as f32,
                depth: z * inverse_w,
                inverse_w,
                u_over_w: uv[0] * inverse_w,
                v_over_w: uv[1] * inverse_w,
            };
        }

        let [a, b, c] = vertices;
        let area = edge(a, b, c.x, c.y);
        if area == 0.0 {
            return None;
        }
        let front_facing = area < 0.0;
        if !front_facing && !triangle.material.double_sided {
            return None;
        }

        let mut normal = (triangle.positions[1] - triangle.positions[0])
            .cross(triangle.positions[2] - triangle.positions[0])
            .normalize();
        if !front_facing {
            normal = -normal;
        }

        Some(Prepared {
            vertices,
            area,
            normal,
        })
    }

    fn fill(
        &mut self,
        triangle: &Triangle,
        texture: &Texture,
        prepared: &Prepared,
        shade: &ShadeTable,
    ) {
        let [a, b, c] = prepared.vertices;
        let material = triangle.material;

        let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as usize;
        let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as usize;
        let max_x = (a.x.max(b.x).max(c.x).ceil() as usize).min(self.width);
        let max_y = (a.y.max(b.y).max(c.y).ceil() as usize).min(self.height);
        let inverse_area = 1.0 / prepared.area;

        let edges = [(b, c), (c, a), (a, b)];
        let covers_boundary = edges.map(|(from, to)| {
            let (dx, dy) = (to.x - from.x, to.y - from.y);
            let (dx, dy) = if prepared.area < 0.0 {
                (dx, dy)
            } else {
                (-dx, -dy)
            };
            dy > 0.0 || (dy == 0.0 && dx < 0.0)
        });

        for y in min_y..max_y {
            let sample_y = y as f32 + 0.5;
            for x in min_x..max_x {
                let sample_x = x as f32 + 0.5;
                let mut weights = [0.0; 3];
                let mut inside = true;
                for (index, (from, to)) in edges.into_iter().enumerate() {
                    let weight =
                        edge(from, to, sample_x, sample_y) * inverse_area;
                    if weight < 0.0
                        || (weight == 0.0 && !covers_boundary[index])
                    {
                        inside = false;
                        break;
                    }
                    weights[index] = weight;
                }
                if !inside {
                    continue;
                }
                let [wa, wb, wc] = weights;

                let depth = wa * a.depth + wb * b.depth + wc * c.depth;
                let index = y * self.width + x;
                if depth > self.depth[index] {
                    continue;
                }

                let inverse_w =
                    wa * a.inverse_w + wb * b.inverse_w + wc * c.inverse_w;
                let u = (wa * a.u_over_w + wb * b.u_over_w + wc * c.u_over_w)
                    / inverse_w;
                let v = (wa * a.v_over_w + wb * b.v_over_w + wc * c.v_over_w)
                    / inverse_w;
                let texel = texture.sample(u, v);
                if texel[3] < ALPHA_TEST {
                    continue;
                }

                let alpha = if material.blend {
                    f32::from(texel[3]) / 255.0
                } else {
                    1.0
                };
                let source = [
                    shade[texel[0] as usize] * alpha,
                    shade[texel[1] as usize] * alpha,
                    shade[texel[2] as usize] * alpha,
                    alpha,
                ];

                let destination = &mut self.color[index];
                for (destination, source) in destination.iter_mut().zip(source)
                {
                    let blended = source * 255.0
                        + f32::from(*destination) * (1.0 - alpha);
                    *destination = blended.round().clamp(0.0, 255.0) as u8;
                }
                self.depth[index] = depth;
            }
        }
    }

    /// Averages `factor` by `factor` blocks into a straight-alpha image.
    pub(crate) fn resolve(&self, factor: u32) -> Texture {
        let factor = factor as usize;
        let (width, height) = (self.width / factor, self.height / factor);
        let mut output = Texture::new(width as u32, height as u32);

        for (index, pixel) in output.pixels_mut().iter_mut().enumerate() {
            let (x, y) = (index % width, index / width);
            let mut sum = [0_u32; 4];
            for sample_y in 0..factor {
                let row = (y * factor + sample_y) * self.width + x * factor;
                for sample in &self.color[row..row + factor] {
                    for (total, channel) in sum.iter_mut().zip(sample) {
                        *total += u32::from(*channel);
                    }
                }
            }

            if sum[3] == 0 {
                continue;
            }
            let channel = |total: u32| {
                (total as f32 / sum[3] as f32 * 255.0).round().min(255.0) as u8
            };
            *pixel = [
                channel(sum[0]),
                channel(sum[1]),
                channel(sum[2]),
                (sum[3] as f32 / (factor * factor) as f32).round() as u8,
            ];
        }
        output
    }
}

struct Prepared {
    vertices: [Vertex; 3],
    area: f32,
    normal: Vec3,
}

fn edge(from: Vertex, to: Vertex, x: f32, y: f32) -> f32 {
    (to.x - from.x) * (y - from.y) - (to.y - from.y) * (x - from.x)
}
