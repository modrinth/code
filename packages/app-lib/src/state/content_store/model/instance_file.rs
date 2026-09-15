#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InstanceFileStatus {
    Healthy,
    Missing,
    Conflict,
}

pub(in crate::state::content_store) enum InstancePathContent {
    Missing,
    File(String),
    Conflict,
}

impl InstancePathContent {
    pub(in crate::state::content_store) fn status(
        &self,
        sha512: &str,
    ) -> InstanceFileStatus {
        match self {
            Self::Missing => InstanceFileStatus::Missing,
            Self::File(hash) if hash == sha512 => InstanceFileStatus::Healthy,
            _ => InstanceFileStatus::Conflict,
        }
    }
}
