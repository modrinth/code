pub fn get_image_content_type(extension: &str) -> Option<&'static str> {
    let content_type = match &*extension {
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        "jpeg" | "jpg" | "jpe" => "image/jpeg",
        "png" => "image/png",
        "svg" | "svgz" => "image/svg+xml",
        "webp" => "image/webp",
        "rgb" => "image/x-rgb",
        "mp4" => "video/mp4",
        _ => "",
    };

    if !content_type.is_empty() {
        Some(content_type)
    } else {
        None
    }
}

pub fn project_file_type(ext: &str) -> Option<&str> {
    match ext {
        "jar" => Some("application/java-archive"),
        "zip" => Some("application/zip"),
        _ => None,
    }
}
