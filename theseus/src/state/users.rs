//! User login info
use crate::{auth::Credentials, config::BINCODE_CONFIG};

const USER_DB_TREE: &[u8] = b"users";

/// The set of users stored in the launcher
#[derive(Debug, Clone)]
pub(crate) struct Users(pub(crate) sled::Tree);

impl Users {
    pub fn init(db: &sled::Db) -> crate::Result<Self> {
        Ok(Self(db.open_tree(USER_DB_TREE)?))
    }

    pub fn insert(
        &mut self,
        credentials: &Credentials,
    ) -> crate::Result<&Self> {
        let id = credentials.id.as_bytes();
        self.0.insert(
            id,
            bincode::encode_to_vec(credentials, *BINCODE_CONFIG)?,
        )?;
        Ok(self)
    }

    pub fn get(&self, id: uuid::Uuid) -> crate::Result<Option<Credentials>> {
        self.0.get(id.as_bytes())?.map_or(Ok(None), |prof| {
            bincode::decode_from_slice(&prof, *BINCODE_CONFIG)
                .map_err(crate::Error::from)
                .map(|it| Some(it.0))
        })
    }

    pub fn remove(&mut self, id: uuid::Uuid) -> crate::Result<&Self> {
        self.0.remove(id.as_bytes())?;
        Ok(self)
    }

    pub fn iter(&self) -> UserIter<impl UserInnerIter> {
        UserIter(self.0.iter().keys(), false)
    }
}

alias_trait! {pub UserInnerIter: Iterator<Item = sled::Result<sled::IVec>>, Send, Sync}

/// An iterator over the set of users
#[derive(Debug)]
pub struct UserIter<I: UserInnerIter>(I, bool);

impl<I: UserInnerIter> Iterator for UserIter<I> {
    type Item = crate::Result<Credentials>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 {
            return None;
        }

        let it = self.0.next()?;
        let res = it.map_err(crate::Error::from).and_then(|it| {
            Ok(bincode::decode_from_slice(&it, *BINCODE_CONFIG)?.0)
        });

        self.1 = res.is_err();
        Some(res)
    }
}
