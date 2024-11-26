use std::collections::HashMap;

use super::v3::project_creation::CreateError;
use super::ApiError;
use crate::models::v2::projects::LegacySideType;
use crate::util::actix::{
    generate_multipart, MultipartSegment, MultipartSegmentData,
};
use actix_multipart::Multipart;
use actix_web::http::header::{
    ContentDisposition, HeaderMap, TryIntoHeaderPair,
};
use actix_web::HttpResponse;
use futures::{stream, Future, StreamExt};
use serde_json::{json, Value};

// 提取 OK 状态的 JSON 响应
pub async fn extract_ok_json<T>(
    response: HttpResponse,
) -> Result<T, HttpResponse>
where
    T: serde::de::DeserializeOwned,
{
    // 如果响应状态是 StatusCode::OK，解析 JSON 并返回
    if response.status() == actix_web::http::StatusCode::OK {
        let failure_http_response = || {
            HttpResponse::InternalServerError().json(json!({
                "error": "reroute_error",
                "description": "无法解析 V2 路由重定向的响应。"
            }))
        };
        // 从 HttpResponse 中提取 JSON，进行修改，然后重新生成 HttpResponse
        let body = response.into_body();
        let bytes = actix_web::body::to_bytes(body)
            .await
            .map_err(|_| failure_http_response())?;
        let json_value: T = serde_json::from_slice(&bytes)
            .map_err(|_| failure_http_response())?;
        Ok(json_value)
    } else {
        Err(response)
    }
}

// 仅移除 404 响应的主体
// 不应在回退无路由找到的处理程序中使用
pub fn flatten_404_error(res: ApiError) -> Result<HttpResponse, ApiError> {
    match res {
        ApiError::NotFound => Ok(HttpResponse::NotFound().body("")),
        _ => Err(res),
    }
}

// 允许内部修改 actix multipart 文件
// 预期：
// 1. 一个 JSON 段
// 2. 任意数量的其他二进制段
// 'closure' 使用 JSON 值和其他段的内容处置进行调用
pub async fn alter_actix_multipart<T, U, Fut>(
    mut multipart: Multipart,
    mut headers: HeaderMap,
    mut closure: impl FnMut(T, Vec<ContentDisposition>) -> Fut,
) -> Result<Multipart, CreateError>
where
    T: serde::de::DeserializeOwned,
    U: serde::Serialize,
    Fut: Future<Output = Result<U, CreateError>>,
{
    let mut segments: Vec<MultipartSegment> = Vec::new();

    let mut json = None;
    let mut json_segment = None;
    let mut content_dispositions = Vec::new();

    if let Some(field) = multipart.next().await {
        let mut field = field?;
        let content_disposition = field.content_disposition().clone();
        let field_name = content_disposition.get_name().unwrap_or("");
        let field_filename = content_disposition.get_filename();
        let field_content_type = field.content_type();
        let field_content_type = field_content_type.map(|ct| ct.to_string());

        let mut buffer = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            buffer.extend_from_slice(&data);
        }

        {
            let json_value: T = serde_json::from_slice(&buffer)?;
            json = Some(json_value);
        }

        json_segment = Some(MultipartSegment {
            name: field_name.to_string(),
            filename: field_filename.map(|s| s.to_string()),
            content_type: field_content_type,
            data: MultipartSegmentData::Binary(vec![]), // 初始化为空，将在之后完成
        });
    }

    while let Some(field) = multipart.next().await {
        let mut field = field?;
        let content_disposition = field.content_disposition().clone();
        let field_name = content_disposition.get_name().unwrap_or("");
        let field_filename = content_disposition.get_filename();
        let field_content_type = field.content_type();
        let field_content_type = field_content_type.map(|ct| ct.to_string());

        let mut buffer = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            buffer.extend_from_slice(&data);
        }

        content_dispositions.push(content_disposition.clone());
        segments.push(MultipartSegment {
            name: field_name.to_string(),
            filename: field_filename.map(|s| s.to_string()),
            content_type: field_content_type,
            data: MultipartSegmentData::Binary(buffer),
        })
    }

    // 完成 JSON 段，带有聚合的内容处置
    {
        let json_value = json.ok_or(CreateError::InvalidInput(
            "在 multipart 中未找到 JSON 段。".to_string(),
        ))?;
        let mut json_segment =
            json_segment.ok_or(CreateError::InvalidInput(
                "在 multipart 中未找到 JSON 段。".to_string(),
            ))?;

        // 使用 JSON 值和其他段的名称调用 closure
        let json_value: U = closure(json_value, content_dispositions).await?;
        let buffer = serde_json::to_vec(&json_value)?;
        json_segment.data = MultipartSegmentData::Binary(buffer);

        // 将 JSON 段插入到开头
        segments.insert(0, json_segment);
    }

    let (boundary, payload) = generate_multipart(segments);

    match (
        "Content-Type",
        format!("multipart/form-data; boundary={}", boundary).as_str(),
    )
        .try_into_pair()
    {
        Ok((key, value)) => {
            headers.insert(key, value);
        }
        Err(err) => {
            CreateError::InvalidInput(format!(
                "插入测试头时出错： {:?}。",
                err
            ));
        }
    };

    let new_multipart =
        Multipart::new(&headers, stream::once(async { Ok(payload) }));

    Ok(new_multipart)
}

// 将 "client_side" 和 "server_side" 对转换为新的 v3 对应字段
pub fn convert_side_types_v3(
    client_side: LegacySideType,
    server_side: LegacySideType,
) -> HashMap<String, Value> {
    use LegacySideType::{Optional, Required};

    let singleplayer = client_side == Required
        || client_side == Optional
        || server_side == Required
        || server_side == Optional;
    let client_and_server = singleplayer;
    let client_only = (client_side == Required || client_side == Optional)
        && server_side != Required;
    let server_only = (server_side == Required || server_side == Optional)
        && client_side != Required;

    let mut fields = HashMap::new();
    fields.insert("singleplayer".to_string(), json!(singleplayer));
    fields.insert("client_and_server".to_string(), json!(client_and_server));
    fields.insert("client_only".to_string(), json!(client_only));
    fields.insert("server_only".to_string(), json!(server_only));
    fields
}

// 将插件加载器从 v2 转换为 v3，用于搜索 facets
// 在每个一级和二级（v2 中允许的）中，我们将每个实例转换为：
// "project_type:mod" 到 "project_type:plugin" 或 "project_type:mod"
pub fn convert_plugin_loader_facets_v3(
    facets: Vec<Vec<String>>,
) -> Vec<Vec<String>> {
    facets
        .into_iter()
        .map(|inner_facets| {
            if inner_facets == ["project_type:mod"] {
                vec![
                    "project_type:plugin".to_string(),
                    "project_type:datapack".to_string(),
                    "project_type:mod".to_string(),
                ]
            } else {
                inner_facets
            }
        })
        .collect::<Vec<_>>()
}

// 将搜索 facets 从 V3 转换回 v2
// 这不是无损的。（见测试）
pub fn convert_side_types_v2(
    side_types: &HashMap<String, Value>,
    project_type: Option<&str>,
) -> (LegacySideType, LegacySideType) {
    let client_and_server = side_types
        .get("client_and_server")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let singleplayer = side_types
        .get("singleplayer")
        .and_then(|x| x.as_bool())
        .unwrap_or(client_and_server);
    let client_only = side_types
        .get("client_only")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let server_only = side_types
        .get("server_only")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);

    convert_side_types_v2_bools(
        Some(singleplayer),
        client_only,
        server_only,
        Some(client_and_server),
        project_type,
    )
}

// 客户端，服务器端
pub fn convert_side_types_v2_bools(
    singleplayer: Option<bool>,
    client_only: bool,
    server_only: bool,
    client_and_server: Option<bool>,
    project_type: Option<&str>,
) -> (LegacySideType, LegacySideType) {
    use LegacySideType::{Optional, Required, Unknown, Unsupported};

    match project_type {
        Some("plugin") => (Unsupported, Required),
        Some("datapack") => (Optional, Required),
        Some("shader") => (Required, Unsupported),
        Some("resourcepack") => (Required, Unsupported),
        _ => {
            let singleplayer =
                singleplayer.or(client_and_server).unwrap_or(false);

            match (singleplayer, client_only, server_only) {
                // 仅单人游戏
                (true, false, false) => (Required, Required),

                // 仅客户端且不为服务器端
                (false, true, false) => (Required, Unsupported),
                (true, true, false) => (Required, Unsupported),

                // 仅服务器端且不为客户端
                (false, false, true) => (Unsupported, Required),
                (true, false, true) => (Unsupported, Required),

                // 同时为服务器端和客户端
                (true, true, true) => (Optional, Optional),
                (false, true, true) => (Optional, Optional),

                // 错误类型
                (false, false, false) => (Unknown, Unknown),
            }
        }
    }
}

// 首字母大写
pub fn capitalize_first(input: &str) -> String {
    let mut result = input.to_owned();
    if let Some(first_char) = result.get_mut(0..1) {
        first_char.make_ascii_uppercase();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::v2::projects::LegacySideType::{
        Optional, Required, Unsupported,
    };

    #[test]
    fn convert_types() {
        // 从 V2 到 V3 再转换回来应该是幂等的 - 对于某些对
        let lossy_pairs = [
            (Optional, Unsupported),
            (Unsupported, Optional),
            (Required, Optional),
            (Optional, Required),
            (Unsupported, Unsupported),
        ];

        for client_side in [Required, Optional, Unsupported] {
            for server_side in [Required, Optional, Unsupported] {
                if lossy_pairs.contains(&(client_side, server_side)) {
                    continue;
                }
                let side_types =
                    convert_side_types_v3(client_side, server_side);
                let (client_side2, server_side2) =
                    convert_side_types_v2(&side_types, None);
                assert_eq!(client_side, client_side2);
                assert_eq!(server_side, server_side2);
            }
        }
    }
}