#![allow(dead_code)]

use chrono::Utc;
use labrinth::{
    database::{self, models::generate_pat_id},
    models::pats::Scopes,
};

use super::database::TemporaryDatabase;

// Creates a PAT with the given scopes, and returns the access token
// Interfacing with the db directly, rather than using a ourte,
//  allows us to test with scopes that are not allowed to be created by PATs
pub async fn create_test_pat(
    scopes: Scopes,
    user_id: i64,
    db: &TemporaryDatabase,
) -> String {
    let mut transaction = db.pool.begin().await.unwrap();
    let id = generate_pat_id(&mut transaction).await.unwrap();
    let pat = database::models::pat_item::DBPersonalAccessToken {
        id,
        name: format!("test_pat_{}", scopes.bits()),
        access_token: format!("mrp_{}", id.0),
        scopes,
        user_id: database::models::ids::DBUserId(user_id),
        created: Utc::now(),
        expires: Utc::now() + chrono::Duration::days(1),
        last_used: None,
    };
    pat.insert(&mut transaction).await.unwrap();
    transaction.commit().await.unwrap();
    pat.access_token
}
