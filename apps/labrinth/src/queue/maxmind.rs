use flate2::read::GzDecoder;
use maxminddb::geoip2::Country;
use std::io::{Cursor, Read};
use std::net::Ipv6Addr;
use std::time::Duration;
use log::{info, warn};
use reqwest::Client;
use tar::Archive;
use tokio::sync::RwLock;

pub struct MaxMindIndexer {
    pub reader: RwLock<Option<maxminddb::Reader<Vec<u8>>>>,
}

impl MaxMindIndexer {
    pub async fn new() -> Result<Self, reqwest::Error> {
        let reader = MaxMindIndexer::inner_index(false).await.ok().flatten();

        Ok(MaxMindIndexer {
            reader: RwLock::new(reader),
        })
    }

    pub async fn index(&self) -> Result<(), reqwest::Error> {
        let reader = MaxMindIndexer::inner_index(false).await?;

        if let Some(reader) = reader {
            let mut reader_new = self.reader.write().await;
            *reader_new = Some(reader);
        }

        Ok(())
    }

    async fn inner_index(
        should_panic: bool,
    ) -> Result<Option<maxminddb::Reader<Vec<u8>>>, reqwest::Error> {
        let license_key = dotenvy::var("MAXMIND_LICENSE_KEY").unwrap();
        let url = format!(
            "https://download.maxmind.com/app/geoip_download?edition_id=GeoLite2-Country&license_key={}&suffix=tar.gz",
            license_key
        );
        // 创建一个 reqwest Client 并设置超时时间
        let client = Client::builder()
            .timeout(Duration::from_secs(10)) // 设置超时时间为 30 秒
            .build()?;

        // 使用自定义的 client 发起请求
        let mut response = client.get(&url).send().await?;
        if !response.status().is_success() {
            info!("maxmind官方下载失败 {}", response.status());
            let url = format!(
                "https://download.maxmind.com/app/geoip_download?edition_id=GeoLite2-Country&license_key={}&suffix=tar.gz",
                license_key
            );

            response = reqwest::get(&url).await?;
            if !response.status().is_success() {
                info!("maxmind备用下载失败 {}", response.status());
            }else {
                info!("maxmind已使用备用下载源 {}", response.status());

            }
        }
        if !response.status().is_success() {
            info!("maxmind全部下载失败 {}", response.status());

            return Ok(None);
        }
        info!("Downloaded maxmind database.");
        let tarfile = GzDecoder::new(Cursor::new(response.bytes().await?.as_ref().to_vec()));
        let mut archive = Archive::new(tarfile);

        if let Ok(entries) = archive.entries() {
            for mut file in entries.flatten() {
                if let Ok(path) = file.header().path() {
                    if path.extension().and_then(|x| x.to_str()) == Some("mmdb")
                    {
                        let mut buf = Vec::new();
                        file.read_to_end(&mut buf).unwrap();

                        let reader =
                            maxminddb::Reader::from_source(buf).unwrap();

                        return Ok(Some(reader));
                    }
                }
            }
        }

        if should_panic {
            panic!("Unable to download maxmind database- did you get a license key?")
        } else {
            warn!("Unable to download maxmind database.");

            Ok(None)
        }
    }

    pub async fn query(&self, ip: Ipv6Addr) -> Option<String> {
        let maxmind = self.reader.read().await;

        if let Some(ref maxmind) = *maxmind {
            maxmind.lookup::<Country>(ip.into()).ok().and_then(|x| {
                x.country.and_then(|x| x.iso_code.map(|x| x.to_string()))
            })
        } else {
            None
        }
    }
}
