<script setup lang="ts">
import { DownloadIcon, HeartIcon, SearchIcon } from '@modrinth/assets'
import { renderString } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { useCompactNumber } from '#ui/composables/format-number.ts'

import { defineMessages, useVIntl } from '../../composables/i18n'
import { injectModrinthClient } from '../../providers/api-client'
import AutoLink from '../base/AutoLink.vue'
import Avatar from '../base/Avatar.vue'
import StyledInput from '../base/StyledInput.vue'

const { formatMessage } = useVIntl()
const { formatCompactNumber } = useCompactNumber()

const props = defineProps<{
	collectionId: string
}>()

const client = injectModrinthClient()

const { data: collection, isLoading: isLoadingCollection } = useQuery({
	queryKey: ['collection', () => props.collectionId],
	queryFn: () => client.labrinth.collections.get(props.collectionId),
	enabled: computed(() => !!props.collectionId),
})

const projectIds = computed(() => collection.value?.projects ?? [])

const { data: projects, isLoading: isLoadingProjects } = useQuery({
	queryKey: ['collection-projects', () => projectIds.value],
	queryFn: () => client.labrinth.projects_v3.getMultiple(projectIds.value),
	enabled: computed(() => projectIds.value.length > 0),
})

const query = ref<string>('')

const sortedProjects = computed(
	() =>
		projects.value
			?.slice()
			.sort((a, b) => b.followers - a.followers)
			.filter((project) => project.name.toLowerCase().includes(query.value.toLowerCase())) ?? [],
)

const supportsMarkdown = computed(() => collection.value?.user === '2REoufqX')

const messages = defineMessages({
	projectCount: {
		id: 'collection-widget.project-count',
		defaultMessage: '{count, plural, one {# project} other {# projects}}',
	},
	searchPlaceholder: {
		id: 'collection-widget.search-placeholder',
		defaultMessage: 'Search projects',
	},
	loadingProjects: {
		id: 'collection-widget.loading-projects',
		defaultMessage: 'Loading projects...',
	},
	emptyCollection: {
		id: 'collection-widget.empty-collection',
		defaultMessage: 'This collection is empty.',
	},
	noSearchResults: {
		id: 'collection-widget.no-search-results',
		defaultMessage: 'No projects match your search.',
	},
})
</script>

<template>
	<div
		class="rounded-2xl border border-solid border-surface-4 bg-surface-3 overflow-hidden grid grid-cols-[2fr_4fr]"
	>
		<div class="flex flex-col gap-2 p-4">
			<template v-if="isLoadingCollection">
				<div class="size-[96px] rounded-[16px] bg-surface-4 animate-pulse"></div>
				<div class="w-52 h-8 rounded-full bg-surface-4 animate-pulse"></div>
				<div class="w-28 h-5 rounded-full bg-surface-4 animate-pulse"></div>
				<div class="w-44 h-5 rounded-full bg-surface-4 animate-pulse"></div>
			</template>
			<template v-else>
				<AutoLink
					:to="`/collection/${collection?.id}`"
					class="flex flex-col gap-2 hover:underline"
					target="_blank"
				>
					<Avatar :src="collection?.icon_url" size="96px" />
					<span class="text-contrast font-semibold text-xl">{{ collection?.name }}</span>
				</AutoLink>
				<span>
					{{ formatMessage(messages.projectCount, { count: collection?.projects.length ?? 0 }) }}
				</span>
				<div
					v-if="supportsMarkdown"
					class="description-body"
					v-html="renderString(collection?.description ?? '')"
				/>
				<p v-else class="m-0 break-words">{{ collection?.description }}</p>
			</template>
		</div>
		<div
			class="flex flex-col bg-surface-2 border-0 border-l border-solid border-surface-4 overflow-hidden"
		>
			<StyledInput
				v-if="(projects?.length ?? 0) > 5"
				v-model="query"
				:placeholder="formatMessage(messages.searchPlaceholder)"
				input-class="bg-transparent !rounded-l-none !rounded-b-none m-1"
				:icon="SearchIcon"
				:disabled="isLoadingCollection || isLoadingProjects"
			/>
			<div
				class="flex flex-col h-[20rem] overflow-y-auto"
				:class="{ 'border-0 border-t border-solid border-surface-4': (projects?.length ?? 0) > 5 }"
			>
				<span v-if="isLoadingProjects" class="w-full py-12 text-center">
					{{ formatMessage(messages.loadingProjects) }}
				</span>
				<template v-else>
					<AutoLink
						v-for="project in sortedProjects"
						:key="project.id"
						:to="`/project/${project.id}`"
						class="flex items-center gap-2 px-3 py-2 even:bg-surface-2.5 hover:bg-surface-4 group"
						target="_blank"
					>
						<Avatar :src="project.icon_url" size="48px" />
						<div class="flex flex-col gap-1 truncate">
							<span class="flex items-center gap-2">
								<span class="text-contrast font-medium text-base group-hover:underline">
									{{ project.name }}
								</span>
								<span class="flex items-center gap-1 text-sm">
									<DownloadIcon class="size-4 text-secondary" />
									{{ formatCompactNumber(project.downloads) }}
								</span>
								<span class="flex items-center gap-1 text-sm">
									<HeartIcon class="size-4 text-secondary" />
									{{ formatCompactNumber(project.followers) }}
								</span>
							</span>
							<span class="text-primary text-sm truncate">{{ project.summary }}</span>
						</div>
					</AutoLink>
					<span v-if="projects?.length === 0" class="w-full py-12 text-center">
						{{ formatMessage(messages.emptyCollection) }}
					</span>
					<span v-else-if="sortedProjects.length === 0" class="w-full py-12 text-center">
						{{ formatMessage(messages.noSearchResults) }}
					</span>
				</template>
			</div>
		</div>
	</div>
</template>
<style scoped lang="scss">
:deep(.description-body) {
	p {
		margin: 0;
		line-height: 1.25;
	}

	a {
		color: var(--color-brand);
		font-weight: 600;
	}

	a:hover {
		text-decoration: underline;
	}
}
</style>
