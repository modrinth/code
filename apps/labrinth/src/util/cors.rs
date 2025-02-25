use actix_cors::Cors;

// Updating this? Remember to update the ratelimit CORS too!
pub fn default_cors() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allow_any_header()
        .allow_any_method()
        .max_age(3600)
        .send_wildcard()
}
