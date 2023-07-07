pub fn get_image_content_type(extension: &str) -> Option<&'static str> {
    match extension {
        "bmp" => Some("image/bmp"),
        "gif" => Some("image/gif"),
        "jpeg" | "jpg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

pub fn get_image_ext(content_type: &str) -> Option<&'static str> {
    match content_type {
        "image/bmp" => Some("bmp"),
        "image/gif" => Some("gif"),
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        _ => None,
    }
}

pub fn project_file_type(ext: &str) -> Option<&str> {
    match ext {
        "jar" => Some("application/java-archive"),
        "zip" | "litemod" => Some("application/zip"),
        "mrpack" => Some("application/x-modrinth-modpack+zip"),
        _ => None,
    }
}
