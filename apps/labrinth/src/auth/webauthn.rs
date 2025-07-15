use actix_web::web::Data;
use regex::Regex;
use webauthn_rs::{Webauthn, WebauthnBuilder};

pub fn startup() -> Data<Webauthn> {
    let url = url::Url::parse(&dotenvy::var("SITE_URL").unwrap_or_default())
        .unwrap();
    
    let rp_id = dotenvy::var("SITE_URL")
        .ok()
        .and_then(|s| {
            Regex::new(r"^(?:http|https)://([^:/]+)")
                .ok()
                .and_then(|re| re.captures(&s))
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
        })
        .unwrap_or_default();

    let builder = WebauthnBuilder::new(&rp_id, &url)
        .expect("Invalid configuration");
    
    let builder = builder.rp_name("Actix-web modrinth");


    let webauthn = Data::new(builder.build()
        .expect("Invalid configuration"));

    webauthn
}