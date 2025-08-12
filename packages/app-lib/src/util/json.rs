use crate::ErrorKind::InputError;
use crate::Result;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt};

pub async fn parse_object_async_reader<R, T>(reader: &mut R) -> Result<T>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    let first_char = reader.read_u8().await?;
    if first_char != b'{' {
        return Err(InputError(format!(
            "Expected '{{' to start JSON object, but found '{}'",
            first_char as char
        ))
        .into());
    }

    let mut json_data = vec![first_char];
    let mut depth = 1usize;
    while depth > 0 {
        let char = reader.read_u8().await?;
        json_data.push(char);
        match char {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'"' => loop {
                let char = reader.read_u8().await?;
                json_data.push(char);
                match char {
                    b'\\' => json_data.push(reader.read_u8().await?),
                    b'"' => break,
                    _ => {}
                }
            },
            _ => {}
        }
    }

    Ok(serde_json::from_slice(&json_data)?)
}
