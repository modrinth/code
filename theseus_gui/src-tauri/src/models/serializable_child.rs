use serde::{ser::SerializeStruct, Serialize, Serializer};
use tokio::process::Child;

#[derive(Debug)]
pub struct SerializableChild {
    #[allow(dead_code)]
    child: Child,
    pid: u32,
}

impl From<Child> for SerializableChild {
    fn from(child: Child) -> Self {
        let pid = child.id().unwrap_or(0);
        SerializableChild { child, pid }
    }
}

impl Serialize for SerializableChild {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("SerializableChild", 2)?;
        state.serialize_field("pid", &self.pid)?;
        state.end()
    }
}
