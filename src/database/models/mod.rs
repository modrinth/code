mod mod_item;
mod version_item;

use crate::database::DatabaseError::NotFound;
use crate::database::Result;
use async_trait::async_trait;
use bson::doc;
use bson::Document;
pub use mod_item::Mod;
use mongodb::Database;
pub use version_item::Version;

#[async_trait]
pub trait Item {
    fn get_collection() -> &'static str;
    async fn get_by_id(client: Database, id: &str) -> Result<Box<Self>> {
        let filter = doc! { "_id": id };
        let collection = client.collection(Self::get_collection());
        let doc: Document = match collection.find_one(filter, None).await? {
            Some(e) => e,
            None => return Err(NotFound()),
        };
        let elem: Box<Self> = Self::from_doc(doc)?;
        Ok(elem)
    }
    fn from_doc(elem: Document) -> Result<Box<Self>>;
}
