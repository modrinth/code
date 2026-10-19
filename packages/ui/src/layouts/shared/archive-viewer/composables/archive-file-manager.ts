import { ClipboardCopyIcon, DownloadIcon } from '@modrinth/assets'
import { computed, markRaw, onScopeDispose, type Ref, ref, shallowReactive, shallowRef } from 'vue'

import { useFormatBytes } from '#ui/composables/format-bytes'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	AVAILABLE_DECOMPILER_TYPES,
	DEFAULT_DECOMPILE_OPTIONS,
	type DecompilerTypes,
} from '#ui/helpers/toolbox/decompile'
import { type FileTypeInfo, getFileTypeInfo, type HashType } from '#ui/helpers/toolbox/file'
import {
	type FileZipTreeNode,
	isZipArchive,
	type LazyDataAccess,
	type RootZipTreeNode,
	type ZipHandler,
	type ZipNode,
} from '#ui/helpers/toolbox/zip'
import {
	type DirectoryResult,
	type DirectoryTree,
	type FileInfo,
	type FileItemResultFrom,
	type FileManagerContext,
	type FileResult,
	type FileTypes,
	RAW_CONTENT_VARIANT,
} from '#ui/layouts/shared/files-tab/providers/file-manager'
import type { FileEntryAction, FileEntryDetail, FileItem } from '#ui/layouts/shared/files-tab/types'
import {
	infoFrom,
	isSameInfo,
	normalizeDirectoryPath,
	parentInfoFrom,
} from '#ui/layouts/shared/files-tab/utils'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import {
	getEditorLanguage,
	getFileExtension,
	isEditableFile,
	isImageFile,
} from '#ui/utils/file-extensions'

import { decompilerMessages } from '../messages'
import { archiveSettings, injectArchiveSettings } from '../settings'
import type { ArchiveSource } from '../types'
import { addToolboxLog } from './toolbox-logs'

const messages = defineMessages({
	readOnly: {
		id: 'archive-viewer.read-only',
		defaultMessage: 'Archives are read-only',
	},
	missingEntry: {
		id: 'archive-viewer.missing-entry',
		defaultMessage: '"{path}" does not exist within this archive',
	},
	disassembler: {
		id: 'archive-viewer.disassembler',
		defaultMessage: 'Disassembler',
	},
	decompileFailed: {
		id: 'archive-viewer.decompile-failed',
		defaultMessage: '{decompiler} could not decompile {path}',
	},
	detectedType: {
		id: 'archive-viewer.detail.detected-type',
		defaultMessage: 'Detected type',
	},
	compressedSize: {
		id: 'archive-viewer.detail.compressed-size',
		defaultMessage: 'Compressed size',
	},
	sha1: {
		id: 'archive-viewer.detail.sha1',
		defaultMessage: 'SHA-1',
	},
	downloadArchive: {
		id: 'archive-viewer.action.download-archive',
		defaultMessage: 'Download archive',
	},
	copySha512: {
		id: 'archive-viewer.action.copy-sha512',
		defaultMessage: 'Copy SHA-512',
	},
	copiedSha512: {
		id: 'archive-viewer.action.copied-sha512',
		defaultMessage: 'Copied SHA-512',
	},
})

export interface ArchiveFileManagerOptions {
	root: RootZipTreeNode
	handler: ZipHandler
	source: ArchiveSource
	/** Path within the archive to open first, e.g. `com/example/Mod.class`. */
	initialPath?: string | null
}

interface DirectoryState {
	result: DirectoryResult
	isLoading: Ref<boolean>
	loadError: Ref<Error | null>
}

function isClassFile(name: string) {
	return getFileExtension(name) === 'class'
}

/** Nested archives (e.g. jar-in-jar) are file nodes with children, which are browsed as folders. */
function isArchive(node: ZipNode) {
	return node.type === 'file' && isZipArchive(node)
}

function toSeconds(date: Date | undefined) {
	return date ? Math.floor(new Date(date).getTime() / 1000) : 0
}

function toError(error: unknown) {
	return error instanceof Error ? error : new Error(String(error))
}

/**
 * A read-only {@link FileManagerContext} over an archive read by the zip worker, for the files
 * layout to browse. The tree grows while nested archives are read in the background, class files
 * open decompiled, and file contents are only ever read off the main thread.
 */
export function useArchiveFileManager({
	root,
	handler,
	source,
	initialPath,
}: ArchiveFileManagerOptions) {
	const { formatMessage } = useVIntl()
	const formatBytes = useFormatBytes()
	const { addNotification } = injectNotificationManager()

	/** Bumped whenever nodes are added to the tree, which listings and search depend on. */
	const treeVersion = ref(0)
	const fullyLoaded = ref(false)
	/** The archive's own SHA-512, once it has been downloaded in full and hashed. */
	const sha512Hash = ref<string | null>(root.sha512Hash ?? null)

	/** The whole archive's hash of the given type, once all of it has been downloaded. */
	function archiveHash(type: HashType) {
		return handler.readArchiveHash(root.key, type)
	}

	let treeUpdated = Promise.withResolvers<void>()

	function nodeAt(path: string): ZipNode | undefined {
		const normalized = normalizeDirectoryPath(path)
		return normalized === '/' ? root : root.entries.get(normalized)
	}

	function fileNodeAt(path: string): FileZipTreeNode | undefined {
		return nodeAt(path)?.asFile()
	}

	const fileTypes = shallowReactive(new WeakMap<FileZipTreeNode, FileTypeInfo | null>())
	const fileTypesRequested = new WeakSet<FileZipTreeNode>()

	/**
	 * The file's type as the worker detects it from its contents, read the first time it's asked
	 * for. Until then it's `undefined`, and callers that track it are updated once it's read.
	 */
	function contentFileType(node: FileZipTreeNode): FileTypeInfo | null | undefined {
		if (!fileTypesRequested.has(node)) {
			fileTypesRequested.add(node)
			node.fileTypeInfo
				.get()
				.catch(() => null)
				.then((info) => fileTypes.set(node, info))
		}
		return fileTypes.get(node)
	}

	/**
	 * The file's type for the editor. Files with a known extension are judged by it; only those
	 * without one have their contents read to tell what they are.
	 */
	function editorFileType(file: FileInfo): FileTypeInfo | null {
		const ext = getFileExtension(file.name)
		const typeFromName = file.name.includes('.') ? getFileTypeInfo(file.name) : null
		if (typeFromName != null || isEditableFile(ext) || isImageFile(ext)) return typeFromName
		const node = fileNodeAt(file.path)
		return (node ? contentFileType(node) : null) ?? null
	}

	/** Every file opens, as the editor shows the ones it can't read as text in another way. */
	function canOpenInEditor() {
		return true
	}

	function editorLanguage(file: FileInfo) {
		return isClassFile(file.name) ? 'java' : getEditorLanguage(file.name, editorFileType(file))
	}

	/** Whether the entry is a nested archive whose own entries haven't been read yet. */
	function isPendingArchive(node: ZipNode) {
		return !fullyLoaded.value && isArchive(node) && node.children.size === 0
	}

	function toItem(node: ZipNode): FileItem {
		const modified = toSeconds(node.metadata.lastModDate)
		const browsable = node.type === 'directory' || isArchive(node)
		return {
			name: node.name,
			type: browsable ? 'directory' : 'file',
			path: normalizeDirectoryPath(node.path),
			modified,
			created: toSeconds(node.metadata.creationDate) || modified,
			size: node.type === 'directory' ? null : node.metadata.uncompressedSize,
			count: browsable ? node.children.size : null,
			readOnly: true,
		}
	}

	function missingEntryError(path: string) {
		return new Error(formatMessage(messages.missingEntry, { path }))
	}

	const directoryStates = new Map<string, DirectoryState>()
	const fileResults = new Map<string, FileResult>()

	/** Entries may only show up once the nested archive holding them is read. */
	function syncDirectoryState(path: string, state: DirectoryState) {
		const node = nodeAt(path)
		state.isLoading.value = node ? isPendingArchive(node) : !fullyLoaded.value
		state.loadError.value = node || !fullyLoaded.value ? null : missingEntryError(path)
	}

	function directoryResult(info: FileInfo<'directory'>): DirectoryResult {
		const path = normalizeDirectoryPath(info.path)
		let state = directoryStates.get(path)
		if (!state) {
			const isLoading = ref(false)
			const loadError = ref<Error | null>(null)
			const data = computed(() => {
				void treeVersion.value
				const node = nodeAt(path)
				return node && (node.type === 'directory' || isArchive(node))
					? [...node.children.values()].map(toItem)
					: []
			})
			state = { result: { ...info, data, isLoading, loadError }, isLoading, loadError }
			syncDirectoryState(path, state)
			directoryStates.set(path, state)
		}
		return state.result
	}

	/** Resolves the file at `path`, waiting for the nested archive holding it to be read. */
	async function waitForFileNode(path: string): Promise<FileZipTreeNode> {
		for (;;) {
			const node = fileNodeAt(path)
			if (node) return node
			if (fullyLoaded.value) throw missingEntryError(path)
			await treeUpdated.promise
		}
	}

	const settings = injectArchiveSettings(null)

	function isDecompiler(value: string | undefined): value is DecompilerTypes {
		return (AVAILABLE_DECOMPILER_TYPES as readonly string[]).includes(value ?? '')
	}

	/** Class files can be read by any of the decompilers, starting with the one the settings name. */
	function contentVariants(file: FileInfo<'file'>) {
		if (!isClassFile(file.name)) return null
		return {
			label: formatMessage(messages.disassembler),
			options: AVAILABLE_DECOMPILER_TYPES.map((type) => ({
				id: type,
				label: formatMessage(decompilerMessages[type]),
			})),
			default:
				settings?.get(archiveSettings.files.decompiler) ?? archiveSettings.files.decompiler.default,
		}
	}

	/**
	 * Class files are decompiled (by the given decompiler, else the default one), everything else
	 * (and any file asked for raw) is read as is. A class file a decompiler couldn't read is noted
	 * in the logs.
	 */
	async function readContent(path: string, variant?: string): Promise<ArrayBuffer> {
		const node = await waitForFileNode(path)
		if (variant === RAW_CONTENT_VARIANT) return node.getBuffer()
		const decompiler = isDecompiler(variant) ? variant : undefined
		const result = await node.asText(
			decompiler ? { ...DEFAULT_DECOMPILE_OPTIONS, decompiler } : undefined,
		)
		if (isClassFile(node.name) && (result.error || !result.wasDecompiled)) {
			addToolboxLog(
				'error',
				'Decompiler',
				formatMessage(messages.decompileFailed, {
					path: normalizeDirectoryPath(path),
					decompiler: formatMessage(
						decompilerMessages[decompiler ?? DEFAULT_DECOMPILE_OPTIONS.decompiler],
					),
				}),
				result.error,
			)
		}
		return result.wasDecompiled
			? new TextEncoder().encode(result.asString).buffer
			: await node.getBuffer()
	}

	function fileResultKey(path: string, variant?: string) {
		return variant ? `${path}\u0000${variant}` : path
	}

	function fileResult(info: FileInfo<'file'>, variant?: string): FileResult {
		const path = normalizeDirectoryPath(info.path)
		const key = fileResultKey(path, variant)
		let result = fileResults.get(key)
		if (!result) {
			const data = shallowRef<ArrayBuffer | null>(null)
			const isLoading = ref(true)
			const loadError = ref<Error | null>(null)
			readContent(path, variant)
				.then(
					(buffer) => (data.value = markRaw(buffer)),
					(error) => (loadError.value = toError(error)),
				)
				.finally(() => (isLoading.value = false))
			result = { ...info, data: computed(() => data.value), isLoading, loadError }
			fileResults.set(key, result)
		}
		return result
	}

	const directoryTree = {
		get: <T extends FileTypes>(info: FileInfo<T>, variant?: string): FileItemResultFrom<T> =>
			(info.type === 'file'
				? fileResult(info as FileInfo<'file'>, variant)
				: directoryResult(info as FileInfo<'directory'>)) as FileItemResultFrom<T>,
		/** Class files aren't prefetched, so hovering over a package doesn't queue up decompiles. */
		prefetch: <T extends FileTypes>(info: FileInfo<T>) => {
			if (info.type === 'file' && isClassFile(info.name)) return
			directoryTree.get(info)
		},
		expandedEntries: ref<string[]>([]),
		loadedEntries: computed(() => {
			void treeVersion.value
			return [...root.entries.values()].map(toItem)
		}),
	} satisfies DirectoryTree

	/** The entry at a path within the archive, or `null` when there is none (yet). */
	function locate(path: string): FileInfo | null {
		const node = nodeAt(path)
		if (!node) return null
		return node === root ? infoFrom('/') : infoFrom(toItem(node))
	}

	const currentLocation = ref<FileInfo>((initialPath ? locate(initialPath) : null) ?? infoFrom('/'))
	const currentDirectory = computed<FileInfo<'directory'>>(() =>
		currentLocation.value.type === 'file'
			? parentInfoFrom(currentLocation.value)
			: (currentLocation.value as FileInfo<'directory'>),
	)

	/** A requested path within a nested archive that hasn't been read yet, opened once it is. */
	let pendingPath = initialPath && !locate(initialPath) ? initialPath : null

	function navigateTo(file: FileInfo) {
		if (file.path !== '/') pendingPath = null
		currentLocation.value = file
	}

	/** Opens a path within the archive, or once it shows up if a nested archive is still being read. */
	function open(path: string) {
		const target = locate(path)
		if (target) {
			pendingPath = null
			if (!isSameInfo(target, currentLocation.value)) navigateTo(target)
		} else if (!fullyLoaded.value) {
			pendingPath = path
		}
	}

	function onTreeChanged() {
		treeVersion.value++
		for (const [path, state] of directoryStates) syncDirectoryState(path, state)

		if (pendingPath) {
			const target = locate(pendingPath)
			if (target) {
				pendingPath = null
				navigateTo(target)
			} else if (fullyLoaded.value) {
				pendingPath = null
			}
		}

		treeUpdated.resolve()
		treeUpdated = Promise.withResolvers<void>()
	}

	const updates = handler.onArchiveUpdate.subscribe((update) => {
		if (update.type === 'node_loaded' && update.root.root === root.key) {
			onTreeChanged()
		} else if (update.type === 'updated_node_data' && update.node.root === root.key) {
			const path = normalizeDirectoryPath(update.node.path)
			for (const key of [...fileResults.keys()]) {
				if (key === path || key.startsWith(`${path}\u0000`)) fileResults.delete(key)
			}
			const node = fileNodeAt(update.node.path)
			if (node) {
				fileTypesRequested.delete(node)
				fileTypes.delete(node)
			}
		} else if (update.type === 'root_hashed' && update.root.root === root.key) {
			sha512Hash.value = update.sha512Hash
		}
	})
	onScopeDispose(() => updates.cancel())

	root.onFinishedLoading.then(() => {
		fullyLoaded.value = true
		onTreeChanged()
	})

	function saveBuffer(buffer: ArrayBuffer, name: string) {
		const url = URL.createObjectURL(new Blob([buffer]))
		const anchor = document.createElement('a')
		anchor.href = url
		anchor.download = name
		anchor.click()
		setTimeout(() => URL.revokeObjectURL(url), 0)
	}

	async function downloadEntry(file: FileInfo) {
		const node = fileNodeAt(file.path)
		if (!node) return
		saveBuffer(await node.getBuffer(), node.name)
	}

	async function readLazy<K extends keyof LazyDataAccess>(
		entry: FileInfo,
		key: K,
	): Promise<ReturnType<LazyDataAccess[K]['get']> | null> {
		const node = fileNodeAt(entry.path)
		return node ? ((await node[key].get()) as ReturnType<LazyDataAccess[K]['get']>) : null
	}

	const isFileNode = (entry: FileInfo) => fileNodeAt(entry.path) != null

	const entryDetails: FileEntryDetail[] = [
		{
			id: 'archive-detected-type',
			label: formatMessage(messages.detectedType),
			shown: isFileNode,
			value: async (entry) => (await readLazy(entry, 'fileTypeInfo'))?.mime ?? null,
		},
		{
			id: 'archive-compressed-size',
			label: formatMessage(messages.compressedSize),
			shown: isFileNode,
			value: (entry) => {
				const node = fileNodeAt(entry.path)
				return node ? formatBytes(node.metadata.compressedSize) : null
			},
		},
		{
			id: 'archive-sha1',
			label: formatMessage(messages.sha1),
			shown: isFileNode,
			value: async (entry) => (await fileNodeAt(entry.path)?.hash('sha1')) ?? null,
		},
	]

	const entryActions: FileEntryAction[] = [
		{
			id: 'archive-download-nested',
			label: formatMessage(messages.downloadArchive),
			icon: DownloadIcon,
			shown: (entry) => {
				const node = nodeAt(entry.path)
				return node != null && isZipArchive(node)
			},
			action: downloadEntry,
		},
		{
			id: 'archive-copy-sha512',
			label: formatMessage(messages.copySha512),
			icon: ClipboardCopyIcon,
			shown: isFileNode,
			action: async (entry) => {
				const hash = await fileNodeAt(entry.path)?.hash('sha512')
				if (!hash) return
				await navigator.clipboard.writeText(hash)
				addNotification({ title: formatMessage(messages.copiedSha512), type: 'success' })
			},
		},
	]

	const context: FileManagerContext = {
		workspaceId: `archive:${source.sha512Hash ?? source.url.split('?')[0]}`,
		currentFile: computed(() => currentLocation.value),
		currentDirectory,
		directoryTree,
		loading: computed(() => directoryResult(currentDirectory.value).isLoading.value),
		error: computed(() => directoryResult(currentDirectory.value).loadError.value),
		navigateTo,

		createItem: async () => null,
		renameItem: async () => null,
		moveItem: async () => null,
		deleteItem: async () => false,
		writeFile: async () => {},
		uploadFiles: () => {},
		downloadFile: downloadEntry,

		isRefreshing: ref(false),
		refresh: async () => fileResults.clear(),

		readOnly: true,
		isReadOnly: () => true,
		readOnlyReason: computed(() => formatMessage(messages.readOnly)),
		basePath: computed(() => source.name),

		canOpenInEditor,
		editorLanguage,
		contentVariants,
		entryActions,
		entryDetails,
	}

	return {
		context,
		location: currentLocation,
		open,
		fullyLoaded,
		treeVersion,
		sha512Hash,
		archiveHash,
		nodeAt,
	}
}
