import { computed, type Ref, ref, watch } from 'vue'

import { useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient } from '#ui/providers/api-client'

import { useArchiveDetailSettings } from '../../../settings'
import type { ArchiveDetailRow } from '../../../types'
import { detailsMessages as messages } from './messages'

/** A version on Modrinth holding a file with the same hash. */
interface ModrinthMatch {
	projectId: string
	projectName: string
	versionId: string
	versionNumber: string
}

/** Lookups by SHA-1, shared by every section, so a file is only looked up once per page. */
const lookups = new Map<string, Promise<ModrinthMatch | null>>()

/**
 * Whether a file (e.g. a nested jar) is already published on Modrinth, found by its SHA-1, as a
 * row linking to the version holding it. Only looked up while the row is shown, which the
 * viewer's settings decide.
 */
export function useModrinthLookupRow(sha1: () => (() => Promise<string>) | undefined) {
	const { formatMessage } = useVIntl()
	const client = injectModrinthClient()
	const settings = useArchiveDetailSettings()

	const visibility = computed(() => settings.visibility('modrinthLookup'))
	const match = ref<ModrinthMatch | null>(null) as Ref<ModrinthMatch | null>
	const loading = ref(false)

	function lookup(hash: string) {
		let found = lookups.get(hash)
		if (!found) {
			found = client.labrinth.versions_v2
				.getVersionFromFileHash(hash, 'sha1')
				.then(async (version) => {
					const project = await client.labrinth.projects_v3.get(version.project_id)
					return {
						projectId: project.slug ?? project.id,
						projectName: project.name,
						versionId: version.id,
						versionNumber: version.version_number,
					}
				})
				.catch(() => null)
			lookups.set(hash, found)
		}
		return found
	}

	watch(
		[sha1, () => visibility.value !== 'never'],
		async ([resolve, shown], _, onCleanup) => {
			let stale = false
			onCleanup(() => (stale = true))
			match.value = null
			if (!resolve || !shown) {
				loading.value = false
				return
			}
			loading.value = true
			const result = await resolve()
				.then(lookup)
				.catch(() => null)
			if (stale) return
			match.value = result
			loading.value = false
		},
		{ immediate: true },
	)

	return computed<ArchiveDetailRow | null>(() => {
		if (!sha1() || visibility.value === 'never') return null
		const found = match.value
		return {
			id: 'modrinth-lookup',
			label: formatMessage(messages.onModrinth),
			value: found
				? formatMessage(messages.onModrinthMatch, {
						project: found.projectName,
						version: found.versionNumber,
					})
				: loading.value
					? null
					: formatMessage(messages.onModrinthNone),
			loading: loading.value,
			href: found ? `/project/${found.projectId}/version/${found.versionId}` : undefined,
			visibility: visibility.value,
		}
	})
}
