use std::collections::HashMap;

use super::ApiError;
use crate::auth::checks::{
    filter_visible_versions, is_visible_project, is_visible_version,
};
use crate::auth::get_user_from_headers;
use crate::database;
use crate::database::models::loader_fields::{
    self, LoaderField, LoaderFieldEnumValue, VersionField,
};
use crate::database::models::version_item::{
    DBLoaderVersion, DependencyBuilder,
};
use crate::database::models::{DBOrganization, image_item};
use crate::database::redis::RedisPool;
use crate::models;
use crate::models::ids::VersionId;
use crate::models::images::ImageContext;
use crate::models::pats::Scopes;
use crate::models::projects::{
    Dependency, FileType, VersionStatus, VersionType,
};
use crate::models::projects::{Loader, skip_nulls};
use crate::models::teams::ProjectPermissions;
use crate::queue::session::AuthQueue;
use crate::search::SearchConfig;
use crate::search::indexing::remove_documents;
use crate::util::img;
use crate::util::validate::validation_errors_to_string;
use actix_web::{HttpRequest, HttpResponse, web};
use ariadne::ids::base62_impl::parse_base62;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "version",
        web::post().to(super::version_creation::version_create),
    );
    cfg.route("versions", web::get().to(versions_get));

    cfg.service(
        web::scope("version")
            .route("{id}", web::get().to(version_get))
            .route("{id}", web::patch().to(version_edit))
            .route("{id}", web::delete().to(version_delete))
            .route(
                "{version_id}/file",
                web::post().to(super::version_creation::upload_file_to_version),
            ),
    );
}

// Given a project ID/slug and a version slug
pub async fn version_project_get(
    req: HttpRequest,
    info: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let info = info.into_inner();
    version_project_get_helper(req, info, pool, redis, session_queue).await
}
pub async fn version_project_get_helper(
    req: HttpRequest,
    id: (String, String),
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let result =
        database::models::DBProject::get(&id.0, &**pool, &redis).await?;

    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ, Scopes::VERSION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    if let Some(project) = result {
        if !is_visible_project(&project.inner, &user_option, &pool, false)
            .await?
        {
            return Err(ApiError::NotFound);
        }

        let versions = database::models::DBVersion::get_many(
            &project.versions,
            &**pool,
            &redis,
        )
        .await?;

        let id_opt = parse_base62(&id.1).ok();
        let version = versions.into_iter().find(|x| {
            Some(x.inner.id.0 as u64) == id_opt
                || x.inner.version_number == id.1
        });

        if let Some(version) = version {
            if is_visible_version(&version.inner, &user_option, &pool, &redis)
                .await?
            {
                return Ok(HttpResponse::Ok()
                    .json(models::projects::Version::from(version)));
            }
        }
    }

    Err(ApiError::NotFound)
}

#[derive(Serialize, Deserialize)]
pub struct VersionIds {
    pub ids: String,
}

pub async fn versions_get(
    req: HttpRequest,
    web::Query(ids): web::Query<VersionIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let version_ids =
        serde_json::from_str::<Vec<models::ids::VersionId>>(&ids.ids)?
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<database::models::DBVersionId>>();
    let versions_data =
        database::models::DBVersion::get_many(&version_ids, &**pool, &redis)
            .await?;

    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::VERSION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    let versions =
        filter_visible_versions(versions_data, &user_option, &pool, &redis)
            .await?;

    Ok(HttpResponse::Ok().json(versions))
}

pub async fn version_get(
    req: HttpRequest,
    info: web::Path<(models::ids::VersionId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    version_get_helper(req, id, pool, redis, session_queue).await
}

pub async fn version_get_helper(
    req: HttpRequest,
    id: models::ids::VersionId,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let version_data =
        database::models::DBVersion::get(id.into(), &**pool, &redis).await?;

    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::VERSION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    if let Some(data) = version_data {
        if is_visible_version(&data.inner, &user_option, &pool, &redis).await? {
            return Ok(
                HttpResponse::Ok().json(models::projects::Version::from(data))
            );
        }
    }

    Err(ApiError::NotFound)
}

#[derive(Serialize, Deserialize, Validate, Default, Debug)]
pub struct EditVersion {
    #[validate(
        length(min = 1, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    pub name: Option<String>,
    #[validate(
        length(min = 1, max = 32),
        regex(path = *crate::util::validate::RE_URL_SAFE)
    )]
    pub version_number: Option<String>,
    #[validate(length(max = 65536))]
    pub changelog: Option<String>,
    pub version_type: Option<models::projects::VersionType>,
    #[validate(
        length(min = 0, max = 4096),
        custom(function = "crate::util::validate::validate_deps")
    )]
    pub dependencies: Option<Vec<Dependency>>,
    pub loaders: Option<Vec<Loader>>,
    pub featured: Option<bool>,
    pub downloads: Option<u32>,
    pub status: Option<VersionStatus>,
    pub file_types: Option<Vec<EditVersionFileType>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub ordering: Option<Option<i32>>,

    // Flattened loader fields
    // All other fields are loader-specific VersionFields
    // These are flattened during serialization
    #[serde(deserialize_with = "skip_nulls")]
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditVersionFileType {
    pub algorithm: String,
    pub hash: String,
    pub file_type: Option<FileType>,
}

pub async fn version_edit(
    req: HttpRequest,
    info: web::Path<(VersionId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    new_version: web::Json<serde_json::Value>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let new_version: EditVersion =
        serde_json::from_value(new_version.into_inner())?;
    version_edit_helper(
        req,
        info.into_inner(),
        pool,
        redis,
        new_version,
        session_queue,
    )
    .await
}
pub async fn version_edit_helper(
    req: HttpRequest,
    info: (VersionId,),
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    new_version: EditVersion,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::VERSION_WRITE]),
    )
    .await?
    .1;

    new_version.validate().map_err(|err| {
        ApiError::Validation(validation_errors_to_string(err, None))
    })?;

    let version_id = info.0.into();

    let result =
        database::models::DBVersion::get(version_id, &**pool, &redis).await?;

    if let Some(version_item) = result {
        let team_member =
            database::models::DBTeamMember::get_from_user_id_project(
                version_item.inner.project_id,
                user.id.into(),
                false,
                &**pool,
            )
            .await?;

        let organization =
            DBOrganization::get_associated_organization_project_id(
                version_item.inner.project_id,
                &**pool,
            )
            .await?;

        let organization_team_member = if let Some(organization) = &organization
        {
            database::models::DBTeamMember::get_from_user_id(
                organization.team_id,
                user.id.into(),
                &**pool,
            )
            .await?
        } else {
            None
        };

        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        );

        if let Some(perms) = permissions {
            if !perms.contains(ProjectPermissions::UPLOAD_VERSION) {
                return Err(ApiError::CustomAuthentication(
                    "You do not have the permissions to edit this version!"
                        .to_string(),
                ));
            }

            let mut transaction = pool.begin().await?;

            if let Some(name) = &new_version.name {
                sqlx::query!(
                    "
                    UPDATE versions
                    SET name = $1
                    WHERE (id = $2)
                    ",
                    name.trim(),
                    version_id as database::models::ids::DBVersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(number) = &new_version.version_number {
                sqlx::query!(
                    "
                    UPDATE versions
                    SET version_number = $1
                    WHERE (id = $2)
                    ",
                    number,
                    version_id as database::models::ids::DBVersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(version_type) = &new_version.version_type {
                sqlx::query!(
                    "
                    UPDATE versions
                    SET version_type = $1
                    WHERE (id = $2)
                    ",
                    version_type.as_str(),
                    version_id as database::models::ids::DBVersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(dependencies) = &new_version.dependencies {
                sqlx::query!(
                    "
                    DELETE FROM dependencies WHERE dependent_id = $1
                    ",
                    version_id as database::models::ids::DBVersionId,
                )
                .execute(&mut *transaction)
                .await?;

                let builders = dependencies
                    .iter()
                    .map(|x| database::models::version_item::DependencyBuilder {
                        project_id: x.project_id.map(|x| x.into()),
                        version_id: x.version_id.map(|x| x.into()),
                        file_name: x.file_name.clone(),
                        dependency_type: x.dependency_type.to_string(),
                    })
                    .collect::<Vec<database::models::version_item::DependencyBuilder>>();

                DependencyBuilder::insert_many(
                    builders,
                    version_item.inner.id,
                    &mut transaction,
                )
                .await?;
            }

            if !new_version.fields.is_empty() {
                let version_fields_names = new_version
                    .fields
                    .keys()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>();

                let all_loaders =
                    loader_fields::Loader::list(&mut *transaction, &redis)
                        .await?;
                let loader_ids = version_item
                    .loaders
                    .iter()
                    .filter_map(|x| {
                        all_loaders
                            .iter()
                            .find(|y| &y.loader == x)
                            .map(|y| y.id)
                    })
                    .collect_vec();

                let loader_fields = LoaderField::get_fields(
                    &loader_ids,
                    &mut *transaction,
                    &redis,
                )
                .await?
                .into_iter()
                .filter(|lf| version_fields_names.contains(&lf.field))
                .collect::<Vec<LoaderField>>();

                let loader_field_ids = loader_fields
                    .iter()
                    .map(|lf| lf.id.0)
                    .collect::<Vec<i32>>();
                sqlx::query!(
                    "
                    DELETE FROM version_fields
                    WHERE version_id = $1
                    AND field_id = ANY($2)
                    ",
                    version_id as database::models::ids::DBVersionId,
                    &loader_field_ids
                )
                .execute(&mut *transaction)
                .await?;

                let mut loader_field_enum_values =
                    LoaderFieldEnumValue::list_many_loader_fields(
                        &loader_fields,
                        &mut *transaction,
                        &redis,
                    )
                    .await?;

                let mut version_fields = Vec::new();
                for (vf_name, vf_value) in new_version.fields {
                    let loader_field = loader_fields
                        .iter()
                        .find(|lf| lf.field == vf_name)
                        .ok_or_else(|| {
                            ApiError::InvalidInput(format!(
                                "Loader field '{vf_name}' does not exist for any loaders supplied."
                            ))
                        })?;
                    let enum_variants = loader_field_enum_values
                        .remove(&loader_field.id)
                        .unwrap_or_default();
                    let vf: VersionField = VersionField::check_parse(
                        version_id,
                        loader_field.clone(),
                        vf_value.clone(),
                        enum_variants,
                    )
                    .map_err(ApiError::InvalidInput)?;
                    version_fields.push(vf);
                }
                VersionField::insert_many(version_fields, &mut transaction)
                    .await?;
            }

            if let Some(loaders) = &new_version.loaders {
                sqlx::query!(
                    "
                    DELETE FROM loaders_versions WHERE version_id = $1
                    ",
                    version_id as database::models::ids::DBVersionId,
                )
                .execute(&mut *transaction)
                .await?;

                let mut loader_versions = Vec::new();
                for loader in loaders {
                    let loader_id =
                        database::models::loader_fields::Loader::get_id(
                            &loader.0,
                            &mut *transaction,
                            &redis,
                        )
                        .await?
                        .ok_or_else(|| {
                            ApiError::InvalidInput(
                                "No database entry for loader provided."
                                    .to_string(),
                            )
                        })?;
                    loader_versions.push(DBLoaderVersion {
                        loader_id,
                        version_id,
                    });
                }
                DBLoaderVersion::insert_many(loader_versions, &mut transaction)
                    .await?;

                crate::database::models::DBProject::clear_cache(
                    version_item.inner.project_id,
                    None,
                    None,
                    &redis,
                )
                .await?;
            }

            if let Some(featured) = &new_version.featured {
                sqlx::query!(
                    "
                    UPDATE versions
                    SET featured = $1
                    WHERE (id = $2)
                    ",
                    featured,
                    version_id as database::models::ids::DBVersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(body) = &new_version.changelog {
                sqlx::query!(
                    "
                    UPDATE versions
                    SET changelog = $1
                    WHERE (id = $2)
                    ",
                    body,
                    version_id as database::models::ids::DBVersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(downloads) = &new_version.downloads {
                if !user.role.is_mod() {
                    return Err(ApiError::CustomAuthentication(
                        "You don't have permission to set the downloads of this mod".to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE versions
                    SET downloads = $1
                    WHERE (id = $2)
                    ",
                    *downloads as i32,
                    version_id as database::models::ids::DBVersionId,
                )
                .execute(&mut *transaction)
                .await?;

                let diff = *downloads - (version_item.inner.downloads as u32);

                sqlx::query!(
                    "
                    UPDATE mods
                    SET downloads = downloads + $1
                    WHERE (id = $2)
                    ",
                    diff as i32,
                    version_item.inner.project_id
                        as database::models::ids::DBProjectId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(status) = &new_version.status {
                if !status.can_be_requested() {
                    return Err(ApiError::InvalidInput(
                        "The requested status cannot be set!".to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE versions
                    SET status = $1
                    WHERE (id = $2)
                    ",
                    status.as_str(),
                    version_id as database::models::ids::DBVersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            if let Some(file_types) = &new_version.file_types {
                for file_type in file_types {
                    let result = sqlx::query!(
                        "
                        SELECT f.id id FROM hashes h
                        INNER JOIN files f ON h.file_id = f.id
                        WHERE h.algorithm = $2 AND h.hash = $1
                        ",
                        file_type.hash.as_bytes(),
                        file_type.algorithm
                    )
                    .fetch_optional(&**pool)
                    .await?
                    .ok_or_else(|| {
                        ApiError::InvalidInput(format!(
                            "Specified file with hash {} does not exist.",
                            file_type.algorithm.clone()
                        ))
                    })?;

                    sqlx::query!(
                        "
                        UPDATE files
                        SET file_type = $2
                        WHERE (id = $1)
                        ",
                        result.id,
                        file_type.file_type.as_ref().map(|x| x.as_str()),
                    )
                    .execute(&mut *transaction)
                    .await?;
                }
            }

            if let Some(ordering) = &new_version.ordering {
                sqlx::query!(
                    "
                    UPDATE versions
                    SET ordering = $1
                    WHERE (id = $2)
                    ",
                    ordering.to_owned() as Option<i32>,
                    version_id as database::models::ids::DBVersionId,
                )
                .execute(&mut *transaction)
                .await?;
            }

            // delete any images no longer in the changelog
            let checkable_strings: Vec<&str> = vec![&new_version.changelog]
                .into_iter()
                .filter_map(|x| x.as_ref().map(|y| y.as_str()))
                .collect();
            let context = ImageContext::Version {
                version_id: Some(version_item.inner.id.into()),
            };

            img::delete_unused_images(
                context,
                checkable_strings,
                &mut transaction,
                &redis,
            )
            .await?;

            transaction.commit().await?;
            database::models::DBVersion::clear_cache(&version_item, &redis)
                .await?;
            database::models::DBProject::clear_cache(
                version_item.inner.project_id,
                None,
                Some(true),
                &redis,
            )
            .await?;
            Ok(HttpResponse::NoContent().body(""))
        } else {
            Err(ApiError::CustomAuthentication(
                "You do not have permission to edit this version!".to_string(),
            ))
        }
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Serialize, Deserialize)]
pub struct VersionListFilters {
    pub loaders: Option<String>,
    pub featured: Option<bool>,
    pub version_type: Option<VersionType>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    /*
        Loader fields to filter with:
        "game_versions": ["1.16.5", "1.17"]

        Returns if it matches any of the values
    */
    pub loader_fields: Option<String>,
}

pub async fn version_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    web::Query(filters): web::Query<VersionListFilters>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let result =
        database::models::DBProject::get(&string, &**pool, &redis).await?;

    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PROJECT_READ, Scopes::VERSION_READ]),
    )
    .await
    .map(|x| x.1)
    .ok();

    if let Some(project) = result {
        if !is_visible_project(&project.inner, &user_option, &pool, false)
            .await?
        {
            return Err(ApiError::NotFound);
        }

        let loader_field_filters = filters.loader_fields.as_ref().map(|x| {
            serde_json::from_str::<HashMap<String, Vec<serde_json::Value>>>(x)
                .unwrap_or_default()
        });
        let loader_filters = filters.loaders.as_ref().map(|x| {
            serde_json::from_str::<Vec<String>>(x).unwrap_or_default()
        });
        let mut versions = database::models::DBVersion::get_many(
            &project.versions,
            &**pool,
            &redis,
        )
        .await?
        .into_iter()
        .skip(filters.offset.unwrap_or(0))
        .take(filters.limit.unwrap_or(usize::MAX))
        .filter(|x| {
            let mut bool = true;

            if let Some(version_type) = filters.version_type {
                bool &= &*x.inner.version_type == version_type.as_str();
            }
            if let Some(loaders) = &loader_filters {
                bool &= x.loaders.iter().any(|y| loaders.contains(y));
            }
            if let Some(loader_fields) = &loader_field_filters {
                for (key, values) in loader_fields {
                    bool &= if let Some(x_vf) =
                        x.version_fields.iter().find(|y| y.field_name == *key)
                    {
                        values.iter().any(|v| x_vf.value.contains_json_value(v))
                    } else {
                        true
                    };
                }
            }
            bool
        })
        .collect::<Vec<_>>();

        let mut response = versions
            .iter()
            .filter(|version| {
                filters
                    .featured
                    .map(|featured| featured == version.inner.featured)
                    .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();

        versions.sort_by(|a, b| {
            b.inner.date_published.cmp(&a.inner.date_published)
        });

        // Attempt to populate versions with "auto featured" versions
        if response.is_empty()
            && !versions.is_empty()
            && filters.featured.unwrap_or(false)
        {
            // TODO: This is a bandaid fix for detecting auto-featured versions.
            // In the future, not all versions will have 'game_versions' fields, so this will need to be changed.
            let (loaders, game_versions) = futures::future::try_join(
                database::models::loader_fields::Loader::list(&**pool, &redis),
                database::models::legacy_loader_fields::MinecraftGameVersion::list(
                    None,
                    Some(true),
                    &**pool,
                    &redis,
                ),
            )
            .await?;

            let mut joined_filters = Vec::new();
            for game_version in &game_versions {
                for loader in &loaders {
                    joined_filters.push((game_version, loader))
                }
            }

            joined_filters.into_iter().for_each(|filter| {
                versions
                    .iter()
                    .find(|version| {
                        // TODO: This is the bandaid fix for detecting auto-featured versions.
                        let game_versions = version
                            .version_fields
                            .iter()
                            .find(|vf| vf.field_name == "game_versions")
                            .map(|vf| vf.value.clone())
                            .map(|v| v.as_strings())
                            .unwrap_or_default();
                        game_versions.contains(&filter.0.version)
                            && version.loaders.contains(&filter.1.loader)
                    })
                    .map(|version| response.push(version.clone()))
                    .unwrap_or(());
            });

            if response.is_empty() {
                versions
                    .into_iter()
                    .for_each(|version| response.push(version));
            }
        }

        response.sort_by(|a, b| {
            b.inner.date_published.cmp(&a.inner.date_published)
        });
        response.dedup_by(|a, b| a.inner.id == b.inner.id);

        let response =
            filter_visible_versions(response, &user_option, &pool, &redis)
                .await?;

        Ok(HttpResponse::Ok().json(response))
    } else {
        Err(ApiError::NotFound)
    }
}

pub async fn version_delete(
    req: HttpRequest,
    info: web::Path<(VersionId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_config: web::Data<SearchConfig>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::VERSION_DELETE]),
    )
    .await?
    .1;
    let id = info.into_inner().0;

    let version = database::models::DBVersion::get(id.into(), &**pool, &redis)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified version does not exist!".to_string(),
            )
        })?;

    if !user.role.is_admin() {
        let team_member =
            database::models::DBTeamMember::get_from_user_id_project(
                version.inner.project_id,
                user.id.into(),
                false,
                &**pool,
            )
            .await
            .map_err(ApiError::Database)?;

        let organization =
            DBOrganization::get_associated_organization_project_id(
                version.inner.project_id,
                &**pool,
            )
            .await?;

        let organization_team_member = if let Some(organization) = &organization
        {
            database::models::DBTeamMember::get_from_user_id(
                organization.team_id,
                user.id.into(),
                &**pool,
            )
            .await?
        } else {
            None
        };
        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(ProjectPermissions::DELETE_VERSION) {
            return Err(ApiError::CustomAuthentication(
                "You do not have permission to delete versions in this team"
                    .to_string(),
            ));
        }
    }

    let mut transaction = pool.begin().await?;
    let context = ImageContext::Version {
        version_id: Some(version.inner.id.into()),
    };
    let uploaded_images = database::models::DBImage::get_many_contexted(
        context,
        &mut transaction,
    )
    .await?;
    for image in uploaded_images {
        image_item::DBImage::remove(image.id, &mut transaction, &redis).await?;
    }

    let result = database::models::DBVersion::remove_full(
        version.inner.id,
        &redis,
        &mut transaction,
    )
    .await?;
    transaction.commit().await?;
    remove_documents(&[version.inner.id.into()], &search_config).await?;
    database::models::DBProject::clear_cache(
        version.inner.project_id,
        None,
        Some(true),
        &redis,
    )
    .await?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound)
    }
}
