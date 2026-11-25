import { defineMessages } from '@modrinth/ui'

export const compatibilityMessages = defineMessages({
	title: {
		id: 'project.settings.server.compatibility.title',
		defaultMessage: 'Server compatibility',
	},
	description: {
		id: 'project.settings.server.compatibility.description',
		defaultMessage:
			'Select whether your server is vanilla or modded and which versions it supports. The Modrinth App uses this when a player joins.',
	},
	vanillaServer: {
		id: 'project.settings.server.compatibility.vanilla-server',
		defaultMessage: 'Vanilla server',
	},
	publishedModpack: {
		id: 'project.settings.server.compatibility.published-modpack',
		defaultMessage: 'Published modpack',
	},
	customModpack: {
		id: 'project.settings.server.compatibility.custom-modpack',
		defaultMessage: 'Custom modpack',
	},
	switchType: {
		id: 'project.settings.server.compatibility.switch-type',
		defaultMessage: 'Switch type',
	},
	setCompatibility: {
		id: 'project.settings.server.compatibility.set-compatibility',
		defaultMessage: 'Set compatibility',
	},
	recommendedVersion: {
		id: 'project.settings.server.compatibility.recommended-version',
		defaultMessage: 'Recommended version',
	},
	supportedVersions: {
		id: 'project.settings.server.compatibility.supported-versions',
		defaultMessage: 'Supported versions',
	},
	requiredModpack: {
		id: 'project.settings.server.compatibility.required-modpack',
		defaultMessage: 'Required modpack',
	},
	requiredVersion: {
		id: 'project.settings.server.compatibility.required-version',
		defaultMessage: 'Required version',
	},
	versionNumber: {
		id: 'project.settings.server.compatibility.version-number',
		defaultMessage: 'v{version}',
	},
	dataLossWarning: {
		id: 'project.settings.server.compatibility.data-loss-warning',
		defaultMessage: 'Data loss warning',
	},
	dataLossDescription: {
		id: 'project.settings.server.compatibility.data-loss-description',
		defaultMessage:
			'Changing the compatibility type will reset your previous compatibility settings and redistribute the new settings to users in the Modrinth App.',
	},
	selectType: {
		id: 'project.settings.server.compatibility.select-type',
		defaultMessage: 'Select compatibility type',
	},
	vanillaDescription: {
		id: 'project.settings.server.compatibility.vanilla-description',
		defaultMessage: 'A vanilla server with no mods.',
	},
	publishedModpackDescription: {
		id: 'project.settings.server.compatibility.published-modpack-description',
		defaultMessage: 'A modded server using a published modpack.',
	},
	customModpackDescription: {
		id: 'project.settings.server.compatibility.custom-modpack-description',
		defaultMessage: 'A modded server using a custom modpack.',
	},
	customModpackNotice: {
		id: 'project.settings.server.compatibility.custom-modpack-notice',
		defaultMessage:
			'Servers with custom modpacks should not be uploaded as separate modpack projects.',
	},
	supportedMcVersions: {
		id: 'project.settings.server.compatibility.supported-mc-versions',
		defaultMessage: 'Supported MC versions',
	},
	recommendedMcVersion: {
		id: 'project.settings.server.compatibility.recommended-mc-version',
		defaultMessage: 'Recommended MC version',
	},
	selectVersion: {
		id: 'project.settings.server.compatibility.select-version',
		defaultMessage: 'Select version',
	},
	selectSupportedVersions: {
		id: 'project.settings.server.compatibility.select-supported-versions',
		defaultMessage: 'Set supported versions before selecting the recommended version',
	},
	recommendedVersionHelp: {
		id: 'project.settings.server.compatibility.recommended-version-help',
		defaultMessage:
			'Players joining the server from the Modrinth App will connect using this version.',
	},
	selectModpack: {
		id: 'project.settings.server.compatibility.select-modpack',
		defaultMessage: 'Select modpack',
	},
	project: {
		id: 'project.settings.server.compatibility.project',
		defaultMessage: 'Project',
	},
	searchModpacks: {
		id: 'project.settings.server.compatibility.search-modpacks',
		defaultMessage: 'Search by name or paste ID...',
	},
	noResults: {
		id: 'project.settings.server.compatibility.no-results',
		defaultMessage: 'No results found',
	},
	version: {
		id: 'project.settings.server.compatibility.version',
		defaultMessage: 'Version',
	},
	searchVersions: {
		id: 'project.settings.server.compatibility.search-versions',
		defaultMessage: 'Search versions...',
	},
	gameVersion: {
		id: 'project.settings.server.compatibility.game-version',
		defaultMessage: 'Game version',
	},
	failedToLoadVersions: {
		id: 'project.settings.server.compatibility.failed-to-load-versions',
		defaultMessage: 'Failed to load versions',
	},
	uploadCustomModpack: {
		id: 'project.settings.server.compatibility.upload-custom-modpack',
		defaultMessage: 'Upload custom modpack',
	},
	dropModpack: {
		id: 'project.settings.server.compatibility.drop-modpack',
		defaultMessage: 'Drag and drop your .mrpack file',
	},
	browseFiles: {
		id: 'project.settings.server.compatibility.browse-files',
		defaultMessage: 'Or click to browse',
	},
	replaceFile: {
		id: 'project.settings.server.compatibility.replace-file',
		defaultMessage: 'Replace file',
	},
	licensePermission: {
		id: 'project.settings.server.compatibility.license-permission',
		defaultMessage:
			'Do you have the appropriate licenses to redistribute all content in this Modpack? <permissions-link>Learn more</permissions-link>',
	},
	compatibilityType: {
		id: 'project.settings.server.compatibility.compatibility-type',
		defaultMessage: 'Compatibility type',
	},
	vanillaVersions: {
		id: 'project.settings.server.compatibility.vanilla-versions',
		defaultMessage: 'Vanilla versions',
	},
	changeType: {
		id: 'project.settings.server.compatibility.change-type',
		defaultMessage: 'Change type',
	},
	updating: {
		id: 'project.settings.server.compatibility.updating',
		defaultMessage: 'Updating…',
	},
	saving: {
		id: 'project.settings.server.compatibility.saving',
		defaultMessage: 'Saving…',
	},
	uploading: {
		id: 'project.settings.server.compatibility.uploading',
		defaultMessage: 'Uploading {progress, number, percent}',
	},
	saveFailed: {
		id: 'project.settings.server.compatibility.save-failed',
		defaultMessage: 'Failed to save server compatibility',
	},
	noProject: {
		id: 'project.settings.server.compatibility.no-project',
		defaultMessage: 'No server project selected',
	},
})
