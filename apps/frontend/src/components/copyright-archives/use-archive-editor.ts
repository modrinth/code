import { injectNotificationManager } from '@modrinth/ui'
import { computed, onMounted, ref } from 'vue'

import { loadArchives, saveArchives } from './storage'
import {
	type ArchiveDraft,
	type ArchiveInput,
	type CopyrightProjectArchive,
	fromDraft,
	toDraft,
} from './types'
import { validateArchive } from './validation'

function clone<T>(value: T): T {
	return JSON.parse(JSON.stringify(value))
}

export function useArchiveEditor() {
	const { addNotification } = injectNotificationManager()
	const saved = ref<ArchiveDraft[]>([])
	const archives = ref<ArchiveDraft[]>([])
	const archiveJson = ref<Record<string, string>>({})
	const validationErrors = ref<Record<string, string>>({})
	const loading = ref(true)
	const saving = ref(false)
	const loadError = ref('')
	const saveError = ref('')
	const busy = computed(() => loading.value || saving.value || !!loadError.value)
	const originalSnapshot = computed(() =>
		saved.value.map((archive) => JSON.stringify(fromDraft(archive), null, '\t')),
	)
	const currentSnapshot = computed(() =>
		archives.value.map(
			(archive) => archiveJson.value[archive.id] ?? JSON.stringify(fromDraft(archive), null, '\t'),
		),
	)
	const hasChanges = computed(
		() => JSON.stringify(originalSnapshot.value) !== JSON.stringify(currentSnapshot.value),
	)

	async function load() {
		loading.value = true
		loadError.value = ''
		try {
			saved.value = (await loadArchives()).map(toDraft)
			reset()
		} catch (error) {
			loadError.value = error instanceof Error ? error.message : String(error)
		} finally {
			loading.value = false
		}
	}

	function reset() {
		if (saving.value) return
		archives.value = clone(saved.value)
		archiveJson.value = Object.fromEntries(
			archives.value.map((archive) => [archive.id, JSON.stringify(fromDraft(archive), null, '\t')]),
		)
		validationErrors.value = {}
		saveError.value = ''
	}

	async function save(): Promise<boolean> {
		if (busy.value || !hasChanges.value) return false
		validationErrors.value = {}
		saveError.value = ''
		const records: CopyrightProjectArchive[] = []
		for (const archive of archives.value) {
			try {
				const value: unknown = JSON.parse(
					archiveJson.value[archive.id] ?? JSON.stringify(fromDraft(archive)),
				)
				const errors = validateArchive(value)
				if (!errors.length && (value as ArchiveInput).id !== archive.id)
					errors.push('The archive ID must remain unchanged.')
				if (errors.length) validationErrors.value[archive.id] = errors.join(' ')
				else records.push(value as CopyrightProjectArchive)
			} catch {
				validationErrors.value[archive.id] = 'Enter valid JSON for a single archive object.'
			}
		}
		if (Object.keys(validationErrors.value).length) return false
		saving.value = true
		try {
			const persisted = await saveArchives(clone(records), clone(saved.value.map(fromDraft)))
			saved.value = persisted.map(toDraft)
			archives.value = clone(saved.value)
			archiveJson.value = Object.fromEntries(
				archives.value.map((archive) => [
					archive.id,
					JSON.stringify(fromDraft(archive), null, '\t'),
				]),
			)
			addNotification({ title: 'Copyright archives saved', type: 'success' })
			return true
		} catch (error) {
			saveError.value = error instanceof Error ? error.message : String(error)
			return false
		} finally {
			saving.value = false
		}
	}

	async function upsert(entries: ArchiveInput[]): Promise<boolean> {
		if (busy.value) return false
		const errors = entries.flatMap(validateArchive)
		if (errors.length) {
			saveError.value = errors.join(' ')
			return false
		}
		const replacements = entries.map((entry) =>
			toDraft({ ...entry, id: entry.id ?? crypto.randomUUID() }),
		)
		const nextSaved = clone(saved.value)
		const nextArchives = clone(archives.value)
		for (const archive of replacements) {
			for (const collection of [nextSaved, nextArchives]) {
				const index = collection.findIndex((record) => record.id === archive.id)
				if (index === -1) collection.push(archive)
				else collection[index] = archive
			}
		}
		saving.value = true
		saveError.value = ''
		try {
			await saveArchives(clone(nextSaved.map(fromDraft)), clone(saved.value.map(fromDraft)))
			saved.value = nextSaved
			archives.value = nextArchives
			for (const archive of replacements) {
				archiveJson.value[archive.id] = JSON.stringify(fromDraft(archive), null, '\t')
				validationErrors.value = Object.fromEntries(
					Object.entries(validationErrors.value).filter(([id]) => id !== archive.id),
				)
			}
			addNotification({ title: 'Copyright archives saved', type: 'success' })
			return true
		} catch (error) {
			saveError.value = error instanceof Error ? error.message : String(error)
			return false
		} finally {
			saving.value = false
		}
	}

	function remove(id: string) {
		if (busy.value) return
		archives.value = archives.value.filter((archive) => archive.id !== id)
		archiveJson.value = Object.fromEntries(
			Object.entries(archiveJson.value).filter(([archiveId]) => archiveId !== id),
		)
		validationErrors.value = Object.fromEntries(
			Object.entries(validationErrors.value).filter(([archiveId]) => archiveId !== id),
		)
	}

	onMounted(load)
	return {
		archives,
		archiveJson,
		validationErrors,
		loading,
		saving,
		busy,
		loadError,
		saveError,
		originalSnapshot,
		currentSnapshot,
		hasChanges,
		load,
		upsert,
		remove,
		save,
		reset,
	}
}
