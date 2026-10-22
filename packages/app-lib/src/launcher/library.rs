use crate::util::platform::OperatingSystemExt;
use minecraft_meta_manifest::{
    minecraft::{Library, OperatingSystem},
    util::{MavenCoordinate, Sha1},
};
use url::Url;

pub(crate) trait LibraryExt {
    /// Native classification does not depend on classifier download metadata.
    fn native_classifier(&self, java_arch: &str) -> Option<String>;

    /// Resolves explicit native metadata or derives its Maven classifier URL.
    fn native_download(
        &self,
        java_arch: &str,
    ) -> anyhow::Result<Option<(Url, Option<Sha1>, Option<u32>)>>;
}

impl LibraryExt for Library {
    /// Native classification does not depend on classifier download metadata.
    fn native_classifier(&self, java_arch: &str) -> Option<String> {
        let natives = self.natives.as_ref()?;
        let os = OperatingSystem::native_arch(java_arch);
        let classifier = natives.get(&os)?;
        let width = if matches!(java_arch, "x86" | "i386" | "i686" | "arm") {
            "32"
        } else {
            "64"
        };
        Some(classifier.replace("${arch}", width))
    }

    /// Resolves explicit native metadata or derives its Maven classifier URL.
    fn native_download(
        &self,
        java_arch: &str,
    ) -> anyhow::Result<Option<(Url, Option<Sha1>, Option<u32>)>> {
        let Some(classifier) = self.native_classifier(java_arch) else {
            return Ok(None);
        };
        let explicit = self
            .downloads
            .as_ref()
            .and_then(|downloads| downloads.classifiers.as_ref())
            .and_then(|classifiers| classifiers.get(&classifier));
        if let Some(download) = explicit
            && let Some(url) = &download.url
        {
            return Ok(Some((
                url.clone(),
                Some(download.sha1),
                Some(download.size),
            )));
        }
        let mut coordinate: MavenCoordinate = self.name.parse()?;
        coordinate.classifier = Some(classifier.parse()?);
        let path = coordinate.to_maven_path();
        let repository = self
            .url
            .as_deref()
            .filter(|url| !url.is_empty())
            .unwrap_or("https://libraries.minecraft.net/");
        let url = Url::parse(&format!(
            "{}/{relative}",
            repository.trim_end_matches('/'),
            relative = &path["maven/".len()..]
        ))?;
        Ok(Some((
            url,
            explicit.map(|download| download.sha1),
            explicit.map(|download| download.size),
        )))
    }
}
