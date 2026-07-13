import type { ContentItem } from '@modrinth/ui'
import { computed, type Ref, ref, watch } from 'vue'

export interface ModFolder {
	id: string
	name: string
	expanded: boolean
	modIds: string[]
	color?: string
}

export const FOLDER_COLORS = [
	'#ef4444',
	'#f97316',
	'#eab308',
	'#1bd96a',
	'#14b8a6',
	'#3b82f6',
	'#6366f1',
	'#a855f7',
	'#ec4899',
	'#64748b',
]

export function getModFolderStorageKey(instanceId: string) {
	return `instance-${instanceId}-mod-folders`
}

function loadFolders(instanceId: string): ModFolder[] {
	try {
		const raw = localStorage.getItem(getModFolderStorageKey(instanceId))
		return raw ? (JSON.parse(raw) as ModFolder[]) : []
	} catch {
		return []
	}
}

function saveFolders(instanceId: string, folders: ModFolder[]) {
	localStorage.setItem(getModFolderStorageKey(instanceId), JSON.stringify(folders))
}

function normalizeFileId(name: string): string {
	return name.replace(/\.disabled$/, '')
}

export function useModFolders(instanceId: Ref<string>) {
	const folders = ref<ModFolder[]>(loadFolders(instanceId.value))

	watch(instanceId, (newId, oldId) => {
		if (newId !== oldId) {
			folders.value = loadFolders(newId)
		}
	})

	watch(
		folders,
		(val) => {
			saveFolders(instanceId.value, val)
		},
		{ deep: true },
	)

	function getModId(item: ContentItem): string {
		const projectId = item.project?.id
		if (projectId) return projectId
		const fileId = normalizeFileId(item.file_name ?? '')
		if (fileId) return fileId
		return item.id
	}

	function isItemInAnyFolder(item: ContentItem): boolean {
		const id = getModId(item)
		return folders.value.some((folder) => folder.modIds.includes(id))
	}

	function getFolderForItem(item: ContentItem): ModFolder | undefined {
		const id = getModId(item)
		return folders.value.find((folder) => folder.modIds.includes(id))
	}

	function isFolderNameTaken(name: string, excludeFolderId?: string): boolean {
		return folders.value.some((f) => f.id !== excludeFolderId && f.name === name)
	}

	function createFolder(name: string, color?: string): ModFolder {
		const folder: ModFolder = {
			id: crypto.randomUUID(),
			name,
			expanded: true,
			modIds: [],
			...(color ? { color } : {}),
		}
		folders.value = [...folders.value, folder]
		return folder
	}

	function deleteFolder(folderId: string) {
		folders.value = folders.value.filter((f) => f.id !== folderId)
	}

	function renameFolder(folderId: string, newName: string) {
		folders.value = folders.value.map((f) => (f.id === folderId ? { ...f, name: newName } : f))
	}

	function setFolderColor(folderId: string, color?: string) {
		folders.value = folders.value.map((f) => (f.id === folderId ? { ...f, color } : f))
	}

	function toggleFolder(folderId: string) {
		folders.value = folders.value.map((f) =>
			f.id === folderId ? { ...f, expanded: !f.expanded } : f,
		)
	}

	function moveModToFolder(item: ContentItem, folderId: string) {
		const modId = getModId(item)
		folders.value = folders.value.map((f) => {
			const withoutMod = f.modIds.filter((id) => id !== modId)
			if (f.id === folderId) {
				return { ...f, modIds: [...withoutMod, modId] }
			}
			return { ...f, modIds: withoutMod }
		})
	}

	function moveModToRoot(item: ContentItem) {
		const modId = getModId(item)
		folders.value = folders.value.map((f) => ({
			...f,
			modIds: f.modIds.filter((id) => id !== modId),
		}))
	}

	function removeModFromFolders(item: ContentItem) {
		moveModToRoot(item)
	}

	const unassignedItems = (allItems: Ref<ContentItem[]>) =>
		computed(() => allItems.value.filter((item) => !isItemInAnyFolder(item)))

	return {
		folders,
		createFolder,
		deleteFolder,
		renameFolder,
		setFolderColor,
		toggleFolder,
		moveModToFolder,
		moveModToRoot,
		removeModFromFolders,
		isItemInAnyFolder,
		getFolderForItem,
		getModId,
		isFolderNameTaken,
		unassignedItems,
	}
}
