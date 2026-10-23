import { computed, type Ref, ref } from 'vue'

import { infoFrom } from '../utils'
import type {
	DirectoryTree,
	FileInfo,
	FileItemResultFrom,
	FileManagerContext,
	FileTypes,
} from './file-manager'

/**
 * A read-only file manager with nothing in it, for while there's nothing to browse yet. It never
 * reports loading, so the files layout shows its workspace right away; hosts show their own
 * progress instead.
 */
export function emptyFileManager(options: {
	readOnlyReason: Readonly<Ref<string>>
}): FileManagerContext {
	const root = infoFrom('/')
	const expandedEntries = ref<string[]>([])
	const directoryTree: DirectoryTree = {
		get: <T extends FileTypes>(file: FileInfo<T>) =>
			({
				...file,
				data: computed(() => (file.type === 'directory' ? [] : null)),
				isLoading: ref(false),
				loadError: ref(null),
			}) as unknown as FileItemResultFrom<T>,
		prefetch: () => {},
		expandedEntries,
		loadedEntries: computed(() => []),
	}

	return {
		workspaceId: null,
		currentFile: computed(() => root),
		currentDirectory: computed(() => root),
		directoryTree,
		loading: computed(() => false),
		error: computed(() => null),
		navigateTo: () => {},
		createItem: async () => null,
		renameItem: async () => null,
		moveItem: async () => null,
		deleteItem: async () => false,
		writeFile: async () => {},
		uploadFiles: () => {},
		downloadFile: async () => {},
		isRefreshing: ref(false),
		refresh: async () => {},
		readOnly: true,
		isReadOnly: () => true,
		readOnlyReason: options.readOnlyReason,
	}
}

/**
 * A file manager standing in for whichever one is current, so the files layout and the workbench
 * around it stay mounted while what they browse changes (e.g. another archive is opened). Falls
 * back to `idle` while there's no current one.
 *
 * Every property is looked up on the current manager when it's read, and as that reads `current`,
 * templates and computeds that read the context update once it changes.
 */
export function switchableFileManager(
	current: Readonly<Ref<FileManagerContext | null>>,
	idle: FileManagerContext,
): FileManagerContext {
	const manager = () => current.value ?? idle
	/** The directory tree is handed on once (e.g. to directory views), so it forwards too. */
	const directoryTree = new Proxy({} as DirectoryTree, {
		get: (_, key) => Reflect.get(manager().directoryTree, key),
		has: (_, key) => key in manager().directoryTree,
	})

	return new Proxy({} as FileManagerContext, {
		get: (_, key) => (key === 'directoryTree' ? directoryTree : Reflect.get(manager(), key)),
		has: (_, key) => key in manager(),
	})
}
