use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::{self, header, multipart};
use tokio::fs;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;
use crate::auth;
use crate::data::DirectoryInfo;
use crate::prelude::Credentials;
use crate::process::Settings;
use crate::util::fetch::{read_json, write, IoSemaphore};

// Get image size
pub async fn check_image(path: String) -> crate::Result<bool> {
    let (w, h) = image::image_dimensions(path)?;
    Ok(w == 64 && h == 64)
}

pub async fn get_heads() -> crate::Result<HashMap<Uuid, String>> {
    let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
    let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
    )));
    let path = crate::State::get().await?
        .directories
        .caches_meta_dir()
        .await
        .join("skindata.json");

    let cache: Cache = 
        if let Ok(json) = read_json::<Cache>(&path, &io_semaphore).await {
            json
        } else {
            Cache {
                capes: HashMap::new(),
                users: HashMap::new(),
                heads: HashMap::new()
            }
        };
    Ok(cache.heads)
}

// Sets player's skin
pub async fn set_skin(skin: String, arms: String, user: Credentials) -> crate::Result<bool> {
    let mut result = false;
    let token = user.access_token;
    let file = if skin.starts_with("data:image/png;base64,") {
        STANDARD.decode(skin.strip_prefix("data:image/png;base64,").unwrap())?
    } else {
        fs::read(skin).await?
    };
    let client = reqwest::Client::new();

    for _n in 0..3 {
        let file_part = multipart::Part::bytes(file.clone())
            .file_name("skin.png")
            .mime_str("image/png")?;

        let form = multipart::Form::new()
            .part("file", file_part)
            .text("variant", arms.clone());
    
        let response = client
        .post("https://api.minecraftservices.com/minecraft/profile/skins")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .multipart(form)
        .send().await?;
        let statcode = response.status();
        if statcode.is_success() {
            let data = parse_skin_data(response.json().await?).await?;
            let _ = add_to_cache(user.id, data.user, HashMap::new(), data.head).await;
            result = true;
            break;
        }
    }
    Ok(result)
}

// Sets the players cape
pub async fn set_cape(capeid: String, token: String) -> crate::Result<bool> {
    let mut result: bool = false;
    let json: Value = json!({ "capeId": capeid });
    let client = reqwest::Client::new();

    for _n in 0..3 {
        let response = if capeid == "no cape" {
            client
            .delete("https://api.minecraftservices.com/minecraft/profile/capes/active")
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .send().await?
        } else {
            client
            .put("https://api.minecraftservices.com/minecraft/profile/capes/active")
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&json)
            .send().await?
        };
        let statcode = response.status();
        if statcode.is_success() {
            result = true;
            break;
        }
    }
    Ok(result)
}

// Returns cape info
pub async fn get_cape_data(cape: String, key: String) -> crate::Result<String> {
    let mut result: String = "no cape".to_string();
    if cape != "no cape" {
        let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
        let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
        )));
        let path = crate::State::get().await?
            .directories
            .caches_meta_dir()
            .await
            .join("skindata.json");

        let json: Cache = read_json(&path, &io_semaphore).await?;
        
        if key == "id" {
            result = json.capes.get(&cape).unwrap().id.clone();
        } else {
            result = json.capes.get(&cape).unwrap().url.clone();
        }
    }

    Ok(result)
}

// Returns users SkinCache
pub async fn get_user_skin_data(id: Uuid) -> crate::Result<SkinCache> {
    let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
    let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
    )));
    let path = crate::State::get().await?
        .directories
        .caches_meta_dir()
        .await
        .join("skindata.json");

    let json: Cache = read_json(&path, &io_semaphore).await?;
    let skin_data = json.users.get(&id).unwrap();
    Ok(skin_data.clone())
}

// call this on launcher launch
pub async fn cache_users_skins() -> crate::Result<bool> {
    let users: Vec<Credentials> = auth::users().await?;

    let mut user_map: HashMap<Uuid, SkinCache> = HashMap::new();
    let mut cape_map: HashMap<String, CapeCache> = HashMap::new();
    let mut head_map: HashMap<Uuid, String> = HashMap::new();
    let client = reqwest::Client::new();
    for user in users {
        let credential = auth::refresh(user.id).await?;
        let token = &credential.access_token;
        let response: Value = client
            .get("https://api.minecraftservices.com/minecraft/profile")
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .send().await?
            .json().await?;

        let data = parse_skin_data(response).await?;
        cape_map.extend(data.capes);
        user_map.insert(user.id, data.user);
        head_map.insert(user.id, data.head);
    }
    let cache = Cache {
        capes: cape_map,
        users: user_map,
        heads: head_map,
    };
    save_to_cache(cache).await
}

// Caches users SkinCache on new login
pub async fn cache_new_user_skin(user: Credentials) -> crate::Result<bool> {
    let credential = auth::refresh(user.id).await?;
    let token = &credential.access_token;
    let response: Value = reqwest::Client::new()
        .get("https://api.minecraftservices.com/minecraft/profile")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .send().await?
        .json().await?;
    let data = parse_skin_data(response).await?;
    add_to_cache(user.id, data.user, data.capes, data.head).await
}


pub async fn save_skin(user: Uuid, data: SkinCache, name: String, model: String, skinid: String) -> crate::Result<bool> {
    let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
    let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
    )));
    let path = crate::State::get().await?
        .directories
        .settings_dir
        .join("skin_manager.json");

    let mut cache: HashMap<Uuid, SkinSave> = 
        if let Ok(json) = read_json::<HashMap<Uuid, SkinSave>>(&path, &io_semaphore).await {
            json
        } else {
            HashMap::new()
        };
    
    let encoded_img = if data.skin.starts_with("data:image/png;base64,") {
        data.skin
    } else {
        format!("data:image/png;base64,{}", STANDARD.encode(fs::read(data.skin).await?.as_slice()))
    };
    let mut created = Utc::now();
    let mut id = Uuid::new_v4(); 
    if !skinid.is_empty() {
        id = Uuid::parse_str(&skinid)?;
        created = cache.get(&id).expect("SkinSave should exist, but doesn't").created;
    };
    let skin_cache = SkinSave {
        name,
        skin: encoded_img,
        cape: data.cape,
        arms: data.arms,
        created,
        updated: Utc::now(),
        model,
        user,
        id,
    };
    cache.insert(id, skin_cache);

    write(&path, &serde_json::to_vec(&cache)?, &io_semaphore).await?;
    Ok(true)
}

pub async fn delete_skin(id: Uuid) -> crate::Result<bool> {
    let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
    let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
    )));
    let path = crate::State::get().await?
        .directories
        .settings_dir
        .join("skin_manager.json");

    let mut cache = read_json::<HashMap<Uuid, SkinSave>>(&path, &io_semaphore)
        .await.expect("skin_manager.json should exist, but doesn't");
    cache.remove(&id);

    write(&path, &serde_json::to_vec(&cache)?, &io_semaphore).await?;
    Ok(true)
}

pub async fn get_skins() -> crate::Result<Vec<SkinSave>> {
    let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
    let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
    )));
    let path = crate::State::get().await?
        .directories
        .settings_dir
        .join("skin_manager.json");

    let cache: HashMap<Uuid, SkinSave> = 
    if let Ok(json) = read_json::<HashMap<Uuid, SkinSave>>(&path, &io_semaphore).await {
        json
    } else {
        HashMap::new()
    };
    let skins: Vec<SkinSave> = cache.into_values().collect();
    Ok(skins)
}

pub async fn get_mojang_launcher_path() -> crate::Result<PathBuf> {
    Ok(dirs::data_dir().unwrap().join(".minecraft"))
}

pub async fn get_mojang_launcher_names(path: PathBuf) -> crate::Result<Vec<MojangNames>> {
    let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
    let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
    )));
    let path = path.join("launcher_skins.json");
    let json = read_json::<HashMap<String, MojangSkins>>(&path, &io_semaphore).await.unwrap();
    let mut skin_names: Vec<MojangNames> = Vec::new();
    for skin in json.into_values() {
        skin_names.push(MojangNames { name: skin.name, selected: true });
    }
    Ok(skin_names)
}

pub async fn import_skin(name: String, path: PathBuf, user: Uuid) -> crate::Result<()> {
    let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
    let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
    )));
    let path = path.join("launcher_skins.json");
    let json = read_json::<HashMap<String, MojangSkins>>(&path, &io_semaphore).await.unwrap();
    for skin in json.into_values() {
        if skin.name == name {
            let data = SkinCache { 
                skin: skin.skin_image, 
                cape: "no cape".to_string(), 
                arms: if skin.slim {"slim".to_string()} else {"classic".to_string()}, 
                unlocked_capes: Vec::new() 
            };
            let _ = save_skin(user, data, name, skin.model_image, String::new()).await;
            break;
        } else {
            continue;
        }
    }
    Ok(())
}

async fn parse_skin_data(response: Value) -> crate::Result<Parsed> {
    let mut cape_map: HashMap<String, CapeCache> = HashMap::new();
    let mut cape_name: String = "no cape".to_string();
    let mut cape_list: Vec<String> = vec![cape_name.clone()];
    for i in 0..response["capes"].as_array().unwrap().len() {
        let key: String = response["capes"][i]["alias"].as_str().unwrap().to_string();
        let id: String = response["capes"][i]["id"].as_str().unwrap().to_string();
        let url: String = response["capes"][i]["url"].as_str().unwrap().to_string();
        let img = reqwest::Client::new()
        .get(url).send().await?
        .bytes().await?;
    let encoded_img = STANDARD.encode(&img);
    
        if response["capes"][i]["state"].as_str().unwrap() == "ACTIVE" {
            cape_name = key.clone();
        }
    
        let cape_cache = CapeCache {
            id,
            url: format!("data:image/png;base64,{encoded_img}")
        };
    
        cape_map.insert(key.clone(), cape_cache);
        cape_list.push(key);        
    }
    let skin_url = response["skins"][0]["url"].as_str().unwrap().to_string();
    let img = reqwest::Client::new()
        .get(skin_url).send().await?
        .bytes().await?;
    let encoded_img = STANDARD.encode(&img);
    
    let crop = image::imageops::crop_imm(
        &image::load_from_memory(&img)?, 8, 8, 8, 8).to_image();
    let mut buf: Vec<u8> = vec![];
    let _ = crop.write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Jpeg(100));
    let encoded_head =  STANDARD.encode(&buf);

    let skin_data: SkinCache = SkinCache {
        skin: format!("data:image/png;base64,{encoded_img}"),
        cape: cape_name,
        arms: response["skins"][0]["variant"].as_str().unwrap().to_lowercase().to_string(),
        unlocked_capes: cape_list
    };
    let data: Parsed = Parsed {
        capes: cape_map,
        user: skin_data,
        head: format!("data:image/jpg;base64,{encoded_head}")
    };
    Ok(data)
}

async fn add_to_cache(id: Uuid, skin: SkinCache, capes: HashMap<String, CapeCache>, head: String) -> crate::Result<bool> {
    let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
    let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
    )));
    let path = crate::State::get().await?
        .directories
        .caches_meta_dir()
        .await
        .join("skindata.json");

    let mut cache: Cache = 
        if let Ok(json) = read_json::<Cache>(&path, &io_semaphore).await {
            json
        } else {
            Cache {
                capes: HashMap::new(),
                users: HashMap::new(),
                heads: HashMap::new(),
            }
        };
    if !capes.is_empty() {
        cache.capes.extend(capes);
    }
    cache.users.insert(id, skin);
    cache.heads.insert(id, head);
    save_to_cache(cache).await
}

async fn save_to_cache(cache: Cache) -> crate::Result<bool> {
    let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
    let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
    )));
    let path = crate::State::get().await?
        .directories
        .caches_meta_dir()
        .await
        .join("skindata.json");

    write(&path, &serde_json::to_vec(&cache)?, &io_semaphore).await?;
    Ok(true)
}

#[derive(Serialize, Deserialize, Debug)]
struct Cache {
    capes: HashMap<String, CapeCache>,
    users: HashMap<Uuid, SkinCache>,
    heads: HashMap<Uuid, String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CapeCache {
    id: String,
    url: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkinCache {
    skin: String,
    cape: String,
    arms: String,
    unlocked_capes: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkinSave {
    name: String,
    user: Uuid,
    id: Uuid,
    skin: String,
    model: String,
    cape: String,
    arms: String,
    created: DateTime<Utc>,
    updated: DateTime<Utc>
}

#[derive(Serialize, Deserialize, Debug)]
struct Parsed {
    capes: HashMap<String, CapeCache>,
    user: SkinCache,
    head: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MojangSkins {
    created: DateTime<Utc>,
    id: String,
    model_image: String,
    name: String,
    skin_image: String,
    slim: bool,
    texture_id: String,
    updated: DateTime<Utc>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MojangNames {
    name: String,
    selected: bool
}