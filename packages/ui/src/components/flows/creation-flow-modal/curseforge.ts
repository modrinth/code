import { defineMessages } from '#ui/composables/i18n'

export const curseforgeMessages = defineMessages({
	title: {
		id: 'creation-flow.curseforge.title',
		defaultMessage: 'Upload a CurseForge modpack',
	},
	description: {
		id: 'creation-flow.curseforge.description',
		defaultMessage: 'Use a modpack that isn’t available on Modrinth.',
	},
	linkLabel: {
		id: 'creation-flow.curseforge.link-label',
		defaultMessage: 'CurseForge modpack link',
	},
	linkDescription: {
		id: 'creation-flow.curseforge.link-description',
		defaultMessage: 'You can use a link to a modpack or a specific modpack version.',
	},
	invalidLink: {
		id: 'creation-flow.curseforge.invalid-link',
		defaultMessage: 'Enter a CurseForge Minecraft modpack or modpack version link.',
	},
	orUpload: {
		id: 'creation-flow.curseforge.or-upload',
		defaultMessage: 'Or upload it yourself',
	},
	fileLabel: {
		id: 'creation-flow.curseforge.file-label',
		defaultMessage: 'CurseForge Modpack file',
	},
	selectedFileLabel: {
		id: 'creation-flow.curseforge.selected-file-label',
		defaultMessage: 'Modpack file',
	},
	fileDescription: {
		id: 'creation-flow.curseforge.file-description',
		defaultMessage:
			'You’ll need the regular modpack file, and can optionally add a server pack after if we can’t find one.',
	},
	serverPackLabel: {
		id: 'creation-flow.curseforge.server-pack-label',
		defaultMessage: 'Server pack <optional>(optional)</optional>',
	},
	serverPackDescription: {
		id: 'creation-flow.curseforge.server-pack-description',
		defaultMessage:
			'You can optionally add a server pack to make it easier to determine what needs to be installed on the server.',
	},
	uploadPrompt: {
		id: 'creation-flow.curseforge.upload-prompt',
		defaultMessage: 'Drag and drop file or click to browse',
	},
	removeFile: {
		id: 'creation-flow.curseforge.remove-file',
		defaultMessage: 'Remove {filename}',
	},
	installationUnavailable: {
		id: 'creation-flow.curseforge.installation-unavailable',
		defaultMessage: 'CurseForge installation is not available yet.',
	},
})

export function isCurseForgeModpackUrl(value: string): boolean {
	try {
		const url = new URL(value.trim())
		return (
			url.protocol === 'https:' &&
			(url.hostname === 'curseforge.com' || url.hostname === 'www.curseforge.com') &&
			!url.username &&
			!url.password &&
			!url.port &&
			/^\/(?:minecraft\/)?modpacks\/[^/]+(?:\/files\/\d+)?\/?$/.test(url.pathname)
		)
	} catch {
		return false
	}
}
