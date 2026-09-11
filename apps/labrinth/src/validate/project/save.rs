use super::{ProjectNag, ProjectNagKind};

/// Selects the fields affected by a save, including cross-field validation dependencies.
#[derive(Clone, Default)]
pub struct ProjectSaveValidation {
	pub all: bool,
	pub name: bool,
	pub summary: bool,
	pub description: bool,
	pub license: bool,
	pub links: bool,
	pub tags: bool,
	pub region: bool,
	pub languages: bool,
	pub java_address: bool,
	pub language_requirements: bool,
	pub compatibility: bool,
	pub gallery_url: Option<String>,
	pub disclosures: bool,
}

impl ProjectSaveValidation {
	pub fn includes(&self, nag: &ProjectNag, gallery_urls: &[&str]) -> bool {
		if self.all {
			return true;
		}
		use ProjectNagKind::*;
		match nag.kind {
			ProjectNameSlur
			| ProjectNameProfanity
			| ProjectNameNonStandardText
			| ProjectNameVersion
			| MinecraftTitleClause => self.name,
			ProjectSummaryMatchesTitle => self.name || self.summary,
			ProjectSummaryNonEnglish => {
				self.summary
					|| self.languages
					|| self.tags || self.language_requirements
			}
			ProjectSummarySlur
			| ProjectSummaryProfanity
			| ProjectSummaryNonStandardText
			| SummaryTooShort
			| ProjectSummarySpam
			| SummarySpecialFormatting
			| ProjectSummaryLinks => self.summary,
			ProjectDescriptionMatchesSummary => {
				self.description || self.summary
			}
			ProjectDescriptionNonEnglish => {
				self.description
					|| self.languages
					|| self.tags || self.language_requirements
			}
			ProjectDescriptionSlur
			| ProjectDescriptionProfanity
			| ProjectDescriptionNonStandardText
			| AddDescription
			| DescriptionTooShort
			| ProjectDescriptionSpam
			| LongHeaders
			| DescriptionEndsWithHeader
			| AdjacentHeaders
			| MissingAltText => self.description,
			SelectLicense | AddCustomLicenseDetails | InvalidLicenseUrl => {
				self.license
			}
			GplLicenseSourceRequired => self.license || self.links,
			LinkValidation => match nag.details["field"].as_str() {
				Some("description") => self.description,
				Some("license") => {
					self.license
						|| (self.links && nag.details["reason"] == "duplicate")
				}
				_ => {
					self.links
						|| (self.license
							&& nag.details["reason"] == "duplicate")
				}
			},
			AddLinks | AddLinksServer => self.links,
			SelectTags
			| TooManyTags
			| TooManyTagsServer
			| MultipleResolutionTags
			| AllTagsSelected => self.tags,
			SelectCountry => self.region,
			AllLanguages | TooManyLanguages | SelectLanguage => self.languages,
			AddJavaAddress => self.java_address,
			SelectCompatibility => self.compatibility,
			GalleryTextSlur | GalleryTextProfanity | GalleryTextNonStandard => {
				self.gallery_url.as_deref().is_some_and(|url| {
					nag.details["gallery_index"]
						.as_u64()
						.and_then(|index| gallery_urls.get(index as usize))
						.is_some_and(|candidate| *candidate == url)
				})
			}
			DisclosuresSpecialFormatting | CheckDisclosures => self.disclosures,
			UploadGalleryImage => self.tags,
			AddIcon | FeatureGalleryImage | ReviewPermissions
			| UploadVersion | SelectEnvironment | ModeratorFeedback => false,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::validate::project::ProjectNagSeverity;

	#[test]
	fn description_saves_do_not_require_unrelated_draft_fields() {
		let scope = ProjectSaveValidation {
			description: true,
			..Default::default()
		};
		for kind in [
			ProjectNagKind::DescriptionTooShort,
			ProjectNagKind::LongHeaders,
		] {
			assert!(scope.includes(
				&ProjectNag::new(kind, ProjectNagSeverity::Required),
				&[]
			));
		}
		for kind in [
			ProjectNagKind::AddIcon,
			ProjectNagKind::UploadVersion,
			ProjectNagKind::SelectLicense,
		] {
			assert!(!scope.includes(
				&ProjectNag::new(kind, ProjectNagSeverity::Required),
				&[]
			));
		}
	}

	#[test]
	fn summary_saves_check_similarity_dependencies_without_requiring_a_description()
	 {
		let scope = ProjectSaveValidation {
			summary: true,
			..Default::default()
		};
		assert!(scope.includes(
			&ProjectNag::new(
				ProjectNagKind::ProjectDescriptionMatchesSummary,
				ProjectNagSeverity::Required
			),
			&[]
		));
		assert!(!scope.includes(
			&ProjectNag::new(
				ProjectNagKind::AddDescription,
				ProjectNagSeverity::Required
			),
			&[]
		));
	}

	#[test]
	fn gallery_saves_only_validate_the_selected_image() {
		let scope = ProjectSaveValidation {
			gallery_url: Some("new.png".into()),
			..Default::default()
		};
		let nag = ProjectNag::new(
			ProjectNagKind::GalleryTextSlur,
			ProjectNagSeverity::Required,
		)
		.with_details(serde_json::json!({ "gallery_index": 1 }));
		assert!(scope.includes(&nag, &["old.png", "new.png"]));
		assert!(!scope.includes(&nag, &["new.png", "old.png"]));
	}
}
