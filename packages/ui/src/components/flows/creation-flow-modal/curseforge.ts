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
	notFound: {
		id: 'creation-flow.curseforge.not-found',
		defaultMessage: 'We couldn’t find this modpack on CurseForge.',
	},
	lookupFailed: {
		id: 'creation-flow.curseforge.lookup-failed',
		defaultMessage: 'We couldn’t reach CurseForge. Please try again.',
	},
	changeButton: {
		id: 'creation-flow.curseforge.change-button',
		defaultMessage: 'Change',
	},
	minecraftVersion: {
		id: 'creation-flow.curseforge.minecraft-version',
		defaultMessage: 'Minecraft {version}',
	},
	checkingServerPack: {
		id: 'creation-flow.curseforge.checking-server-pack',
		defaultMessage: 'Checking for a server pack...',
	},
	serverPackAvailable: {
		id: 'creation-flow.curseforge.server-pack-available',
		defaultMessage:
			'A server pack is available for this version. We’ll install {filename} on the server.',
	},
	serverPackClosest: {
		id: 'creation-flow.curseforge.server-pack-closest',
		defaultMessage:
			'A server pack is not available for this version. The closest version with a server pack is <link>{version}</link>.',
	},
	serverPackNone: {
		id: 'creation-flow.curseforge.server-pack-none',
		defaultMessage:
			'We could not find a server pack for any version of this project. We recommend choosing a different pack or uploading the server pack manually.',
	},
	ftbInstaller: {
		id: 'creation-flow.curseforge.ftb-installer',
		defaultMessage:
			'This version comes with FTB’s server installer, which we’ll use to set up the server.',
	},
})

/**
 * Normalizes any CurseForge modpack page link to its project or file link, dropping extra path
 * segments, queries, and fragments, such as those of the files list.
 *
 * @returns The canonical link, or `null` when the value isn't a CurseForge modpack link.
 */
export function normalizeCurseForgeModpackUrl(value: string): string | null {
	const trimmed = value.trim()
	let url: URL
	try {
		url = new URL(/^[a-z][a-z\d+.-]*:\/\//i.test(trimmed) ? trimmed : `https://${trimmed}`)
	} catch {
		return null
	}
	if (
		(url.protocol !== 'https:' && url.protocol !== 'http:') ||
		(url.hostname !== 'curseforge.com' && url.hostname !== 'www.curseforge.com') ||
		url.username ||
		url.password ||
		url.port
	) {
		return null
	}

	const segments = url.pathname.split('/').filter(Boolean)
	if (segments[0] === 'minecraft') segments.shift()
	const [section, slug, kind, fileId] = segments
	if (section !== 'modpacks' || !slug || !/^[\w-]+$/.test(slug)) return null

	const projectUrl = `https://www.curseforge.com/minecraft/modpacks/${slug}`
	return (kind === 'files' || kind === 'download') && fileId && /^\d+$/.test(fileId)
		? `${projectUrl}/files/${fileId}`
		: projectUrl
}

export function isCurseForgeModpackUrl(value: string): boolean {
	return normalizeCurseForgeModpackUrl(value) !== null
}

/**
 * Installs the CurseForge modpack selected in the creation flow through a FlameAnvil session,
 * either from the resolved link or from the uploaded client pack, with an optional uploaded
 * server pack.
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
	const modpack = modpackFile ? null : ctx.curseforgeModpack.value
	const link = modpack ? normalizeCurseForgeModpackUrl(ctx.curseforgeUrl.value) : null
	let filename: string
	if (modpackFile) {
		filename = modpackFile.name
	} else if (modpack && link) {
		filename = modpack.file.file_name
	} else {
		throw new Error('A CurseForge modpack link or file is required')
	}

	const { session: staleSession } = await flameanvil.getSession(worldId, { auth })
	if (staleSession) {
		await flameanvil.cancelSession(worldId, staleSession.upload_id, { auth }).catch(() => {})
	}

	const { upload_id: uploadId } = await flameanvil.createSession(worldId, { auth })
	try {
		const modpackSize = modpackFile?.size ?? 0
		const total = modpackSize + (serverPackFile?.size ?? 0)
		const track = <T>(handle: UploadHandle<T>, offset: number) =>
			handle.onProgress(({ loaded }) => {
				options.onUploadProgress?.({
					loaded: offset + loaded,
					total,
					progress: total > 0 ? (offset + loaded) / total : 0,
				})
			}).promise

		if (modpackFile) {
			await track(flameanvil.uploadPlayerPack(worldId, uploadId, modpackFile, { auth }), 0)
		}
		if (serverPackFile) {
			await track(
				flameanvil.uploadServerPack(worldId, uploadId, serverPackFile, { auth }),
				modpackSize,
			)
		}

		await flameanvil.executeSession(
			worldId,
			uploadId,
			{
				link,
				file_id: modpack?.file.id ?? null,
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
