<template>
	<div class="flex w-72 max-w-[80vw] flex-col gap-2 py-1">
		<Input
			v-if="projectIds.length > 6"
			v-model="search"
			:placeholder="formatMessage(commonMessages.searchPlaceholder)"
			:aria-label="formatMessage(commonMessages.searchLabel)"
			size="small"
		/>
		<div
			v-bind="containerProps"
			class="max-h-[60dvh] overscroll-contain"
			:style="{ height: `${Math.min(filteredIds.length, 8) * 32}px` }"
		>
			<div v-bind="wrapperProps">
				<div v-for="item in list" :key="item.data" class="flex h-8 items-center">
					<QueueProjectPreview :project-id="item.data" />
				</div>
			</div>
		</div>
		<span v-if="indexing" class="text-xs text-secondary">{{
			formatMessage(messages.searchingProjects)
		}}</span>
		<span v-else-if="indexError && search.trim()" class="text-xs text-secondary">{{
			formatMessage(messages.loadError)
		}}</span>
		<span v-if="!filteredIds.length && !indexing" class="text-secondary">{{
			formatMessage(search.trim() ? messages.noMatchingProjects : emptyMessage)
		}}</span>
	</div>
</template>

<script setup lang="ts">
import {
	commonMessages,
	injectModrinthClient,
	Input,
	type MessageDescriptor,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { refDebounced, useVirtualList } from '@vueuse/core'
import { computed, onScopeDispose, ref, shallowRef, watch } from 'vue'

import { projectReviewMessages as messages } from './messages'
import {
	type QueueProjectPreview as ProjectPreview,
	queueProjectPreviewKey,
	queueProjectPreviewOptions,
	seedQueueProjectPreviews,
} from './queue-project-cache'
import QueueProjectPreview from './queue-project-preview.vue'

const props = defineProps<{
	projectIds: string[]
	emptyMessage: MessageDescriptor
}>()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()
const search = ref('')
const debouncedSearch = refDebounced(search, 250)
const projects = shallowRef(new Map<string, ProjectPreview>())

function readCachedProjects() {
	projects.value = new Map(
		props.projectIds.flatMap((id) => {
			const project = queryClient.getQueryData<ProjectPreview>(queueProjectPreviewKey(id))
			return project ? [[id, project] as const] : []
		}),
	)
}

watch(
	() => props.projectIds,
	(ids) => {
		seedQueueProjectPreviews(queryClient, ids)
		readCachedProjects()
	},
	{ immediate: true },
)
const unsubscribe = queryClient.getQueryCache().subscribe((event) => {
	if (
		event.type === 'updated' &&
		event.action.type === 'success' &&
		event.query.queryKey[1] === 'queue-preview'
	) {
		readCachedProjects()
	}
})
onScopeDispose(unsubscribe)

const filteredIds = computed(() => {
	const term = search.value.trim().toLocaleLowerCase()
	if (!term) return props.projectIds
	return props.projectIds.filter(
		(id) =>
			id.toLocaleLowerCase().includes(term) ||
			projects.value.get(id)?.name.toLocaleLowerCase().includes(term),
	)
})
const { list, containerProps, wrapperProps, scrollTo } = useVirtualList(filteredIds, {
	itemHeight: 32,
	overscan: 2,
})
watch(search, () => scrollTo(0))

const { isFetching: indexing, isError: indexError } = useQuery(
	computed(() => ({
		queryKey: [
			'moderation',
			'queue-search-index',
			props.projectIds,
			!!debouncedSearch.value.trim(),
		] as const,
		enabled: !!debouncedSearch.value.trim(),
		staleTime: 0,
		retry: false,
		queryFn: async ({ signal }: { signal: AbortSignal }) => {
			const missing = props.projectIds.filter((id) => !projects.value.has(id))
			let failure: unknown
			for (let offset = 0; offset < missing.length; offset += 5) {
				signal.throwIfAborted()
				const results = await Promise.allSettled(
					missing
						.slice(offset, offset + 5)
						.map((id) =>
							queryClient.ensureQueryData(queueProjectPreviewOptions(id, client, queryClient)),
						),
				)
				for (const result of results) {
					if (result.status === 'rejected') failure = result.reason
				}
			}
			if (failure) throw failure
			return true
		},
	})),
)
</script>
