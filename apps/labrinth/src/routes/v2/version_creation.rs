use crate::database::models::loader_fields::VersionField;
use crate::database::models::{project_item, version_item};
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::ids::ImageId;
use crate::models::projects::{
    Dependency, FileType, Loader, ProjectId, Version, VersionId, VersionStatus,
    VersionType,
};
use crate::models::v2::projects::LegacyVersion;
use crate::queue::moderation::AutomatedModerationQueue;
use crate::queue::session::AuthQueue;
use crate::routes::v3::project_creation::CreateError;
use crate::routes::v3::version_creation;
use crate::routes::{v2_reroute, v3};
use actix_multipart::Multipart;
use actix_web::http::header::ContentDisposition;
use actix_web::web::Data;
use actix_web::{post, web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use validator::Validate;

pub fn default_requested_status() -> VersionStatus {
    VersionStatus::Listed // 默认请求状态为“已列出”
}

#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct InitialVersionData {
    #[serde(alias = "mod_id")]
    pub project_id: Option<ProjectId>, // 项目ID
    #[validate(length(min = 1, max = 256))]
    pub file_parts: Vec<String>, // 文件部分
    #[validate(
        length(min = 1, max = 32),
        regex = "crate::util::validate::RE_URL_SAFE"
    )]
    pub version_number: String, // 版本号
    #[validate(
        length(min = 1, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    #[serde(alias = "name")]
    pub version_title: String, // 版本标题
    #[validate(length(max = 65536))]
    #[serde(alias = "changelog")]
    pub version_body: Option<String>, // 版本日志
    #[validate(
        length(min = 0, max = 4096),
        custom(function = "crate::util::validate::validate_deps")
    )]
    pub dependencies: Vec<Dependency>, // 依赖项
    #[validate(length(min = 1))]
    pub game_versions: Vec<String>, // 游戏版本
    #[serde(alias = "version_type")]
    pub release_channel: VersionType, // 发布渠道
    #[validate(length(min = 1))]
    pub loaders: Vec<Loader>, // 加载器
    pub featured: bool,               // 是否推荐
    pub primary_file: Option<String>, // 主文件
    #[serde(default = "default_requested_status")]
    pub status: VersionStatus, // 状态
    #[serde(default = "HashMap::new")]
    pub file_types: HashMap<String, Option<FileType>>, // 文件类型
    #[validate(length(max = 10))]
    #[serde(default)]
    pub uploaded_images: Vec<ImageId>, // 上传的图片
    pub ordering: Option<i32>,        // 排序
    pub curse: bool
}

#[derive(Serialize, Deserialize, Clone)]
struct InitialFileData {
    #[serde(default = "HashMap::new")]
    pub file_types: HashMap<String, Option<FileType>>, // 文件类型
}

// 在 `/api/v1/version` 下
#[post("version")]
pub async fn version_create(
    req: HttpRequest,
    payload: Multipart,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    file_host: Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: Data<AuthQueue>,
    moderation_queue: Data<AutomatedModerationQueue>,
) -> Result<HttpResponse, CreateError> {
    let payload = v2_reroute::alter_actix_multipart(
        payload,
        req.headers().clone(),
        |legacy_create: InitialVersionData,
         content_dispositions: Vec<ContentDisposition>| {
            let client = client.clone();
            let redis = redis.clone();
            async move {
                // 将输入数据转换为 V3 格式

                let mut fields = HashMap::new();
                fields.insert(
                    "game_versions".to_string(),
                    json!(legacy_create.game_versions),
                );

                // 获取给定加载器的所有可能的侧类型字段 - 我们将使用这些字段来检查是否需要转换/应用单人游戏等。
                let loaders =
                    match v3::tags::loader_list(client.clone(), redis.clone())
                        .await
                    {
                        Ok(loader_response) => {
                            (v2_reroute::extract_ok_json::<
                                Vec<v3::tags::LoaderData>,
                            >(loader_response)
                            .await)
                                .unwrap_or_default()
                        }
                        Err(_) => vec![],
                    };

                let loader_fields_aggregate = loaders
                    .into_iter()
                    .filter_map(|loader| {
                        if legacy_create
                            .loaders
                            .contains(&Loader(loader.name.clone()))
                        {
                            Some(loader.supported_fields)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                // 复制项目的示例版本的侧类型。
                // 如果没有版本存在，则默认为所有 false。
                // 这本质上是有损的，但对此无能为力，因为侧类型不再与项目关联，
                // 因此无法轻松访问“缺失”的那些，并且版本确实需要显式设置这些字段。
                let side_type_loader_field_names = [
                    "singleplayer",      // 单人游戏
                    "client_and_server", // 客户端和服务器
                    "client_only",       // 仅客户端
                    "server_only",       // 仅服务器
                ];

                // 检查 loader_fields_aggregate 是否包含这些侧类型中的任何一个
                // 我们假设这四个字段是关联在一起的。
                if loader_fields_aggregate
                    .iter()
                    .any(|f| side_type_loader_field_names.contains(&f.as_str()))
                {
                    // 如果是这样，我们获取项目的示例版本的字段，并设置侧类型以匹配。
                    fields.extend(
                        side_type_loader_field_names
                            .iter()
                            .map(|f| (f.to_string(), json!(false))),
                    );
                    if let Some(example_version_fields) =
                        get_example_version_fields(
                            legacy_create.project_id,
                            client,
                            &redis,
                        )
                        .await?
                    {
                        fields.extend(
                            example_version_fields.into_iter().filter_map(
                                |f| {
                                    if side_type_loader_field_names
                                        .contains(&f.field_name.as_str())
                                    {
                                        Some((
                                            f.field_name,
                                            f.value.serialize_internal(),
                                        ))
                                    } else {
                                        None
                                    }
                                },
                            ),
                        );
                    }
                }
                // 通过文件扩展名预测处理项目类型
                let mut project_type = None;
                for file_part in &legacy_create.file_parts {
                    if let Some(ext) = file_part.split('.').last() {
                        match ext {
                            "mrpack" | "mrpack-primary" => {
                                project_type = Some("modpack");
                                break;
                            }
                            // 其他类型不重要
                            _ => {}
                        }
                        break;
                    }
                }

                // 类似地，检查实际内容处置的 mrpacks，以防 file_parts 错误
                for content_disposition in content_dispositions {
                    // 使用 version_create 函数获取文件名和扩展名
                    let (_, file_extension) =
                        version_creation::get_name_ext(&content_disposition)?;
                    crate::util::ext::project_file_type(file_extension)
                        .ok_or_else(|| {
                            CreateError::InvalidFileType(
                                file_extension.to_string(),
                            )
                        })?;

                    if file_extension == "mrpack" {
                        project_type = Some("modpack");
                        break;
                    }
                }
                
                if legacy_create.curse == true {
                    project_type = Some("modpack");
                }

                // Modpacks 现在使用“mrpack”加载器，并且加载器被转换为加载器字段。
                // 直接设置“project_type”已被删除，现在是基于加载器的。
                if project_type == Some("modpack") {
                    fields.insert(
                        "mrpack_loaders".to_string(),
                        json!(legacy_create.loaders),
                    );
                }

                let loaders = if project_type == Some("modpack") {
                    vec![Loader("mrpack".to_string())]
                } else {
                    legacy_create.loaders
                };

                Ok(v3::version_creation::InitialVersionData {
                    project_id: legacy_create.project_id,
                    file_parts: legacy_create.file_parts,
                    version_number: legacy_create.version_number,
                    version_title: legacy_create.version_title,
                    version_body: legacy_create.version_body,
                    dependencies: legacy_create.dependencies,
                    release_channel: legacy_create.release_channel,
                    loaders,
                    featured: legacy_create.featured,
                    primary_file: legacy_create.primary_file,
                    status: legacy_create.status,
                    file_types: legacy_create.file_types,
                    uploaded_images: legacy_create.uploaded_images,
                    ordering: legacy_create.ordering,
                    fields,
                })
            }
        },
    )
    .await?;


    // let mut error = None;
    //
    // while let Some(item) = payload.next().await {
    //     let mut field: Field = item?;
    //     let result = async {
    //         let content_disposition = field.content_disposition().clone();
    //         let name = content_disposition.get_name().ok_or_else(|| {
    //             CreateError::MissingValueError("Missing content name".to_string())
    //         })?;
    //         println!("{}", name);
    //
    //         if name == "data" {
    //             let mut data = Vec::new();
    //             while let Some(chunk) = field.next().await {
    //                 data.extend_from_slice(&chunk?);
    //             }
    //             let file_data: InitialFileData = serde_json::from_slice(&data)?;
    //             // 将解析后的数据转换为 JSON 字符串
    //             let json_output = serde_json::to_string(&file_data)?;
    //             // 如果需要更具可读性的格式，可以使用 serde_json::to_string_pretty
    //             // let json_output = serde_json::to_string_pretty(&file_data)?;
    //
    //             println!("{}", json_output);  // 输出 JSON 字符串到控制台
    //
    //             return Ok(());
    //         }
    //         // let data = read_from_field(
    //         //     &mut field, 500 * (1 << 20),
    //         //     "项目文件超出了 500MB 的上限。请联系版主或管理员以请求上传更大文件的权限。"
    //         // ).await?;
    //         //
    //         // info!("Reading field 4");
    //
    //
    //
    //
    //         Ok(())
    //     }
    //         .await;
    //     if result.is_err() {
    //         error = result.err();
    //     }
    // }
    //
    // if let Some(error) = error {
    //     return Err(error);
    // }

    // 调用 V3 项目创建
    let response = v3::version_creation::version_create(
        req,
        payload,
        client.clone(),
        redis.clone(),
        file_host,
        session_queue,
        moderation_queue,
    )
    .await?;

    // 将响应转换为 V2 格式
    match v2_reroute::extract_ok_json::<Version>(response).await {
        Ok(version) => {
            let v2_version = LegacyVersion::from(version);
            Ok(HttpResponse::Ok().json(v2_version))
        }
        Err(response) => Ok(response),
    }
}

// 获取项目的示例版本的版本字段（如果存在）。
async fn get_example_version_fields(
    project_id: Option<ProjectId>,
    pool: Data<PgPool>,
    redis: &RedisPool,
) -> Result<Option<Vec<VersionField>>, CreateError> {
    let project_id = match project_id {
        Some(project_id) => project_id,
        None => return Ok(None),
    };

    let vid =
        match project_item::Project::get_id(project_id.into(), &**pool, redis)
            .await?
            .and_then(|p| p.versions.first().cloned())
        {
            Some(vid) => vid,
            None => return Ok(None),
        };

    let example_version =
        match version_item::Version::get(vid, &**pool, redis).await? {
            Some(version) => version,
            None => return Ok(None),
        };
    Ok(Some(example_version.version_fields))
}

// 在 /api/v1/version/{version_id} 下
#[post("{version_id}/file")]
pub async fn upload_file_to_version(
    req: HttpRequest,
    url_data: web::Path<(VersionId,)>,
    payload: Multipart,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    file_host: Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, CreateError> {
    // 返回 NoContent，因此无需转换为 V2
    let response = v3::version_creation::upload_file_to_version(
        req,
        url_data,
        payload,
        client.clone(),
        redis.clone(),
        file_host,
        session_queue,
    )
    .await?;
    Ok(response)
}
