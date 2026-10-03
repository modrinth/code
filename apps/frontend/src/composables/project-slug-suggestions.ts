import { generateUrlSlug } from '@modrinth/moderation/src/utils'
import { injectModrinthClient } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { refDebounced } from '@vueuse/core'
import { computed, type MaybeRefOrGetter, ref, toValue } from 'vue'

import { projectQueryOptions, STALE_TIME } from '~/composables/queries/project'

const CHECK_DEBOUNCE = 300
const PROJECT_SLUG_REGEX = /^[a-zA-Z0-9._-]{3,64}$/

interface ProjectSlugSuggestionOptions {
	title: MaybeRefOrGetter<string>
	username?: MaybeRefOrGetter<string | null | undefined>
	currentProjectId?: MaybeRefOrGetter<string | null | undefined>
	enabled?: MaybeRefOrGetter<boolean>
}

export { generateUrlSlug }

function isValidProjectSlug(value: string) {
	return PROJECT_SLUG_REGEX.test(value)
}

function generateProjectSlugSuggestions(title: string, username?: string | null) {
	const titleSlug = generateUrlSlug(title)
	const titleWords = title
		.trim()
		.split(/\s+/)
		.map((word) => generateUrlSlug(word))
		.filter(Boolean)
	const acronym = titleWords.length > 1 ? titleWords.map((word) => word[0]).join('') : ''
	const withoutDashes = titleSlug.replaceAll('-', '')
	const usernameSlug = username ? generateUrlSlug(username) : ''
	let withUsername = ''

	if (titleSlug && usernameSlug) {
		const availableTitleLength = 64 - usernameSlug.length - 1
		const truncatedTitle = titleSlug.slice(0, availableTitleLength).replace(/-+$/, '')
		if (truncatedTitle) withUsername = `${truncatedTitle}-${usernameSlug}`
	}

	return [...new Set([titleSlug, acronym, withoutDashes, withUsername])].filter(isValidProjectSlug)
}

export function useSlugSuggestionVisibility() {
	const visible = ref(false)

	function onFocusIn() {
		visible.value = true
	}

	function onFocusOut(event: FocusEvent) {
		const container = event.currentTarget as HTMLElement
		if (!container.contains(event.relatedTarget as Node | null)) visible.value = false
	}

	return {
		onFocusIn,
		onFocusOut,
		visible,
	}
}

export function useProjectSlugSuggestions({
	title,
	username,
	currentProjectId,
	enabled = true,
}: ProjectSlugSuggestionOptions) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const candidates = computed(() =>
		generateProjectSlugSuggestions(toValue(title), toValue(username)),
	)
	const debouncedCandidates = refDebounced(candidates, CHECK_DEBOUNCE)
	const query = useQuery(
		computed(() => {
			const slugs = candidates.value
			const projectId = toValue(currentProjectId) ?? ''
			return {
				queryKey: ['project', 'slug-suggestions', slugs, projectId] as const,
				queryFn: async () => {
					const availability = await Promise.all(
						slugs.map((slug) =>
							queryClient.fetchQuery({
								...projectQueryOptions.slugAvailability(slug, projectId, client),
								staleTime: STALE_TIME,
								gcTime: STALE_TIME,
							}),
						),
					)
					return slugs.filter((_, index) => availability[index])
				},
				enabled:
					!import.meta.server &&
					toValue(enabled) &&
					slugs.length > 0 &&
					slugs.join() === debouncedCandidates.value.join(),
				staleTime: STALE_TIME,
				gcTime: STALE_TIME,
				retry: false,
			}
		}),
	)

	return {
		checking: query.isFetching,
		suggestions: computed(() => (toValue(enabled) ? (query.data.value ?? []) : [])),
	}
}
