pub fn get_image_content_type(extension: &str) -> Option<&'static str> {
    match extension {
        "bmp" => Some("image/bmp"),
        "gif" => Some("image/gif"),
        "jpeg" | "jpg" | "jpe" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "svg" | "svgz" => Some("image/svg+xml"),
        "webp" => Some("image/webp"),
        "rgb" => Some("image/x-rgb"),
        "mp4" => Some("video/mp4"),
        _ => None,
    }
}

pub fn project_file_type(ext: &str) -> Option<&str> {
    match ext {
        "jar" => Some("application/java-archive"),
        "zip" => Some("application/zip"),
        "mrpack" => Some("application/x-modrinth-modpack+zip"),
        _ => None,
    }
}
