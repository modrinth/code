import { ModrinthApiError } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { type MaybeRefOrGetter, onScopeDispose, ref, toValue, watch } from 'vue'

const STALE_TIME = 1000 * 60 * 5
const CHECK_DEBOUNCE = 300
const PROJECT_SLUG_UNSAFE_CHARS = /[^a-zA-Z0-9._-]/g
const PROJECT_SLUG_REGEX = /^[a-zA-Z0-9._-]{3,64}$/

interface ProjectSlugSuggestionOptions {
	title: MaybeRefOrGetter<string>
	username?: MaybeRefOrGetter<string | null | undefined>
	currentProjectId?: MaybeRefOrGetter<string | null | undefined>
}

export function generateUrlSlug(value: string) {
	return value
		.trim()
		.toLowerCase()
		.replaceAll(' ', '-')
		.replaceAll(PROJECT_SLUG_UNSAFE_CHARS, '')
		.replaceAll(/--+/gm, '-')
}

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
}: ProjectSlugSuggestionOptions) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const suggestions = ref<string[]>([])
	const checking = ref(false)
	let debounceTimer: ReturnType<typeof setTimeout> | undefined
	let requestId = 0

	async function isAvailable(slug: string, projectId?: string | null) {
		try {
			const result = await queryClient.fetchQuery({
				queryKey: ['project', 'check', slug],
				queryFn: () => client.labrinth.projects_v2.check(slug),
				staleTime: STALE_TIME,
				retry: false,
			})
			return result.id === projectId
		} catch (error) {
			return error instanceof ModrinthApiError && error.statusCode === 404
		}
	}

	watch(
		() => [toValue(title), toValue(username), toValue(currentProjectId)] as const,
		([newTitle, newUsername, projectId]) => {
			if (import.meta.server) return

			clearTimeout(debounceTimer)
			const currentRequestId = ++requestId
			const candidates = generateProjectSlugSuggestions(newTitle, newUsername)
			suggestions.value = []

			if (candidates.length === 0) {
				checking.value = false
				return
			}

			checking.value = true
			debounceTimer = setTimeout(async () => {
				const availability = await Promise.all(
					candidates.map((candidate) => isAvailable(candidate, projectId)),
				)
				if (currentRequestId !== requestId) return

				suggestions.value = candidates.filter((_, index) => availability[index])
				checking.value = false
			}, CHECK_DEBOUNCE)
		},
		{ immediate: true },
	)

	onScopeDispose(() => clearTimeout(debounceTimer))

	return {
		checking,
		suggestions,
	}
}
