//! Renders a skin onto a glTF player model.
//!
//! ```sh
//! cargo run -p modrinth-skin-rendering --example render -- model.gltf skin.png preview.png
//! ```

use modrinth_skin_rendering::{
    Camera, Model, RenderOptions, Scene, SkinTexture, Texture, Vec3, render,
    render_face,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [model, skin, output, rest @ ..] = &args[..] else {
        eprintln!(
            "usage: render <model.gltf> <skin.png> <output.png> [cape.png]"
        );
        std::process::exit(2);
    };

    let model = Model::from_gltf(&std::fs::read(model)?)?;
    let skin = SkinTexture::from_png(&std::fs::read(skin)?)?;
    let cape = match rest.first() {
        Some(path) => Some(Texture::from_png(&std::fs::read(path)?)?),
        None => None,
    };

    let scene = Scene::build(&model, &skin, cape.as_ref());
    let camera = Camera {
        position: Vec3::new(-1.6, 1.2, 5.4),
        target: Vec3::new(0.0, 1.0, 0.0),
        up: Vec3::Y,
        fov_y_degrees: 24.0,
        near: 0.1,
        far: 100.0,
    };
    let image = render(&scene, &camera, &RenderOptions::default());
    std::fs::write(output, image.to_png()?)?;

    let face_path = format!("{}.face.png", output.trim_end_matches(".png"));
    std::fs::write(face_path, render_face(&skin, 64).to_png()?)?;
    Ok(())
}
