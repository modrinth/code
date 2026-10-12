import type {
	AbstractModrinthClient,
	Archon,
	UploadHandle,
	UploadProgress,
} from '@modrinth/api-client'

import { defineMessages } from '#ui/composables/i18n'

import type { CreationFlowContextValue } from './creation-flow-context'

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

/**
 * Installs the CurseForge modpack selected in the creation flow through a FlameAnvil session,
 * either from the entered link or from the uploaded client and optional server packs.
 *
 * Any session left over from an abandoned attempt is cancelled first, and the new session is
 * cancelled if anything fails before the installation is queued.
 *
 * @returns The installation progress key Kyros publishes for the queued installation.
 */
export async function installCurseForgeModpack(
	client: AbstractModrinthClient,
	worldId: string,
	ctx: CreationFlowContextValue,
	options: {
		softOverride: boolean
		auth?: Archon.Servers.v0.JWTAuth
		onUploadProgress?: (progress: UploadProgress) => void
	},
): Promise<Archon.Websocket.v0.InstallProgressCurseForgeModpackKey> {
	const flameanvil = client.kyros.flameanvil_v1
	const { auth } = options
	const modpackFile = ctx.curseforgeModpackFile.value
	const serverPackFile = ctx.curseforgeServerPackFile.value
	const link = ctx.curseforgeUrl.value.trim()
	if (!modpackFile && !isCurseForgeModpackUrl(link)) {
		throw new Error('A CurseForge modpack link or file is required')
	}

	const filename = modpackFile
		? modpackFile.name
		: (await flameanvil.queryLink(worldId, { link }, { auth })).file.file_name

	const { session: staleSession } = await flameanvil.getSession(worldId, { auth })
	if (staleSession) {
		await flameanvil.cancelSession(worldId, staleSession.upload_id, { auth }).catch(() => {})
	}

	const { upload_id: uploadId } = await flameanvil.createSession(worldId, { auth })
	try {
		if (modpackFile) {
			const total = modpackFile.size + (serverPackFile?.size ?? 0)
			const track = <T>(handle: UploadHandle<T>, offset: number) =>
				handle.onProgress(({ loaded }) => {
					options.onUploadProgress?.({
						loaded: offset + loaded,
						total,
						progress: total > 0 ? (offset + loaded) / total : 0,
					})
				}).promise

			await track(flameanvil.uploadPlayerPack(worldId, uploadId, modpackFile, { auth }), 0)
			if (serverPackFile) {
				await track(
					flameanvil.uploadServerPack(worldId, uploadId, serverPackFile, { auth }),
					modpackFile.size,
				)
			}
		}

		await flameanvil.executeSession(
			worldId,
			uploadId,
			{
				link: modpackFile ? null : link,
				soft_override: options.softOverride,
				properties: ctx.buildProperties(),
			},
			{ auth },
		)
	} catch (error) {
		await flameanvil.cancelSession(worldId, uploadId, { auth }).catch(() => {})
		throw error
	}

	return { type: 'curse_forge_modpack', filename }
}
