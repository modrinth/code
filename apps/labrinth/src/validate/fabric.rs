use crate::database::models::legacy_loader_fields::MinecraftGameVersion;
use crate::models::projects::Loader;
use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
    validate_pack_formats,
};
use std::io::Cursor;
use zip::ZipArchive;

pub struct FabricValidator;

const FABRIC_18W43B_TIMESTAMP: i64 = 1_540_393_350;
const LEGACY_FABRIC_1_3_TIMESTAMP: i64 = 1_343_253_600;
const LEGACY_FABRIC_1_13_2_TIMESTAMP: i64 = 1_540_208_467;

impl super::Validator for FabricValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["jar"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["fabric"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn ensure_required_loaders(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
        loaders: &[Loader],
    ) -> Result<(), ValidationError> {
        if loaders.iter().any(|loader| loader.0 == "bta-babric") {
            return Ok(());
        }

        let manifest = match archive.by_name("fabric.mod.json") {
            Ok(manifest) => manifest,
            Err(zip::result::ZipError::FileNotFound) => return Ok(()),
            Err(error) => return Err(error.into()),
        };

        let Ok(metadata) =
            serde_json::from_reader::<_, serde_json::Value>(manifest)
        else {
            return Ok(());
        };

        if has_bta_loader_dependency(&metadata["depends"]["fabricloader"]) {
            return Err(ValidationError::InvalidInput(
				"files whose `fabric.mod.json` requires a `fabricloader` version containing `-bta.` must include the `bta-babric` loader".into(),
			));
        }
        Ok(())
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("fabric.mod.json").is_err() {
            return Ok(ValidationResult::Warning(
                "No fabric.mod.json present for Fabric file.",
            ));
        }

        Ok(validate_pack_formats(archive))
    }
}

fn has_bta_loader_dependency(requirement: &serde_json::Value) -> bool {
    match requirement {
        serde_json::Value::String(range) => range.contains("-bta."),
        serde_json::Value::Array(ranges) => {
            ranges.iter().any(has_bta_loader_dependency)
        }
        _ => false,
    }
}

pub(super) fn validate_game_versions(
    loaders: &[Loader],
    game_versions: &[MinecraftGameVersion],
) -> Result<(), ValidationError> {
    for loader in loaders {
        let valid = !game_versions.is_empty()
            && game_versions.iter().all(|version| match loader.0.as_str() {
                "fabric" => {
                    version.created.timestamp() >= FABRIC_18W43B_TIMESTAMP
                }
                "legacy-fabric" => (LEGACY_FABRIC_1_3_TIMESTAMP
                    ..=LEGACY_FABRIC_1_13_2_TIMESTAMP)
                    .contains(&version.created.timestamp()),
                "babric" | "bta-babric" => version.version == "b1.7.3",
                _ => true,
            });
        let requirement = match loader.0.as_str() {
            "fabric" => "`18w43b` or later",
            "legacy-fabric" => "between `1.3` and `1.13.2`, inclusive",
            "babric" | "bta-babric" => "`b1.7.3` only",
            _ => continue,
        };
        if !valid {
            return Err(ValidationError::InvalidInput(
                format!(
                    "the `{}` loader requires minecraft versions {requirement}",
                    loader.0,
                )
                .into(),
            ));
        }
    }
    Ok(())
}
