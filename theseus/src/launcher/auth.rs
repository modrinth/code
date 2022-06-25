//! Authentication flow
// TODO: Implement authentication
#[derive(Debug)]
pub struct Credentials {
    pub id: uuid::Uuid,
    pub username: String,
    pub access_token: String,
}
