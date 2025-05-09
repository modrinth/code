use flate2::read::GzDecoder;
use maxminddb::geoip2::Country;
use std::io::{Cursor, Read};
use std::net::Ipv6Addr;
use tar::Archive;
use tokio::sync::RwLock;
use tracing::warn;

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
        let response = reqwest::get(
            format!(
                "https://download.maxmind.com/app/geoip_download?edition_id=GeoLite2-Country&license_key={}&suffix=tar.gz",
                dotenvy::var("MAXMIND_LICENSE_KEY").unwrap()
            )
        ).await?.bytes().await.unwrap().to_vec();

        let tarfile = GzDecoder::new(Cursor::new(response));
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
            panic!(
                "Unable to download maxmind database- did you get a license key?"
            )
        } else {
            warn!("Unable to download maxmind database.");

            Ok(None)
        }
    }

    pub async fn query(&self, ip: Ipv6Addr) -> Option<String> {
        let maxmind = self.reader.read().await;

        if let Some(ref maxmind) = *maxmind {
            maxmind
                .lookup::<Country>(ip.into())
                .ok()
                .flatten()
                .and_then(|x| {
                    x.country.and_then(|x| x.iso_code.map(|x| x.to_string()))
                })
        } else {
            None
        }
    }
}
