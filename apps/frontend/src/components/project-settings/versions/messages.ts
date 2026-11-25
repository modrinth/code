import { defineMessages } from '@modrinth/ui'

export const versionActionMessages = defineMessages({
	create: {
		id: 'project.settings.versions.empty.create',
		defaultMessage: 'Create version',
	},
})

export const messages = defineMessages({
	withheldVersionsWarningTitle: {
		id: 'project.versions.withheld-versions-warning.title',
		defaultMessage:
			'{count, plural, one {Version {version_name}} other {Versions}} withheld due to unknown embedded content',
	},
	withheldVersionsWarningDescription: {
		id: 'project.versions.withheld-versions-warning.description',
		defaultMessage:
			'{count, plural, one {This version is} other {These versions are}} currently withheld and not publicly listed. Please provide proof that you have permission to redistribute certain files included in the modpack {count, plural, one {version} other {versions}}.',
	},
	deleteVersionConfirmTitle: {
		id: 'project.versions.delete-confirm.title',
		defaultMessage: 'Are you sure you want to delete this version?',
	},
	deleteVersionConfirmDescription: {
		id: 'project.versions.delete-confirm.description',
		defaultMessage: 'This will remove this version forever (like really forever).',
	},
	deleteButton: {
		id: 'project.versions.delete-button',
		defaultMessage: 'Delete',
	},
	editFilesOption: {
		id: 'project.versions.edit-files-option',
		defaultMessage: 'Edit files',
	},
	editDetailsOption: {
		id: 'project.versions.edit-details-option',
		defaultMessage: 'Edit details',
	},
	editMetadataOption: {
		id: 'project.versions.edit-metadata-option',
		defaultMessage: 'Edit metadata',
	},
	openInNewTabOption: {
		id: 'project.versions.open-in-new-tab-option',
		defaultMessage: 'Open in new tab',
	},
	copyLinkOption: {
		id: 'project.versions.copy-link-option',
		defaultMessage: 'Copy link',
	},
	shareOption: {
		id: 'project.versions.share-option',
		defaultMessage: 'Share',
	},
	copyIdOption: {
		id: 'project.versions.copy-id-option',
		defaultMessage: 'Copy ID',
	},
	copyMavenCoordinatesOption: {
		id: 'project.versions.copy-maven-coordinates-option',
		defaultMessage: 'Copy Maven coordinates',
	},
	versionDeletedTitle: {
		id: 'project.versions.version-deleted-title',
		defaultMessage: 'Version deleted',
	},
	versionDeletedText: {
		id: 'project.versions.version-deleted-text',
		defaultMessage: 'The version has been successfully deleted.',
	},
	emptyTitle: {
		id: 'project.settings.versions.empty.title',
		defaultMessage: 'No versions created',
	},
	emptyDescription: {
		id: 'project.settings.versions.empty.description',
		defaultMessage: 'Create your first project version.',
	},
})
