use crate::auth::get_user_from_headers;
use crate::auth::validate::get_maybe_user_from_headers;
use crate::database::PgPool;
use crate::database::models::shared_instance_item::{
    DBSharedInstance, DBSharedInstanceUser, DBSharedInstanceVersion,
};
use crate::database::models::{
    DBSharedInstanceId, DBSharedInstanceVersionId, generate_shared_instance_id,
};
use crate::database::redis::RedisPool;
use crate::file_hosting::FileHost;
use crate::models::ids::{SharedInstanceId, SharedInstanceVersionId};
use crate::models::pats::Scopes;
use crate::models::shared_instances::{
    SharedInstance, SharedInstanceUserPermissions, SharedInstanceVersion,
};
use crate::models::users::User;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::routes::read_typed_from_payload;
use actix_web::web::{Data, Redirect};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use futures_util::future::try_join_all;
use serde::Deserialize;
use validator::Validate;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(shared_instance_create)
        .service(shared_instance_list)
        .service(shared_instance_get)
        .service(shared_instance_edit)
        .service(shared_instance_delete)
        .service(shared_instance_version_list)
        .service(shared_instance_version_get)
        .service(shared_instance_version_delete)
        .service(shared_instance_version_download);
}

#[derive(Deserialize, Validate)]
pub struct CreateSharedInstance {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    pub title: String,
    #[serde(default)]
    pub public: bool,
}

#[utoipa::path(tag = "shared instances", responses((status = OK)))]
#[post("/shared-instance")]
pub async fn shared_instance_create(
    req: HttpRequest,
    pool: Data<PgPool>,
    mut body: web::Payload,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let new_instance: CreateSharedInstance =
        read_typed_from_payload(&mut body).await?;

    let mut transaction = pool.begin().await?;

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SHARED_INSTANCE_CREATE,
    )
    .await?
    .1;

    let id = generate_shared_instance_id(&mut transaction).await?;

    let instance = DBSharedInstance {
        id,
        title: new_instance.title,
        owner_id: user.id.into(),
        public: new_instance.public,
        current_version_id: None,
    };
    instance.insert(&mut transaction).await?;

    transaction.commit().await?;

    Ok(HttpResponse::Created().json(SharedInstance {
        id: id.into(),
        title: instance.title,
        owner: user.id,
        public: instance.public,
        current_version: None,
        additional_users: Some(vec![]),
    }))
}

#[utoipa::path(tag = "shared instances", responses((status = OK)))]
#[get("/shared-instance")]
pub async fn shared_instance_list(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SHARED_INSTANCE_READ,
    )
    .await?
    .1;

    // TODO: Something for moderators to be able to see all instances?
    let instances =
        DBSharedInstance::list_for_user(user.id.into(), &**pool).await?;
    let instances = try_join_all(instances.into_iter().map(
        async |instance| -> Result<SharedInstance, ApiError> {
            let version = if let Some(version_id) = instance.current_version_id
            {
                DBSharedInstanceVersion::get(version_id, &**pool).await?
            } else {
                None
            };
            let instance_id = instance.id;
            Ok(SharedInstance::from_db(
                instance,
                Some(
                    DBSharedInstanceUser::get_from_instance(
                        instance_id,
                        &**pool,
                        &redis,
                    )
                    .await?,
                ),
                version,
            ))
        },
    ))
    .await?;

    Ok(HttpResponse::Ok().json(instances))
}

#[utoipa::path(tag = "shared instances", responses((status = OK)))]
#[get("/shared-instance/{id}")]
pub async fn shared_instance_get(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    info: web::Path<(SharedInstanceId,)>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0.into();

    let user = get_maybe_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SHARED_INSTANCE_READ,
    )
    .await?
    .map(|(_, user)| user);

    let shared_instance = DBSharedInstance::get(id, &**pool).await?;

    if let Some(shared_instance) = shared_instance {
        let users =
            DBSharedInstanceUser::get_from_instance(id, &**pool, &redis)
                .await?;

        let privately_accessible = user.is_some_and(|user| {
            can_access_instance_privately(&shared_instance, &users, &user)
        });
        if !shared_instance.public && !privately_accessible {
            return Err(ApiError::NotFound);
        }

        let current_version =
            if let Some(version_id) = shared_instance.current_version_id {
                DBSharedInstanceVersion::get(version_id, &**pool).await?
            } else {
                None
            };
        let shared_instance = SharedInstance::from_db(
            shared_instance,
            privately_accessible.then_some(users),
            current_version,
        );

        Ok(HttpResponse::Ok().json(shared_instance))
    } else {
        Err(ApiError::NotFound)
    }
}

fn can_access_instance_privately(
    instance: &DBSharedInstance,
    users: &[DBSharedInstanceUser],
    user: &User,
) -> bool {
    user.role.is_mod()
        || instance.owner_id == user.id.into()
        || users.iter().any(|x| x.user_id == user.id.into())
}

#[derive(Deserialize, Validate)]
pub struct EditSharedInstance {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    pub title: Option<String>,
    pub public: Option<bool>,
}

#[utoipa::path(tag = "shared instances", responses((status = NO_CONTENT)))]
#[patch("/shared-instance/{id}")]
pub async fn shared_instance_edit(
    req: HttpRequest,
    pool: Data<PgPool>,
    mut body: web::Payload,
    redis: Data<RedisPool>,
    info: web::Path<(SharedInstanceId,)>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0.into();
    let edit_instance: EditSharedInstance =
        read_typed_from_payload(&mut body).await?;

    let mut transaction = pool.begin().await?;

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SHARED_INSTANCE_WRITE,
    )
    .await?
    .1;

    let Some(instance) = DBSharedInstance::get(id, &**pool).await? else {
        return Err(ApiError::NotFound);
    };

    if !user.role.is_mod() && instance.owner_id != user.id.into() {
        let permissions = DBSharedInstanceUser::get_user_permissions(
            id,
            user.id.into(),
            &**pool,
        )
        .await?;
        if let Some(permissions) = permissions {
            if !permissions.contains(SharedInstanceUserPermissions::EDIT) {
                return Err(ApiError::CustomAuthentication(
                    "You do not have permission to edit this shared instance."
                        .to_string(),
                ));
            }
        } else {
            return Err(ApiError::NotFound);
        }
    }

    if let Some(title) = edit_instance.title {
        sqlx::query!(
            "
            UPDATE shared_instances
            SET title = $1
            WHERE id = $2
            ",
            title,
            id as DBSharedInstanceId,
        )
        .execute(&mut transaction)
        .await?;
    }

    if let Some(public) = edit_instance.public {
        sqlx::query!(
            "
            UPDATE shared_instances
            SET public = $1
            WHERE id = $2
            ",
            public,
            id as DBSharedInstanceId,
        )
        .execute(&mut transaction)
        .await?;
    }

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[utoipa::path(tag = "shared instances", responses((status = NO_CONTENT)))]
#[delete("/shared-instance/{id}")]
pub async fn shared_instance_delete(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    info: web::Path<(SharedInstanceId,)>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let id: DBSharedInstanceId = info.into_inner().0.into();

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SHARED_INSTANCE_DELETE,
    )
    .await?
    .1;

    let Some(instance) = DBSharedInstance::get(id, &**pool).await? else {
        return Err(ApiError::NotFound);
    };

    if !user.role.is_mod() && instance.owner_id != user.id.into() {
        let permissions = DBSharedInstanceUser::get_user_permissions(
            id,
            user.id.into(),
            &**pool,
        )
        .await?;
        if let Some(permissions) = permissions {
            if !permissions.contains(SharedInstanceUserPermissions::DELETE) {
                return Err(ApiError::CustomAuthentication(
                    "You do not have permission to delete this shared instance.".to_string()
                ));
            }
        } else {
            return Err(ApiError::NotFound);
        }
    }

    sqlx::query!(
        "
        DELETE FROM shared_instances
        WHERE id = $1
        ",
        id as DBSharedInstanceId,
    )
    .execute(&**pool)
    .await?;

    DBSharedInstanceUser::clear_cache(id, &redis).await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[utoipa::path(tag = "shared instances", responses((status = OK)))]
#[get("/shared-instance/{id}/version")]
pub async fn shared_instance_version_list(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    info: web::Path<(SharedInstanceId,)>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0.into();

    let user = get_maybe_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SHARED_INSTANCE_READ,
    )
    .await?
    .map(|(_, user)| user);

    let shared_instance = DBSharedInstance::get(id, &**pool).await?;

    if let Some(shared_instance) = shared_instance {
        if !can_access_instance_as_maybe_user(
            &pool,
            &redis,
            &shared_instance,
            user,
        )
        .await?
        {
            return Err(ApiError::NotFound);
        }

        let versions =
            DBSharedInstanceVersion::get_for_instance(id, &**pool).await?;
        let versions = versions
            .into_iter()
            .map(Into::into)
            .collect::<Vec<SharedInstanceVersion>>();

        Ok(HttpResponse::Ok().json(versions))
    } else {
        Err(ApiError::NotFound)
    }
}

#[utoipa::path(tag = "shared instances", responses((status = OK)))]
#[get("/shared-instance-version/{id}")]
pub async fn shared_instance_version_get(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    info: web::Path<(SharedInstanceVersionId,)>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let version_id = info.into_inner().0.into();

    let user = get_maybe_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SHARED_INSTANCE_READ,
    )
    .await?
    .map(|(_, user)| user);

    let version = DBSharedInstanceVersion::get(version_id, &**pool).await?;

    if let Some(version) = version {
        let instance =
            DBSharedInstance::get(version.shared_instance_id, &**pool).await?;
        if let Some(instance) = instance {
            if !can_access_instance_as_maybe_user(
                &pool, &redis, &instance, user,
            )
            .await?
            {
                return Err(ApiError::NotFound);
            }

            let version: SharedInstanceVersion = version.into();
            Ok(HttpResponse::Ok().json(version))
        } else {
            Err(ApiError::NotFound)
        }
    } else {
        Err(ApiError::NotFound)
    }
}

async fn can_access_instance_as_maybe_user(
    pool: &PgPool,
    redis: &RedisPool,
    instance: &DBSharedInstance,
    user: Option<User>,
) -> Result<bool, ApiError> {
    if instance.public {
        return Ok(true);
    }
    let users =
        DBSharedInstanceUser::get_from_instance(instance.id, pool, redis)
            .await?;
    Ok(user.is_some_and(|user| {
        can_access_instance_privately(instance, &users, &user)
    }))
}

#[utoipa::path(tag = "shared instances", responses((status = NO_CONTENT)))]
#[delete("/shared-instance-version/{id}")]
pub async fn shared_instance_version_delete(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    info: web::Path<(SharedInstanceVersionId,)>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let version_id = info.into_inner().0.into();

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SHARED_INSTANCE_VERSION_DELETE,
    )
    .await?
    .1;

    let shared_instance_version =
        DBSharedInstanceVersion::get(version_id, &**pool).await?;

    if let Some(shared_instance_version) = shared_instance_version {
        let shared_instance = DBSharedInstance::get(
            shared_instance_version.shared_instance_id,
            &**pool,
        )
        .await?;
        if let Some(shared_instance) = shared_instance {
            if !user.role.is_mod() && shared_instance.owner_id != user.id.into()
            {
                let permissions = DBSharedInstanceUser::get_user_permissions(
                    shared_instance.id,
                    user.id.into(),
                    &**pool,
                )
                .await?;
                if let Some(permissions) = permissions {
                    if !permissions
                        .contains(SharedInstanceUserPermissions::DELETE)
                    {
                        return Err(ApiError::CustomAuthentication(
                            "You do not have permission to delete this shared instance version.".to_string()
                        ));
                    }
                } else {
                    return Err(ApiError::NotFound);
                }
            }

            delete_instance_version(shared_instance.id, version_id, &pool)
                .await?;

            Ok(HttpResponse::NoContent().body(""))
        } else {
            Err(ApiError::NotFound)
        }
    } else {
        Err(ApiError::NotFound)
    }
}

async fn delete_instance_version(
    instance_id: DBSharedInstanceId,
    version_id: DBSharedInstanceVersionId,
    pool: &PgPool,
) -> Result<(), ApiError> {
    let mut transaction = pool.begin().await?;

    sqlx::query!(
        "
        DELETE FROM shared_instance_versions
        WHERE id = $1
        ",
        version_id as DBSharedInstanceVersionId,
    )
    .execute(&mut transaction)
    .await?;

    sqlx::query!(
        "
        UPDATE shared_instances
        SET current_version_id = (
            SELECT id FROM shared_instance_versions
            WHERE shared_instance_id = $1
            ORDER BY created DESC
            LIMIT 1
        )
        WHERE id = $1
        ",
        instance_id as DBSharedInstanceId,
    )
    .execute(&mut transaction)
    .await?;

    transaction.commit().await?;
    Ok(())
}

#[utoipa::path(tag = "shared instances", responses((status = TEMPORARY_REDIRECT)))]
#[get("/shared-instance-version/{id}/download")]
pub async fn shared_instance_version_download(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    file_host: Data<dyn FileHost>,
    info: web::Path<(SharedInstanceVersionId,)>,
    session_queue: Data<AuthQueue>,
) -> Result<Redirect, ApiError> {
    let version_id = info.into_inner().0.into();

    let user = get_maybe_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SHARED_INSTANCE_VERSION_READ,
    )
    .await?
    .map(|(_, user)| user);

    let version = DBSharedInstanceVersion::get(version_id, &**pool).await?;

    if let Some(version) = version {
        let instance =
            DBSharedInstance::get(version.shared_instance_id, &**pool).await?;
        if let Some(instance) = instance {
            if !can_access_instance_as_maybe_user(
                &pool, &redis, &instance, user,
            )
            .await?
            {
                return Err(ApiError::NotFound);
            }

            let file_name = format!(
                "shared_instance/{}.mrpack",
                SharedInstanceVersionId::from(version_id)
            );
            let url =
                file_host.get_url_for_private_file(&file_name, 180).await?;

            Ok(Redirect::to(url).see_other())
        } else {
            Err(ApiError::NotFound)
        }
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    shared_instance_create,
    shared_instance_list,
    shared_instance_get,
    shared_instance_edit,
    shared_instance_delete,
    shared_instance_version_list,
    shared_instance_version_get,
    shared_instance_version_delete,
    shared_instance_version_download,
))]
#[allow(dead_code)]
pub(crate) struct RouteDoc;
