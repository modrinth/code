use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use actix_http::StatusCode;
use actix_web::test;
use ariadne::ids::base62_impl::parse_base62;
use common::api_common::models::{CommonProject, CommonVersion};
use common::api_common::{ApiProject, ApiVersion};
use common::database::{ENEMY_USER_PAT, USER_USER_PAT};
use common::environment::{TestEnvironment, with_test_environment};
use dashmap::DashMap;
use labrinth::database::models::DatabaseError;
use labrinth::database::models::project_item::{
    PROJECTS_NAMESPACE, PROJECTS_SLUGS_NAMESPACE,
};
use labrinth::database::models::version_item::VERSIONS_NAMESPACE;
use labrinth::database::redis::{KeyBuilder, RedisPool, RedisTopology};
use redis::cluster_routing::Slot;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{Barrier, Notify};
use tokio::time::timeout;
use uuid::Uuid;

pub mod common;

async fn isolated_redis_pool(label: &str) -> RedisPool {
    labrinth::env::init().expect("failed to initialize test environment");
    RedisPool::new(format!("redis_test_{label}_{}", Uuid::new_v4())).await
}

fn clustered_key_builder(label: &str) -> KeyBuilder {
    KeyBuilder::new(
        format!("redis_test_{label}_{}", Uuid::new_v4()),
        RedisTopology::Cluster,
    )
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct CachedNumber {
    value: usize,
}

/// Protects redis-rs's cross-slot result recombination and Labrinth's 32-key
/// chunk boundary, including missing and duplicate keys whose positions must
/// not shift.
#[actix_rt::test]
async fn cross_slot_mget_preserves_input_order_across_chunks() {
    let pool = isolated_redis_pool("cross_slot_mget").await;
    let keys = clustered_key_builder("cross_slot_mget");
    let mut connection = pool.connect().await.unwrap();

    let stored_keys = (0..65)
        .map(|index| keys.entity("raw", format!("key-{index}")))
        .collect::<Vec<_>>();
    let slots = stored_keys
        .iter()
        .map(Slot::for_key)
        .collect::<HashSet<_>>();
    assert!(slots.len() > 1, "test keys must span Redis hash slots");

    let mut expected_values = HashMap::new();
    for (index, key) in stored_keys.iter().enumerate() {
        let value = format!("value-{index}").into_bytes();
        connection.set(key, &value, None).await.unwrap();
        expected_values.insert(key.clone(), value);
    }

    let mut query_keys = stored_keys.iter().rev().cloned().collect::<Vec<_>>();
    query_keys.insert(1, keys.entity("raw", "missing-first-chunk"));
    query_keys.insert(33, keys.entity("raw", "missing-second-chunk"));
    query_keys.push(stored_keys[7].clone());

    let actual = connection.get_many(&query_keys).await.unwrap();
    let expected = query_keys
        .iter()
        .map(|key| expected_values.get(key).cloned())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);

    // The typed path is used by analytics counters, so it must receive the
    // same cross-slot splitting and ordered recombination as cache reads.
    let counter_keys = (0..3)
        .map(|index| keys.entity("counter", format!("key-{index}")))
        .collect::<Vec<_>>();
    assert!(
        counter_keys
            .iter()
            .map(Slot::for_key)
            .collect::<HashSet<_>>()
            .len()
            > 1
    );
    for (index, key) in counter_keys.iter().enumerate() {
        connection.set(key, index as u32, None).await.unwrap();
    }
    assert_eq!(
        connection
            .get_many_typed::<u32>(&counter_keys)
            .await
            .unwrap(),
        vec![Some(0), Some(1), Some(2)]
    );
}

/// Protects the serialized cache path from losing ordering or cardinality when
/// a many-get crosses both hash slots and Labrinth's command chunk boundary.
#[actix_rt::test]
async fn cross_slot_deserialized_mget_preserves_input_order() {
    let pool = isolated_redis_pool("cross_slot_deserialized").await;
    let keys = clustered_key_builder("cross_slot_deserialized");
    let mut connection = pool.connect().await.unwrap();

    let stored_keys = (0..65)
        .map(|index| keys.entity("serialized", format!("key-{index}")))
        .collect::<Vec<_>>();
    assert!(
        stored_keys
            .iter()
            .map(Slot::for_key)
            .collect::<HashSet<_>>()
            .len()
            > 1
    );
    for (index, key) in stored_keys.iter().enumerate() {
        connection
            .set_serialized(key, CachedNumber { value: index }, None)
            .await
            .unwrap();
    }

    let mut query_keys = stored_keys.iter().rev().cloned().collect::<Vec<_>>();
    query_keys.insert(32, keys.entity("serialized", "missing"));
    query_keys.push(stored_keys[12].clone());

    let actual = connection
        .get_many_deserialized::<CachedNumber>(&query_keys)
        .await
        .unwrap();
    let expected = query_keys
        .iter()
        .map(|key| {
            stored_keys
                .iter()
                .position(|stored| stored == key)
                .map(|value| CachedNumber { value })
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

/// Protects cross-slot invalidation, same-slot hash-tag behavior, and empty
/// batches; these are the three shapes used by model cache clear operations.
#[actix_rt::test]
async fn delete_many_and_same_slot_batches_work_in_cluster_mode() {
    let pool = isolated_redis_pool("delete_many").await;
    let keys = clustered_key_builder("delete_many");
    let mut connection = pool.connect().await.unwrap();

    let cross_slot_keys = (0..5)
        .map(|index| keys.entity("delete", format!("key-{index}")))
        .collect::<Vec<_>>();
    assert!(
        cross_slot_keys
            .iter()
            .map(Slot::for_key)
            .collect::<HashSet<_>>()
            .len()
            > 1
    );
    for key in &cross_slot_keys {
        connection.set(key, "present", None).await.unwrap();
    }
    connection.delete_many(&cross_slot_keys).await.unwrap();
    assert_eq!(
        connection.get_many(&cross_slot_keys).await.unwrap(),
        vec![None; cross_slot_keys.len()]
    );

    let same_slot_keys = vec![
        keys.with_slot("same_slot", "left", "shared-entity"),
        keys.with_slot("same_slot", "right", "shared-entity"),
    ];
    assert_eq!(
        Slot::for_key(&same_slot_keys[0]),
        Slot::for_key(&same_slot_keys[1])
    );
    connection
        .set(&same_slot_keys[0], "left", None)
        .await
        .unwrap();
    connection
        .set(&same_slot_keys[1], "right", None)
        .await
        .unwrap();
    assert_eq!(
        connection.get_many(&same_slot_keys).await.unwrap(),
        vec![Some(b"left".to_vec()), Some(b"right".to_vec())]
    );

    connection.delete_many(&[]).await.unwrap();
    assert!(connection.get_many(&[]).await.unwrap().is_empty());
}

/// Protects hash tags from user-controlled braces, percent signs, and empty
/// values changing the substring Redis uses to select a cluster slot.
#[actix_rt::test]
async fn cluster_key_builder_escapes_slot_tags() {
    let keys = clustered_key_builder("slot_escaping");

    for (slot_tag, escaped) in [
        ("", "%00"),
        ("contains%percent", "contains%25percent"),
        ("contains{open", "contains%7Bopen"),
        ("contains}close", "contains%7Dclose"),
    ] {
        let first = keys.with_slot("escape", "first", slot_tag);
        let second = keys.with_slot("escape", "second", slot_tag);
        assert!(first.contains(&format!("{{{escaped}}}")));
        assert_eq!(Slot::for_key(first), Slot::for_key(second));
    }

    // Metadata keys intentionally share one slot so metadata pipelines remain
    // legal even when their logical entity IDs differ.
    assert_eq!(
        Slot::for_key(keys.metadata("metadata", "first")),
        Slot::for_key(keys.metadata("metadata", "second"))
    );
}

/// Protects the process-local single-flight contract: concurrent cache misses
/// for one key perform one backing fetch and all waiters receive its value.
#[actix_rt::test]
async fn cache_lock_coalesces_concurrent_misses_for_one_key() {
    let pool = isolated_redis_pool("single_flight").await;
    let barrier = Arc::new(Barrier::new(16));
    let fetch_count = Arc::new(AtomicUsize::new(0));
    let mut tasks = Vec::new();

    for _ in 0..16 {
        let pool = pool.clone();
        let barrier = barrier.clone();
        let fetch_count = fetch_count.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            pool.get_cached_keys_raw(
                "single_flight:v1",
                &["shared".to_string()],
                move |keys| async move {
                    fetch_count.fetch_add(1, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(75)).await;
                    let values = DashMap::new();
                    for key in keys {
                        values.insert(key.clone(), format!("value-{key}"));
                    }
                    Ok::<_, DatabaseError>(values)
                },
            )
            .await
        }));
    }

    for task in tasks {
        let values = task.await.unwrap().unwrap();
        assert_eq!(values.get("shared"), Some(&"value-shared".to_string()));
    }
    assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
}

/// Protects per-key locking for overlapping many-gets: `[A, B]` and `[B, C]`
/// may fetch independently, but the shared `B` must only be loaded once.
#[actix_rt::test]
async fn cache_lock_coalesces_only_overlapping_keys() {
    let pool = isolated_redis_pool("overlapping_locks").await;
    let barrier = Arc::new(Barrier::new(2));
    let fetch_counts = Arc::new(DashMap::<String, usize>::new());
    let mut tasks = Vec::new();

    for requested in [
        vec!["A".to_string(), "B".to_string()],
        vec!["B".to_string(), "C".to_string()],
    ] {
        let pool = pool.clone();
        let barrier = barrier.clone();
        let fetch_counts = fetch_counts.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            pool.get_cached_keys_raw(
                "overlapping_locks:v1",
                &requested,
                move |keys| async move {
                    tokio::time::sleep(Duration::from_millis(75)).await;
                    let values = DashMap::new();
                    for key in keys {
                        fetch_counts
                            .entry(key.clone())
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                        values.insert(key.clone(), format!("value-{key}"));
                    }
                    Ok::<_, DatabaseError>(values)
                },
            )
            .await
        }));
    }

    for task in tasks {
        assert_eq!(task.await.unwrap().unwrap().len(), 2);
    }
    for key in ["A", "B", "C"] {
        assert_eq!(fetch_counts.get(key).map(|count| *count), Some(1));
    }
}

/// Protects the lock map from becoming a global mutex: a slow miss for one
/// key must not delay an unrelated cache key.
#[actix_rt::test]
async fn cache_lock_does_not_block_independent_keys() {
    let pool = isolated_redis_pool("independent_locks").await;
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());

    let slow_pool = pool.clone();
    let slow_started = started.clone();
    let slow_release = release.clone();
    let slow = tokio::spawn(async move {
        slow_pool
            .get_cached_keys_raw(
                "independent_locks:v1",
                &["slow".to_string()],
                move |keys| async move {
                    slow_started.notify_one();
                    slow_release.notified().await;
                    let values = DashMap::new();
                    values.insert(keys[0].clone(), "slow-value".to_string());
                    Ok::<_, DatabaseError>(values)
                },
            )
            .await
    });
    started.notified().await;

    let fast = timeout(
        Duration::from_secs(1),
        pool.get_cached_keys_raw(
            "independent_locks:v1",
            &["fast".to_string()],
            |keys| async move {
                let values = DashMap::new();
                values.insert(keys[0].clone(), "fast-value".to_string());
                Ok::<_, DatabaseError>(values)
            },
        ),
    )
    .await
    .expect("an unrelated key was blocked by the slow cache fill")
    .unwrap();
    assert_eq!(fast.get("fast"), Some(&"fast-value".to_string()));

    release.notify_one();
    assert_eq!(
        slow.await.unwrap().unwrap().get("slow"),
        Some(&"slow-value".to_string())
    );
}

/// Protects lock cleanup on both error and cancellation so a failed cache fill
/// cannot strand future requests behind a stale local lock.
#[actix_rt::test]
async fn cache_lock_is_released_after_error_and_cancellation() {
    let pool = isolated_redis_pool("lock_recovery").await;

    let failed = pool
        .get_cached_keys_raw(
            "error_recovery:v1",
            &["key".to_string()],
            |_| async {
                Err::<DashMap<String, String>, _>(DatabaseError::Internal(
                    eyre::eyre!("intentional cache fill failure"),
                ))
            },
        )
        .await;
    assert!(failed.is_err());

    let recovered = timeout(
        Duration::from_secs(1),
        pool.get_cached_keys_raw(
            "error_recovery:v1",
            &["key".to_string()],
            |keys| async move {
                let values = DashMap::new();
                values.insert(keys[0].clone(), "recovered".to_string());
                Ok::<_, DatabaseError>(values)
            },
        ),
    )
    .await
    .expect("error left the cache key locked")
    .unwrap();
    assert_eq!(recovered.get("key"), Some(&"recovered".to_string()));

    let started = Arc::new(Notify::new());
    let cancelled_pool = pool.clone();
    let cancelled_started = started.clone();
    let cancelled = tokio::spawn(async move {
        cancelled_pool
            .get_cached_keys_raw(
                "cancellation_recovery:v1",
                &["key".to_string()],
                move |_| async move {
                    cancelled_started.notify_one();
                    std::future::pending::<
                        Result<DashMap<String, String>, DatabaseError>,
                    >()
                    .await
                },
            )
            .await
    });
    started.notified().await;
    cancelled.abort();
    assert!(cancelled.await.unwrap_err().is_cancelled());

    let recovered = timeout(
        Duration::from_secs(1),
        pool.get_cached_keys_raw(
            "cancellation_recovery:v1",
            &["key".to_string()],
            |keys| async move {
                let values = DashMap::new();
                values.insert(keys[0].clone(), "recovered".to_string());
                Ok::<_, DatabaseError>(values)
            },
        ),
    )
    .await
    .expect("cancellation left the cache key locked")
    .unwrap();
    assert_eq!(recovered.get("key"), Some(&"recovered".to_string()));
}

/// Protects stale-while-revalidate behavior: an expired cached value may serve
/// a waiter immediately while exactly one writer refreshes it in the background.
#[actix_rt::test]
async fn expired_cache_value_serves_waiter_while_writer_refreshes() {
    let pool = isolated_redis_pool("stale_while_revalidate").await;
    let namespace = "stale_while_revalidate:v1";
    let logical_key = "key".to_string();
    let mut connection = pool.connect().await.unwrap();
    let redis_key = connection.key().entity(namespace, &logical_key);
    connection
        .set_serialized(
            &redis_key,
            json!({
                "key": logical_key,
                "alias": null,
                "iat": 0,
                "val": "stale",
            }),
            None,
        )
        .await
        .unwrap();

    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let writer_pool = pool.clone();
    let writer_started = started.clone();
    let writer_release = release.clone();
    let writer = tokio::spawn(async move {
        writer_pool
            .get_cached_keys_raw(
                namespace,
                &["key".to_string()],
                move |keys| async move {
                    writer_started.notify_one();
                    writer_release.notified().await;
                    let values = DashMap::new();
                    values.insert(keys[0].clone(), "fresh".to_string());
                    Ok::<_, DatabaseError>(values)
                },
            )
            .await
    });
    started.notified().await;

    let stale = timeout(
        Duration::from_secs(1),
        pool.get_cached_keys_raw(namespace, &["key".to_string()], |_| async {
            Err::<DashMap<String, String>, _>(DatabaseError::Internal(
                eyre::eyre!("stale waiter unexpectedly became writer"),
            ))
        }),
    )
    .await
    .expect("stale value waited for the refresh")
    .unwrap();
    assert_eq!(stale.get("key"), Some(&"stale".to_string()));

    release.notify_one();
    assert_eq!(
        writer.await.unwrap().unwrap().get("key"),
        Some(&"fresh".to_string())
    );
    let fresh = pool
        .get_cached_keys_raw(namespace, &["key".to_string()], |_| async {
            Err::<DashMap<String, String>, _>(DatabaseError::Internal(
                eyre::eyre!("fresh value unexpectedly missed cache"),
            ))
        })
        .await
        .unwrap();
    assert_eq!(fresh.get("key"), Some(&"fresh".to_string()));
}

/// Protects case-insensitive slug locking: differently-cased aliases must map
/// to one local lock and one backing fetch, matching their lowercase Redis key.
#[actix_rt::test]
async fn case_insensitive_slug_requests_share_one_cache_lock() {
    let pool = isolated_redis_pool("slug_lock_casing").await;
    let canonical_id = "A1b2C3d4";
    let barrier = Arc::new(Barrier::new(2));
    let fetch_count = Arc::new(AtomicUsize::new(0));
    let mut tasks = Vec::new();

    for requested in ["MiXeD-Slug".to_string(), "mixed-slug".to_string()] {
        let pool = pool.clone();
        let barrier = barrier.clone();
        let fetch_count = fetch_count.clone();
        tasks.push(tokio::spawn(async move {
            let requested = vec![requested];
            barrier.wait().await;
            pool.get_cached_keys_raw_with_slug(
                "slug_values:v1",
                Some("slug_aliases:v1"),
                false,
                &requested,
                move |_| async move {
                    fetch_count.fetch_add(1, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(75)).await;
                    let values = DashMap::new();
                    values.insert(
                        canonical_id.to_string(),
                        (Some("MiXeD-Slug".to_string()), "value".to_string()),
                    );
                    Ok::<_, DatabaseError>(values)
                },
            )
            .await
        }));
    }

    for task in tasks {
        let values = task.await.unwrap().unwrap();
        assert_eq!(values.get(canonical_id), Some(&"value".to_string()));
    }
    assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
}

/// Protects the cluster-aware blocking pool and Pub/Sub seed rotation, which
/// use separate connection paths from ordinary cache commands.
#[actix_rt::test]
async fn blocking_queue_and_pubsub_work_with_cluster_connections() {
    let pool = isolated_redis_pool("blocking_and_pubsub").await;
    let queue_key = pool.key().entity("blocking", "queue");
    let mut connection = pool.connect().await.unwrap();
    connection
        .lpush(&queue_key, b"queued-value".as_slice())
        .await
        .unwrap();
    let popped = pool
        .brpop(&queue_key, Duration::from_secs(1))
        .await
        .unwrap()
        .expect("BRPOP did not receive the queued value");
    assert_eq!(popped[0], queue_key.as_bytes());
    assert_eq!(popped[1], b"queued-value");

    // An unreachable first seed verifies that the reconnect loop advances to
    // another cluster seed instead of abandoning the subscription.
    let valid_seed = labrinth::env::ENV
        .REDIS_URL
        .split(',')
        .next()
        .unwrap()
        .to_string();
    let channel: &'static str = Box::leak(
        format!("redis-test-pubsub-{}", Uuid::new_v4()).into_boxed_str(),
    );
    let mut receiver = RedisPool::subscribe_with_seed_urls(
        vec!["redis://127.0.0.1:1".to_string(), valid_seed],
        channel,
    );

    let mut delivered = false;
    for _ in 0..20 {
        pool.publish(channel, b"cluster-message".as_slice())
            .await
            .unwrap();
        if let Ok(Some(message)) =
            timeout(Duration::from_millis(100), receiver.recv()).await
        {
            assert_eq!(message, b"cluster-message");
            delivered = true;
            break;
        }
    }
    assert!(delivered, "Pub/Sub did not recover from the invalid seed");
}

/// Protects the real many-get routes as one end-to-end cluster scenario:
/// partial cache hits, IDs and case-insensitive slugs, duplicates, misses,
/// invalidation, file hashes, dependency caches, and visibility filtering.
#[actix_rt::test]
async fn many_get_routes_handle_cross_slot_cache_lifecycle() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<common::api_v3::ApiV3>| async move {
            let api = &test_env.api;
            let alpha = &test_env.dummy.project_alpha;
            let beta = &test_env.dummy.project_beta;
            let alpha_project_id = parse_base62(&alpha.project_id).unwrap();
            let beta_project_id = parse_base62(&beta.project_id).unwrap();
            let alpha_version_id = parse_base62(&alpha.version_id).unwrap();
            let beta_version_id = parse_base62(&beta.version_id).unwrap();
            let missing_project_id = "R3d1sPj8";
            let missing_version_id = "R3d1sVn8";

            // Clearing known dummy-data entries makes the first single read a
            // deliberate warm hit and the other entities deliberate cold hits.
            let mut redis = test_env.db.redis_pool.connect().await.unwrap();
            let cache_keys = vec![
                redis.key().entity(PROJECTS_NAMESPACE, alpha_project_id),
                redis.key().entity(PROJECTS_NAMESPACE, beta_project_id),
                redis.key().entity(
                    PROJECTS_SLUGS_NAMESPACE,
                    alpha.project_slug.to_lowercase(),
                ),
                redis.key().entity(
                    PROJECTS_SLUGS_NAMESPACE,
                    beta.project_slug.to_lowercase(),
                ),
                redis.key().entity(VERSIONS_NAMESPACE, alpha_version_id),
                redis.key().entity(VERSIONS_NAMESPACE, beta_version_id),
                redis.key().entity(
                    "versions_files:v1",
                    format!("sha1_{}", alpha.file_hash),
                ),
                redis.key().entity(
                    "versions_files:v1",
                    format!("sha1_{}", beta.file_hash),
                ),
            ];
            assert!(
                cache_keys
                    .iter()
                    .map(Slot::for_key)
                    .collect::<HashSet<_>>()
                    .len()
                    > 1,
                "route cache keys must exercise multiple slots"
            );
            redis.delete_many(&cache_keys).await.unwrap();

            api.get_project_deserialized_common(
                &alpha.project_id,
                USER_USER_PAT,
            )
            .await;
            let project_cache_keys = vec![
                redis.key().entity(PROJECTS_NAMESPACE, alpha_project_id),
                redis.key().entity(PROJECTS_NAMESPACE, beta_project_id),
            ];
            let cached_projects =
                redis.get_many(&project_cache_keys).await.unwrap();
            assert!(cached_projects[0].is_some());
            assert!(cached_projects[1].is_none());

            let uppercase_beta_slug = beta.project_slug.to_uppercase();
            let project_request = [
                alpha.project_id.as_str(),
                uppercase_beta_slug.as_str(),
                missing_project_id,
                alpha.project_slug.as_str(),
                alpha.project_id.as_str(),
            ];
            for _ in 0..2 {
                let response =
                    api.get_projects(&project_request, USER_USER_PAT).await;
                assert_status!(&response, StatusCode::OK);
                let projects: Vec<CommonProject> =
                    test::read_body_json(response).await;
                let project_ids = projects
                    .iter()
                    .map(|project| project.id.to_string())
                    .collect::<HashSet<_>>();
                assert_eq!(projects.len(), 2);
                assert_eq!(
                    project_ids,
                    HashSet::from([
                        alpha.project_id.clone(),
                        beta.project_id.clone(),
                    ])
                );
            }

            api.get_version_deserialized_common(
                &alpha.version_id,
                USER_USER_PAT,
            )
            .await;
            let versions = api
                .get_versions_deserialized_common(
                    vec![
                        alpha.version_id.clone(),
                        beta.version_id.clone(),
                        missing_version_id.to_string(),
                        alpha.version_id.clone(),
                    ],
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(
                versions
                    .iter()
                    .map(|version| version.id.to_string())
                    .collect::<HashSet<_>>(),
                HashSet::from([
                    alpha.version_id.clone(),
                    beta.version_id.clone(),
                ])
            );

            let hashes = [
                alpha.file_hash.as_str(),
                beta.file_hash.as_str(),
                "missing-hash",
                alpha.file_hash.as_str(),
            ];
            let versions_by_hash = api
                .get_versions_from_hashes_deserialized_common(
                    &hashes,
                    "sha1",
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(versions_by_hash.len(), 2);

            // Warming dependencies before the edit ensures the assertion below
            // depends on invalidating the project's separate dependency slot.
            let dependencies = api
                .get_project_dependencies(&alpha.project_id, USER_USER_PAT)
                .await;
            assert_status!(&dependencies, StatusCode::OK);
            let dependencies: serde_json::Value =
                test::read_body_json(dependencies).await;
            assert!(dependencies["projects"].as_array().unwrap().is_empty());

            let new_slug = format!("cluster-cache-{}", Uuid::new_v4().simple());
            let response = api
                .edit_project(
                    &alpha.project_id,
                    json!({ "slug": new_slug }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);
            assert_status!(
                &api.get_project(&alpha.project_slug, USER_USER_PAT).await,
                StatusCode::NOT_FOUND
            );
            assert_status!(
                &api.get_project(&new_slug, USER_USER_PAT).await,
                StatusCode::OK
            );

            let updated_version_name = "cluster cache invalidation";
            let response = api
                .edit_version(
                    &alpha.version_id,
                    json!({
                        "name": updated_version_name,
                        "dependencies": [{
                            "project_id": beta.project_id,
                            "dependency_type": "required",
                            "file_name": "cluster-test.jar",
                        }],
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let updated_versions: Vec<CommonVersion> = api
                .get_versions_deserialized_common(
                    vec![alpha.version_id.clone(), beta.version_id.clone()],
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(
                updated_versions
                    .iter()
                    .find(|version| version.id.to_string() == alpha.version_id)
                    .unwrap()
                    .name,
                updated_version_name
            );
            let updated_by_hash = api
                .get_versions_from_hashes_deserialized_common(
                    &[alpha.file_hash.as_str(), beta.file_hash.as_str()],
                    "sha1",
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(
                updated_by_hash[&alpha.file_hash].name,
                updated_version_name
            );

            let dependencies = api
                .get_project_dependencies(&alpha.project_id, USER_USER_PAT)
                .await;
            assert_status!(&dependencies, StatusCode::OK);
            let dependencies: serde_json::Value =
                test::read_body_json(dependencies).await;
            assert!(
                dependencies["projects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|project| project["id"] == beta.project_id)
            );

            // Visibility is evaluated after cache resolution, so a warm private
            // project must not leak through a many-get to another user.
            let response = api
                .get_projects(
                    &[alpha.project_id.as_str(), beta.project_id.as_str()],
                    ENEMY_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::OK);
            let visible_projects: Vec<CommonProject> =
                test::read_body_json(response).await;
            assert_eq!(visible_projects.len(), 1);
            assert_eq!(visible_projects[0].id.to_string(), alpha.project_id);
        },
    )
    .await;
}
