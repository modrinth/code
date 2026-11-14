use crate::math::{Mat4, Vec3};
use crate::raster::Framebuffer;
use crate::scene::Scene;
use crate::texture::{SkinTexture, Texture};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y_degrees: f32,
    /// Triangles that cross the near plane are not drawn.
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(-1.6, 1.2, -5.4),
            target: Vec3::new(0.0, 1.0, 0.0),
            up: Vec3::Y,
            fov_y_degrees: 24.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

/// Each face is lit by `ambient + directional * max(0, normal · direction)` in linear color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lighting {
    pub ambient: f32,
    pub directional: f32,
    /// Points from the scene towards the light.
    pub direction: Vec3,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            ambient: 0.64,
            directional: 0.38,
            direction: Vec3::new(-2.0, 4.0, -3.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderOptions {
    pub width: u32,
    pub height: u32,
    /// Samples per pixel along each axis. Clamped to `1..=4`.
    pub supersampling: u32,
    pub camera: Camera,
    pub lighting: Lighting,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 360,
            height: 504,
            supersampling: 3,
            camera: Camera::default(),
            lighting: Lighting::default(),
        }
    }
}

pub fn render(scene: &Scene, options: &RenderOptions) -> Texture {
    let factor = options.supersampling.clamp(1, 4);
    let (width, height) = (options.width.max(1), options.height.max(1));
    let camera = &options.camera;

    let view_projection =
        Mat4::perspective(
            camera.fov_y_degrees.to_radians(),
            width as f32 / height as f32,
            camera.near,
            camera.far,
        ) * Mat4::look_at(camera.position, camera.target, camera.up);

    let mut framebuffer = Framebuffer::new(width * factor, height * factor);
    framebuffer.draw(scene, &view_projection, &options.lighting);
    framebuffer.resolve(factor)
}

/// The front of the head with the hat layer on top, scaled to `size` without smoothing.
pub fn render_face(skin: &SkinTexture, size: u32) -> Texture {
    let texture = skin.normalized();
    let has_hat = (0..8)
        .flat_map(|y| (0..8).map(move |x| (x, y)))
        .any(|(x, y)| texture.get(40 + x, 8 + y)[3] > 0);

    let mut output = Texture::new(size, size);
    for y in 0..size {
        for x in 0..size {
            let (source_x, source_y) = (x * 8 / size, y * 8 / size);
            let mut pixel = texture.get(8 + source_x, 8 + source_y);
            if has_hat {
                pixel = over(texture.get(40 + source_x, 8 + source_y), pixel);
            }
            output.set(x, y, pixel);
        }
    }
    output
}

fn over(top: [u8; 4], bottom: [u8; 4]) -> [u8; 4] {
    let top_alpha = f32::from(top[3]) / 255.0;
    let bottom_alpha = f32::from(bottom[3]) / 255.0 * (1.0 - top_alpha);
    let alpha = top_alpha + bottom_alpha;
    if alpha <= 0.0 {
        return [0; 4];
    }
    let channel = |index: usize| {
        ((f32::from(top[index]) * top_alpha
            + f32::from(bottom[index]) * bottom_alpha)
            / alpha)
            .round() as u8
    };
    [
        channel(0),
        channel(1),
        channel(2),
        (alpha * 255.0).round() as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModelConventions;
    use crate::scene::tests::test_model;
    use crate::scene::{Material, Player, Triangle};

    fn solid(pixel: [u8; 4]) -> Texture {
        Texture::from_pixels(1, 1, vec![pixel]).unwrap()
    }

    fn options(size: u32) -> RenderOptions {
        RenderOptions {
            width: size,
            height: size,
            supersampling: 1,
            camera: Camera {
                position: Vec3::new(0.0, 0.0, 4.0),
                target: Vec3::ZERO,
                up: Vec3::Y,
                fov_y_degrees: 60.0,
                near: 0.1,
                far: 100.0,
            },
            lighting: Lighting {
                ambient: 1.0,
                directional: 0.0,
                direction: Vec3::Y,
            },
        }
    }

    fn quad(
        z: f32,
        half: f32,
        texture: usize,
        material: Material,
    ) -> [Triangle; 2] {
        let corners = [
            Vec3::new(-half, -half, z),
            Vec3::new(half, -half, z),
            Vec3::new(half, half, z),
            Vec3::new(-half, half, z),
        ];
        [[0, 1, 2], [0, 2, 3]].map(|indices| Triangle {
            positions: indices.map(|index| corners[index]),
            uvs: [[0.5, 0.5]; 3],
            texture,
            material,
        })
    }

    #[test]
    fn an_empty_scene_is_fully_transparent() {
        let image = render(&Scene::default(), &options(8));
        assert!(image.pixels().iter().all(|pixel| *pixel == [0; 4]));
    }

    #[test]
    fn full_ambient_light_reproduces_the_texture_color() {
        let scene = Scene {
            textures: vec![solid([200, 100, 50, 255])],
            triangles: quad(0.0, 10.0, 0, Material::default()).to_vec(),
        };
        let image = render(&scene, &options(8));
        assert!(image.pixels().iter().all(|p| *p == [200, 100, 50, 255]));
    }

    #[test]
    fn nearer_opaque_triangles_hide_farther_ones_in_any_order() {
        let near = quad(1.0, 10.0, 0, Material::default());
        let far = quad(0.0, 10.0, 1, Material::default());
        for triangles in [[near, far].concat(), [far, near].concat()] {
            let scene = Scene {
                textures: vec![
                    solid([255, 0, 0, 255]),
                    solid([0, 0, 255, 255]),
                ],
                triangles,
            };
            assert_eq!(render(&scene, &options(4)).get(2, 2), [255, 0, 0, 255]);
        }
    }

    #[test]
    fn back_faces_are_culled_unless_double_sided() {
        let mut triangles = quad(0.0, 10.0, 0, Material::default());
        for triangle in &mut triangles {
            triangle.positions.swap(1, 2);
        }
        let mut scene = Scene {
            textures: vec![solid([9, 9, 9, 255])],
            triangles: triangles.to_vec(),
        };
        assert_eq!(render(&scene, &options(4)).get(2, 2), [0; 4]);

        for triangle in &mut scene.triangles {
            triangle.material.double_sided = true;
        }
        assert_eq!(render(&scene, &options(4)).get(2, 2), [9, 9, 9, 255]);
    }

    #[test]
    fn texels_below_the_alpha_cutoff_are_discarded() {
        let scene = Scene {
            textures: vec![solid([255, 255, 255, 20])],
            triangles: quad(0.0, 10.0, 0, Material::default()).to_vec(),
        };
        assert_eq!(render(&scene, &options(4)).get(2, 2), [0; 4]);
    }

    #[test]
    fn blended_triangles_mix_with_what_is_behind_them() {
        let blend = Material {
            blend: true,
            ..Material::default()
        };
        let scene = Scene {
            textures: vec![solid([255, 255, 255, 128]), solid([0, 0, 0, 255])],
            triangles: [
                quad(1.0, 10.0, 0, blend),
                quad(0.0, 10.0, 1, Material::default()),
            ]
            .concat(),
        };
        let pixel = render(&scene, &options(4)).get(2, 2);
        assert_eq!(pixel[3], 255);
        assert!((127..=129).contains(&pixel[0]), "{pixel:?}");
    }

    #[test]
    fn emissive_triangles_are_brighter_than_lit_ones() {
        let lit = Scene {
            textures: vec![solid([100, 100, 100, 255])],
            triangles: quad(0.0, 10.0, 0, Material::default()).to_vec(),
        };
        let mut glowing = lit.clone();
        for triangle in &mut glowing.triangles {
            triangle.material.emissive = true;
        }
        let dim = Lighting {
            ambient: 0.5,
            ..options(4).lighting
        };
        let render_with = |scene| {
            render(
                scene,
                &RenderOptions {
                    lighting: dim,
                    ..options(4)
                },
            )
            .get(2, 2)[0]
        };
        assert!(render_with(&lit) < 100);
        assert!(render_with(&glowing) > 100);
    }

    #[test]
    fn faces_turned_towards_the_light_are_brighter() {
        let scene = Scene {
            textures: vec![solid([200, 200, 200, 255])],
            triangles: quad(0.0, 10.0, 0, Material::default()).to_vec(),
        };
        let render_lit_from = |direction| {
            render(
                &scene,
                &RenderOptions {
                    lighting: Lighting {
                        ambient: 0.2,
                        directional: 0.6,
                        direction,
                    },
                    ..options(4)
                },
            )
            .get(2, 2)[0]
        };
        assert!(
            render_lit_from(Vec3::new(0.0, 0.0, 1.0))
                > render_lit_from(Vec3::new(0.0, 0.0, -1.0))
        );
    }

    #[test]
    fn supersampling_softens_edges() {
        let scene = Scene {
            textures: vec![solid([255, 255, 255, 255])],
            triangles: vec![Triangle {
                positions: [
                    Vec3::new(-1.0, -1.0, 0.0),
                    Vec3::new(1.0, -1.0, 0.0),
                    Vec3::new(-1.0, 1.0, 0.0),
                ],
                uvs: [[0.5, 0.5]; 3],
                texture: 0,
                material: Material::default(),
            }],
        };
        let partial = |supersampling| {
            render(
                &scene,
                &RenderOptions {
                    supersampling,
                    ..options(16)
                },
            )
            .pixels()
            .iter()
            .filter(|pixel| pixel[3] != 0 && pixel[3] != 255)
            .count()
        };
        assert_eq!(partial(1), 0);
        assert!(partial(3) > 0);
    }

    #[test]
    fn a_player_fills_the_middle_of_the_default_frame() {
        let skin = SkinTexture::new(
            Texture::from_pixels(64, 64, vec![[120, 80, 40, 255]; 64 * 64])
                .unwrap(),
        )
        .unwrap();
        let scene = Scene::build(
            &test_model(),
            &ModelConventions::default(),
            &Player::new(&skin),
        );
        let image = render(&scene, &RenderOptions::default());
        assert_eq!((image.width(), image.height()), (360, 504));
        assert_eq!(image.get(180, 252)[3], 255);
        assert_eq!(image.get(2, 2)[3], 0);
        assert_eq!(image.get(357, 501)[3], 0);
    }

    #[test]
    fn faces_show_the_hat_layer_over_the_head() {
        let mut texture = Texture::new(64, 64);
        for y in 8..16 {
            for x in 8..16 {
                texture.set(x, y, [10, 20, 30, 255]);
            }
        }
        texture.set(40, 8, [200, 0, 0, 255]);
        let face = render_face(&SkinTexture::new(texture).unwrap(), 16);

        assert_eq!((face.width(), face.height()), (16, 16));
        assert_eq!(face.get(0, 0), [200, 0, 0, 255]);
        assert_eq!(face.get(1, 1), [200, 0, 0, 255]);
        assert_eq!(face.get(2, 0), [10, 20, 30, 255]);
        assert_eq!(face.get(15, 15), [10, 20, 30, 255]);
    }
}
