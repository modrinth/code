use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::{self, header, multipart};
use tokio;
use futures::future;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;
use crate::minecraft_auth;
use crate::data::DirectoryInfo;
use crate::state::Credentials;
use crate::process::Settings;
use crate::util::fetch::{read_json, write, IoSemaphore};

// Get image size
pub async fn check_image(path: String) -> crate::Result<bool> {
    let (w, h) = image::image_dimensions(path)?;
    Ok(w == 64 && h == 64)
}

pub async fn check_skin(skin: String, id: Uuid) -> crate::Result<bool> {
    let mut val: bool = true;
    let library: Vec<SkinSave> = get_skins().await?;
    for save in library {
        if save.user != id { continue; }
        if save.skin == skin {
            val = false;
            break;
        }
    }
    Ok(val)
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
    Ok(read_json::<Cache>(&path, &io_semaphore).await?.heads)
}

// Sets player's skin
pub async fn set_skin(skin: String, arms: String, creds: Credentials) -> crate::Result<bool> {
    let token = creds.access_token;
    let file: Vec<u8> = 
        if let Some(data) = skin.strip_prefix("data:image/png;base64,") {
            STANDARD.decode(data)?
        } else {
            tokio::fs::read(skin).await?
        };
    let client = reqwest::Client::new();

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
        let data = parse_skin_data(response.json().await?, creds.id).await?;
        add_to_cache(creds.id, data.user, HashMap::new(), data.head).await
    } else { Ok(false) }
}

// Sets the players cape
pub async fn set_cape(capeid: String, token: String) -> crate::Result<bool> {
    let json: Value = json!({ "capeId": capeid });
    let client = reqwest::Client::new();

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
    Ok(statcode.is_success())
}

// Returns cape info
pub async fn get_cape_data(cape: String, key: String) -> crate::Result<String> {
    if cape == "no cape" {
        Ok(cape)
    } else {
        let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
        let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
        )));
        let path = crate::State::get().await?
            .directories
            .caches_meta_dir()
            .await
            .join("skindata.json");

        if key == "id" { Ok(read_json::<Cache>(&path, &io_semaphore).await?.capes.get(&cape).unwrap().id.clone()) }
        else { Ok(read_json::<Cache>(&path, &io_semaphore).await?.capes.get(&cape).unwrap().url.clone()) }
    }
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

    let json = read_json::<Cache>(&path, &io_semaphore).await?;
    let skin_data = json.users.get(&id).expect("User data should exist, but doesn't");
    Ok(skin_data.clone())
}

// call this on launcher launch
pub async fn cache_users_skins() -> crate::Result<bool> {
    let users: Vec<Credentials> = minecraft_auth::users().await?;

    let mut user_map: HashMap<Uuid, SkinCache> = HashMap::new();
    let mut cape_map: HashMap<String, CapeData> = HashMap::new();
    let mut head_map: HashMap<Uuid, String> = HashMap::new();
    let client = reqwest::Client::new();

    let responses: Vec<Option<Parsed>> = future::join_all(users.into_iter().map(|user| {
        let client = &client;
        async move {
            let token = if user.expires < Utc::now() {
                minecraft_auth::refresh(user.id).await.unwrap().access_token
            } else {
                user.access_token
            };
            let response = client
                .get("https://api.minecraftservices.com/minecraft/profile")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .send().await.unwrap();

            if response.status().is_success() {
                parse_skin_data(response.json().await.unwrap(), user.id).await.ok()
            } else {
                None
            }   
        }
    })).await;

    for data in responses {
        if data.is_none() { continue };
        let data = data.unwrap();
        cape_map.extend(data.capes);
        user_map.insert(data.id, data.user);
        head_map.insert(data.id, data.head);
    }
    let cache = Cache {
        capes: cape_map,
        users: user_map,
        heads: head_map,
    };
    save_to_cache(cache).await
}

// Caches users SkinCache on new login
pub async fn cache_new_user_skin(creds: Credentials) -> crate::Result<bool> {
    let token = if creds.expires < Utc::now() {
        minecraft_auth::refresh(creds.id).await?.access_token
    } else {
        creds.access_token
    };
    let response = reqwest::Client::new()
        .get("https://api.minecraftservices.com/minecraft/profile")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .send().await?;
    if response.status().is_success() {
        let data = parse_skin_data(response.json().await?, creds.id).await?;
        add_to_cache(creds.id, data.user, data.capes, data.head).await
    } else {
        Ok(false)
    }
}

// Saves skin data to the skin manager
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
    
    let encoded_img =
        if data.skin.starts_with("data:image/png;base64,") {
            data.skin
        } else {
            format!("data:image/png;base64,{}", STANDARD.encode(tokio::fs::read(data.skin).await?.as_slice()))
        };

    let id = if skinid.is_empty() {
        Uuid::new_v4()
    } else {
        Uuid::parse_str(&skinid)?
    };
    let created = if skinid.is_empty() {
        Utc::now()
    } else {
        cache.get(&id).expect("SkinSave should exist, but doesn't").created
    };
    let order = if skinid.is_empty() {
        HashMap::from([(user, 0)])
    } else {
        cache.get(&id).expect("SkinSave should exist, but doesn't").order.clone()
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
        order,
    };
    cache.insert(id, skin_cache);

    write(&path, &serde_json::to_vec(&cache)?, &io_semaphore).await?;
    Ok(true)
}

pub async fn update_skins(saves: Vec<SkinSave>) -> crate::Result<()> {
    let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
    let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
    )));
    let path = crate::State::get().await?
        .directories
        .settings_dir
        .join("skin_manager.json");

    let mut cache = HashMap::new();
    for skin in saves {
        cache.insert(skin.id, skin);
    }
    write(&path, &serde_json::to_vec(&cache)?, &io_semaphore).await?;
    Ok(())
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

    if let Ok(json) = read_json::<HashMap<Uuid, SkinSave>>(&path, &io_semaphore).await {
        Ok(json.into_values().collect())
    } else {
        Ok(Vec::new())
    }
}

// Gets list of skins to import
pub async fn get_launcher_names(path: PathBuf, installer: String) -> crate::Result<Vec<MojangNames>> {
    let json = import_json(path, installer).await;
    if json.is_ok() {
        let mut skin_names = vec![];
        for skin in json?.into_values() {
            let name = if skin.name.is_some() { skin.name.unwrap() }
                else { "Untitled".to_string() };
            skin_names.push(MojangNames { name, selected: true, id: skin.id })
        }
        Ok(skin_names)
    } else { Ok(vec![]) }
}

pub async fn import_skin(id: String, path: PathBuf, installer: String) -> crate::Result<SkinCache> {
    let mut data = SkinCache { skin: "".to_string(), cape: "no cape".to_string(), arms: "".to_string(), unlocked_capes: vec![] };
    let json = import_json(path, installer).await;

    if json.is_ok() {
        for skin in json?.into_values() {
            if skin.id == id {
                data.skin = skin.skin_image;
                data.arms = if skin.slim {"slim".to_string()} else {"classic".to_string()};
                if skin.cape_id.is_some() {
                    let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
                    let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
                settings.max_concurrent_writes,
                    )));
                    let path = crate::State::get().await?
                        .directories.caches_meta_dir().await
                        .join("skindata.json");
                    let capes = read_json::<Cache>(&path, &io_semaphore).await?.capes;
                    for (key, value) in capes {
                        if value.id == skin.cape_id.clone().unwrap() { data.cape = key }
                    }
                }
                break;
            }
        };
    }
    Ok(data)
}

async fn import_json(path: PathBuf, installer: String) -> crate::Result<HashMap<String, MojangSkins>> {
    let settings = Settings::init(&DirectoryInfo::get_initial_settings_file()?).await?;
    let io_semaphore: IoSemaphore = IoSemaphore(RwLock::new(Semaphore::new(
        settings.max_concurrent_writes,
    )));
    let path = if installer == "Mojang" { path.join("launcher_custom_skins.json") }
        else { path.join("Install").join("launcher_skins.json") };

    if installer == "Mojang" { Ok(read_json::<Mojang>(&path, &io_semaphore).await?.custom_skins) }
        else { Ok(read_json::<HashMap<String, MojangSkins>>(&path, &io_semaphore).await?) }
}

async fn parse_skin_data(response: Value, id: Uuid) -> crate::Result<Parsed> {
    let mut cape_map: HashMap<String, CapeData> = HashMap::new();
    let mut cape_name: String = "no cape".to_string();
    let mut cape_list: Vec<String> = vec![cape_name.clone()];
    for i in 0..response["capes"].as_array().unwrap().len() {
        let key: String = response["capes"][i]["alias"].as_str().unwrap().to_string();
        let cape_id: String = response["capes"][i]["id"].as_str().unwrap().to_string();
        let url: String = response["capes"][i]["url"].as_str().unwrap().to_string();
        let img = reqwest::Client::new()
        .get(url).send().await?
        .bytes().await?;
        let encoded_img = STANDARD.encode(&img);
    
        if response["capes"][i]["state"].as_str().unwrap() == "ACTIVE" {
            cape_name = key.clone();
        }

        let cape_cache = CapeData {
            id: cape_id,
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
        head: format!("data:image/jpg;base64,{encoded_head}"),
        id,
    };
    Ok(data)
}

async fn add_to_cache(id: Uuid, skin: SkinCache, capes: HashMap<String, CapeData>, head: String) -> crate::Result<bool> {
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
    capes: HashMap<String, CapeData>,
    users: HashMap<Uuid, SkinCache>,
    heads: HashMap<Uuid, String>
}

#[derive(Serialize, Deserialize, Debug)]
struct CapeData {
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
    order: HashMap<Uuid, u8>,
    skin: String,
    model: String,
    cape: String,
    arms: String,
    created: DateTime<Utc>,
    updated: DateTime<Utc>
}

#[derive(Serialize, Deserialize, Debug)]
struct Parsed {
    capes: HashMap<String, CapeData>,
    user: SkinCache,
    head: String,
    id: Uuid
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mojang {
    custom_skins: HashMap<String, MojangSkins>,
    version: u8
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MojangSkins {
    created: DateTime<Utc>,
    id: String,
    model_image: String,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default
    )]
    name: Option<String>,
    skin_image: String,
    slim: bool,
    updated: DateTime<Utc>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default
    )]
    texture_id: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default
    )]
    cape_id: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MojangNames {
    name: String,
    id: String,
    selected: bool
}