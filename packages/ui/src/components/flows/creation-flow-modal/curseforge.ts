import type {
	AbstractModrinthClient,
	Archon,
	Kyros,
	UploadHandle,
	UploadProgress,
} from '@modrinth/api-client'
import { ref } from 'vue'

import { defineMessages } from '#ui/composables/i18n'
import { createContext } from '#ui/providers/create-context'

import type {
	CreationFlowContextValue,
	CurseForgePackFile,
	CurseForgeTarget,
} from './creation-flow-context'

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
	processingFile: {
		id: 'creation-flow.curseforge.processing-file',
		defaultMessage: 'Processing...',
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

/** The CurseForge modpack picked from a link or uploaded packs, shared by the flows that pick one. */
export type CurseForgeSelection = Pick<
	CreationFlowContextValue,
	| 'curseforgeUrl'
	| 'curseforgeModpackFile'
	| 'curseforgeServerPackFile'
	| 'curseforgeModpack'
	| 'curseforgeFileId'
	| 'curseforgeFiles'
	| 'curseforgeProcessing'
	| 'queryCurseForgeLink'
	| 'listCurseForgeFiles'
>

export const [injectCurseForgeSelection, provideCurseForgeSelection] =
	createContext<CurseForgeSelection>('CurseForgeStage', 'curseforgeSelection')

/** Creates an empty CurseForge selection that resolves links against the given world. */
export function createCurseForgeSelection(
	client: AbstractModrinthClient,
	target: () => CurseForgeTarget,
): CurseForgeSelection {
	return {
		curseforgeUrl: ref(''),
		curseforgeModpackFile: ref<CurseForgePackFile | null>(null),
		curseforgeServerPackFile: ref<CurseForgePackFile | null>(null),
		curseforgeModpack: ref<Kyros.FlameAnvil.v1.LinkQueryResponse | null>(null),
		curseforgeFileId: ref<number | null>(null),
		curseforgeFiles: ref<Kyros.FlameAnvil.v1.LinkQueryFile[]>([]),
		curseforgeProcessing: ref(false),
		queryCurseForgeLink: (link, fileId) => {
			const { worldId, auth } = target()
			return client.kyros.flameanvil_v1.queryLink(worldId, { link, file_id: fileId }, { auth })
		},
		listCurseForgeFiles: (projectId) => {
			const { worldId, auth } = target()
			return client.kyros.flameanvil_v1.listProjectFiles(worldId, projectId, undefined, { auth })
		},
	}
}

/** Whether the selection is ready to install: an uploaded pack, or a link resolved for the selected file. */
export function isCurseForgeSelectionReady(selection: CurseForgeSelection): boolean {
	if (selection.curseforgeProcessing.value) return false
	if (selection.curseforgeModpackFile.value) return true
	const modpack = selection.curseforgeModpack.value
	return !!modpack && modpack.file.id === selection.curseforgeFileId.value
}

export function hasCurseForgeUploads(selection: CurseForgeSelection): boolean {
	return !!selection.curseforgeModpackFile.value || !!selection.curseforgeServerPackFile.value
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
	const { filename } = await executeCurseForgeSelection(client, worldId, ctx, {
		softOverride: options.softOverride,
		properties: ctx.buildProperties(),
		auth: options.auth,
		onUploadProgress: options.onUploadProgress,
	})

	return { type: 'curse_forge_modpack', filename }
}

/**
 * Restores the client files of a world installed from a CurseForge modpack, by updating it to the
 * selected modpack once a backup of the world succeeds.
 *
 * @returns The backup made before restoring, which reports the restoration's progress.
 */
export async function restoreCurseForgeClientFiles(
	client: AbstractModrinthClient,
	worldId: string,
	selection: CurseForgeSelection,
	options?: {
		onUploadProgress?: (progress: UploadProgress) => void
	},
): Promise<string | null> {
	const { response } = await executeCurseForgeSelection(client, worldId, selection, {
		softOverride: true,
		restoreClientFiles: true,
		properties: { known: {} },
		onUploadProgress: options?.onUploadProgress,
	})

	return response.backup_id
}

async function executeCurseForgeSelection(
	client: AbstractModrinthClient,
	worldId: string,
	selection: CurseForgeSelection,
	options: {
		softOverride: boolean
		restoreClientFiles?: boolean
		properties: Archon.Content.v1.PropertiesFields
		auth?: Archon.Servers.v0.JWTAuth
		onUploadProgress?: (progress: UploadProgress) => void
	},
) {
	const flameanvil = client.kyros.flameanvil_v1
	const { auth } = options
	const modpackFile = selection.curseforgeModpackFile.value
	const serverPackFile = selection.curseforgeServerPackFile.value
	const modpack = modpackFile ? null : selection.curseforgeModpack.value
	const link = modpack ? normalizeCurseForgeModpackUrl(selection.curseforgeUrl.value) : null
	let filename: string
	if (modpackFile) {
		filename = modpackFile.name
	} else if (modpack && link) {
		filename = modpack.file.file_name
	} else {
		throw new Error('A CurseForge modpack link or file is required')
	}

	const response = await withFlameAnvilSession(client, worldId, auth, async (uploadId) => {
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

		return flameanvil.executeSession(
			worldId,
			uploadId,
			{
				link,
				file_id: modpack?.file.id ?? null,
				soft_override: options.softOverride,
				restore_client_files: options.restoreClientFiles,
				properties: options.properties,
			},
			{ auth },
		)
	})

	return { filename, response }
}

/**
 * Switches a modpack installed from a CurseForge link to another file of the same project.
 *
 * @returns The installation progress key Kyros publishes for the queued installation.
 */
export async function switchCurseForgeModpackFile(
	client: AbstractModrinthClient,
	worldId: string,
	link: Archon.Content.v1.CurseForgeModpackLink,
	file: Kyros.FlameAnvil.v1.ProjectFile,
): Promise<Archon.Websocket.v0.InstallProgressCurseForgeModpackKey> {
	await withFlameAnvilSession(client, worldId, undefined, (uploadId) =>
		client.kyros.flameanvil_v1.executeSession(worldId, uploadId, {
			link: `https://www.curseforge.com/minecraft/modpacks/${link.slug}`,
			file_id: file.id,
			soft_override: true,
			properties: { known: {} },
		}),
	)

	return { type: 'curse_forge_modpack', filename: file.file_name }
}

/**
 * Runs `run` within a new FlameAnvil session, cancelling any session left over from an abandoned
 * attempt first, and the new session if `run` fails before the installation is queued.
 */
async function withFlameAnvilSession<T>(
	client: AbstractModrinthClient,
	worldId: string,
	auth: Archon.Servers.v0.JWTAuth | undefined,
	run: (uploadId: string) => Promise<T>,
): Promise<T> {
	const flameanvil = client.kyros.flameanvil_v1
	const { session: staleSession } = await flameanvil.getSession(worldId, { auth })
	if (staleSession) {
		await flameanvil.cancelSession(worldId, staleSession.upload_id, { auth }).catch(() => {})
	}

	const { upload_id: uploadId } = await flameanvil.createSession(worldId, { auth })
	try {
		return await run(uploadId)
	} catch (error) {
		await flameanvil.cancelSession(worldId, uploadId, { auth }).catch(() => {})
		throw error
	}
}
