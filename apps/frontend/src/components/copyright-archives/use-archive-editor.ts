import type { Labrinth } from '@modrinth/api-client'
import { injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

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

	const client = injectModrinthClient()
	const api = client.labrinth.copyright_archives_internal
	const queryClient = useQueryClient()
	const queryKey = ['copyright-archives', 'list'] as const
	const {
		data,
		isFetching: loading,
		error: queryError,
		refetch: load,
	} = useQuery({
		queryKey,
		enabled: import.meta.client,
		refetchOnWindowFocus: false,
		queryFn: async () => {
			const records: CopyrightProjectArchive[] = []
			const limit = 100
			for (let offset = 0; ; offset += limit) {
				const page = await api.list({ limit, offset })
				records.push(...page)
				if (page.length < limit) return records
			}
		},
	})
	const loadError = computed(() => queryError.value?.message ?? '')
	watch(
		queryError,
		(error) => {
			if (error)
				addNotification({
					title: 'Could not load copyright archives',
					text: error.message,
					type: 'error',
				})
		},
		{ immediate: true },
	)
	const { mutateAsync: persist, isPending: saving } = useMutation({
		mutationFn: async (operation: () => Promise<void>) => operation(),
		retry: false,
		onSuccess: () => {
			queryClient.setQueryData(queryKey, clone(saved.value.map(fromDraft)))
			void queryClient.invalidateQueries({ queryKey })
		},
	})
	const busy = computed(() => loading.value || saving.value || !!loadError.value)
	watch(
		data,
		(records) => {
			if (!records || hasChanges.value || saving.value) return
			saved.value = records.map(toDraft)
			reset()
		},
		{ immediate: true },
	)

	function accept(records: CopyrightProjectArchive[]) {
		for (const record of records) {
			const index = saved.value.findIndex((archive) => archive.id === record.id)
			if (index === -1) saved.value.push(toDraft(record))
			else saved.value[index] = toDraft(record)
		}
	}

	function reset() {
		if (saving.value) return
		archives.value = clone(saved.value)
		archiveJson.value = Object.fromEntries(
			archives.value.map((archive) => [archive.id, JSON.stringify(fromDraft(archive), null, '\t')]),
		)
		validationErrors.value = {}
	}

	async function save(): Promise<boolean> {
		if (busy.value || !hasChanges.value) return false
		validationErrors.value = {}
		const records: Labrinth.CopyrightArchives.Internal.Replace[] = []
		const originals = new Map(saved.value.map((archive) => [archive.id, fromDraft(archive)]))
		for (const archive of archives.value) {
			try {
				const value: unknown = JSON.parse(
					archiveJson.value[archive.id] ?? JSON.stringify(fromDraft(archive)),
				)
				if (JSON.stringify(value) === JSON.stringify(originals.get(archive.id))) continue
				const errors = validateArchive(value)
				if (!errors.length && (value as ArchiveInput).id !== archive.id)
					errors.push('The archive ID must remain unchanged.')
				if (errors.length) validationErrors.value[archive.id] = errors.join(' ')
				else {
					const record = fromDraft(
						toDraft(value as CopyrightProjectArchive),
					) as Labrinth.CopyrightArchives.Internal.Replace
					if (JSON.stringify(record) !== JSON.stringify(originals.get(archive.id)))
						records.push(record)
				}
			} catch {
				validationErrors.value[archive.id] = 'Enter valid JSON for a single archive object.'
			}
		}
		if (Object.keys(validationErrors.value).length) {
			addNotification({
				title: 'Could not save copyright archives',
				text: Object.entries(validationErrors.value)
					.map(([id, message]) => `Archive ${id}: ${message}`)
					.join('\n'),
				type: 'error',
			})
			return false
		}
		try {
			await persist(async () => {
				if (records.length) {
					await api.replace(records)
					accept(records)
				}
				const ids = new Set(archives.value.map((archive) => archive.id))
				const deleted = saved.value
					.filter((archive) => !ids.has(archive.id))
					.map((archive) => archive.id)
				if (deleted.length) {
					await api.delete(deleted)
					saved.value = saved.value.filter((archive) => ids.has(archive.id))
				}
			})
			reset()
			addNotification({ title: 'Copyright archives saved', type: 'success' })
			return true
		} catch (error) {
			addNotification({
				title: 'Could not save copyright archives',
				text: error instanceof Error ? error.message : String(error),
				type: 'error',
			})
			return false
		}
	}

	async function upsert(entries: ArchiveInput[]): Promise<boolean> {
		if (busy.value) return false
		const errors = entries.flatMap(validateArchive)
		if (errors.length) {
			addNotification({
				title: 'Invalid copyright archives',
				text: errors.join(' '),
				type: 'error',
			})
			return false
		}
		const ids = new Set<string>()
		for (const entry of entries) {
			if (
				entry.id &&
				(ids.has(entry.id) || !saved.value.some((archive) => archive.id === entry.id))
			) {
				addNotification({
					title: 'Invalid copyright archives',
					text: `Unknown or duplicate archive ID "${entry.id}".`,
					type: 'error',
				})
				return false
			}
			if (entry.id) ids.add(entry.id)
		}
		function apply(records: CopyrightProjectArchive[]) {
			accept(records)
			for (const record of records) {
				const archive = toDraft(record)
				const index = archives.value.findIndex((entry) => entry.id === record.id)
				if (index === -1) archives.value.push(archive)
				else archives.value[index] = archive
				archiveJson.value[record.id] = JSON.stringify(fromDraft(archive), null, '\t')
				Reflect.deleteProperty(validationErrors.value, record.id)
			}
		}
		const replacements = entries.filter(
			(entry): entry is Labrinth.CopyrightArchives.Internal.Replace => !!entry.id,
		)
		const additions = entries.filter((entry) => !entry.id)
		let replacementsSaved = false
		try {
			await persist(async () => {
				if (replacements.length) {
					await api.replace(replacements)
					replacementsSaved = true
					apply(replacements)
				}
				if (additions.length) {
					const newIds = await api.create(additions)
					apply(additions.map((entry, index) => ({ ...entry, id: newIds[index] })))
				}
			})
			addNotification({ title: 'Copyright archives saved', type: 'success' })
			return true
		} catch (error) {
			const outcome =
				replacements.length && !replacementsSaved
					? `No archives were saved. Failed to replace ${replacements.length} existing archive(s). ${additions.length} new archive(s) were not attempted.`
					: `${replacements.length} existing archive(s) replaced. Failed to create ${additions.length} new archive(s).`
			addNotification({
				title: replacementsSaved
					? 'Copyright archives partially saved'
					: 'Could not save copyright archives',
				text: `${outcome}\n${error instanceof Error ? error.message : String(error)}`,
				type: 'error',
			})
			return false
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

	return {
		archives,
		archiveJson,
		validationErrors,
		loading,
		saving,
		busy,
		loadError,
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
