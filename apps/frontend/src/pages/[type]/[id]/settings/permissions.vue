<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { RightArrowIcon, SearchIcon, SortAscIcon, SortDescIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	Combobox,
	type ComboboxOption,
	commonMessages,
	defineMessages,
	EmptyState,
	ExternalProjectPermissionsCard,
	injectModrinthClient,
	injectProjectPageContext,
	IntlFormatted,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

const { formatMessage } = useVIntl()
const flags = useFeatureFlags()
const { projectV2: project } = injectProjectPageContext()
const { labrinth } = injectModrinthClient()

if (!flags.value.modpackPermissionsPage) {
	throw createError({
		fatal: true,
		statusCode: 404,
	})
}

type SortType = 'Oldest' | 'Newest'

const searchQuery = ref('')
const currentSortType = ref<SortType>('Oldest')

const {
	data: attributionData,
	error: attributionError,
	isPending: pending,
} = useQuery({
	queryKey: ['project-attribution', project.value.id],
	queryFn: () => labrinth.attribution_internal.listProjectAttribution(project.value.id),
})

const sortTypes: ComboboxOption<SortType>[] = [
	{ value: 'Oldest', label: 'Oldest' },
	{ value: 'Newest', label: 'Newest' },
]

function isAttributed(group: Labrinth.Attribution.Internal.AttributionGroup): boolean {
	return group.attribution !== null && group.attribution !== undefined
}

function isNoPermission(group: Labrinth.Attribution.Internal.AttributionGroup): boolean {
	const a = group.attribution
	if (!a || typeof a !== 'object') return false
	return (a as { type?: string }).type === 'no_permission'
}

const filteredGroups = computed(() => {
	const groups = attributionData.value ?? []
	const query = searchQuery.value.trim().toLowerCase()
	const filtered = query
		? groups.filter((group) => {
				if (group.flame_project_title?.toLowerCase().includes(query)) return true
				return (group.files ?? []).some((file) => file.name.toLowerCase().includes(query))
			})
		: [...groups]
	const direction = currentSortType.value === 'Newest' ? -1 : 1
	filtered.sort((a, b) => {
		const aTime = a.attributed_at ? Date.parse(a.attributed_at) : 0
		const bTime = b.attributed_at ? Date.parse(b.attributed_at) : 0
		if (aTime !== bTime) return (aTime - bTime) * direction
		return a.id.localeCompare(b.id) * direction
	})
	return filtered
})

const totalGroups = computed(() => attributionData.value?.length ?? 0)

const stats = computed(() => {
	const groups = attributionData.value ?? []
	let attributed = 0
	let pending = 0
	let noPermission = 0
	for (const group of groups) {
		if (isNoPermission(group)) {
			noPermission++
		} else if (isAttributed(group)) {
			attributed++
		} else {
			pending++
		}
	}
	return { total: groups.length, attributed, pending, noPermission }
})

const projectIsApproved = computed(() => project.value.status === 'approved')

const messages = defineMessages({
	searchPlaceholder: {
		id: 'project.settings.permissions.search-placeholder',
		defaultMessage:
			'Search {count} {count, plural, one {external project} other {external projects}}...',
	},
	infoBannerTitle: {
		id: 'project.settings.permissions.info-banner.title',
		defaultMessage: 'Learn how attributions work',
	},
	infoBannerDescription: {
		id: 'project.settings.permissions.info-banner.description',
		defaultMessage: `If you include content that isn’t hosted on Modrinth, you need to let us know where it’s from and verify that you have permission to distribute the files. Check out <link>our guide</link> to learn about how to do this properly!`,
	},
	learnMore: {
		id: 'project.settings.permissions.learn-more',
		defaultMessage: 'Learn more',
	},
	emptyStateHeading: {
		id: 'project.settings.permissions.empty-state.heading',
		defaultMessage: `You're all set!`,
	},
	emptyStateDescription: {
		id: 'project.settings.permissions.empty-state.description',
		defaultMessage: `None of your versions contain external content, so you don't need to worry about obtaining permissions.`,
	},
	completedTitle: {
		id: 'project.settings.permissions.completed.title',
		defaultMessage: `Attributions completed!`,
	},
	completedDescription: {
		id: 'project.settings.permissions.completed.description',
		defaultMessage: 'All external content has attributions provided.',
	},
	failTitle: {
		id: 'project.settings.permissions.fail.title',
		defaultMessage: `Some content can't be included`,
	},
	failDescription: {
		id: 'project.settings.permissions.fail.description',
		defaultMessage: `You don't have permission to redistribute some of the external content you've added. In order to publish on Modrinth, remove the infringing content.`,
	},
	attentionNeededTitle: {
		id: 'project.settings.permissions.attention-needed.title',
		defaultMessage: `Unknown embedded content`,
	},
	attentionNeededDescriptionApproved: {
		id: 'project.settings.permissions.attention-needed.description.proj-approved',
		defaultMessage: `Please provide proof that you have permission to redistribute all of the following files and any withheld versions will be automatically published.`,
	},
	attentionNeededDescriptionDraft: {
		id: 'project.settings.permissions.attention-needed.description.proj-draft',
		defaultMessage: `Please provide proof that you have permission to redistribute all of the following files before you can submit your project for review.`,
	},
	noResults: {
		id: 'project.settings.permissions.no-results',
		defaultMessage: 'No external projects match your search.',
	},
})

function dismissInfoBanner() {
	flags.value.dismissedExternalProjectsInfo = true
	saveFeatureFlags()
}
</script>
<template>
	<Admonition
		v-if="!flags.dismissedExternalProjectsInfo"
		type="info"
		class="mb-4"
		:header="formatMessage(messages.infoBannerTitle)"
		dismissible
		@dismiss="dismissInfoBanner"
	>
		<IntlFormatted :message-id="messages.infoBannerDescription">
			<template #link="{ children }">
				<a class="text-link" target="_blank"> <component :is="() => children" /> </a>
			</template>
		</IntlFormatted>
		<template #actions>
			<div class="flex">
				<ButtonStyled color="blue">
					<a> {{ formatMessage(messages.learnMore) }} <RightArrowIcon /> </a>
				</ButtonStyled>
			</div>
		</template>
	</Admonition>
	<template v-if="pending">
		<div class="flex flex-col gap-3">
			<div
				v-for="i in 3"
				:key="i"
				class="h-[56px] w-full animate-pulse rounded-2xl bg-surface-3"
			></div>
		</div>
	</template>
	<template v-else-if="totalGroups > 0">
		<Admonition
			v-if="stats.pending === 0 && stats.noPermission === 0"
			type="success"
			class="mb-4"
			:header="formatMessage(messages.completedTitle)"
			:body="formatMessage(messages.completedDescription)"
		/>
		<Admonition
			v-if="stats.pending > 0"
			type="warning"
			class="mb-4"
			:header="formatMessage(messages.attentionNeededTitle)"
			:body="
				formatMessage(
					projectIsApproved
						? messages.attentionNeededDescriptionApproved
						: messages.attentionNeededDescriptionDraft,
				)
			"
		/>
		<Admonition
			v-if="stats.noPermission > 0"
			type="critical"
			class="mb-4"
			:header="formatMessage(messages.failTitle)"
			:body="formatMessage(messages.failDescription)"
		/>
		<div class="grid grid-cols-[1fr_auto] gap-2">
			<StyledInput
				v-model="searchQuery"
				type="search"
				:placeholder="
					formatMessage(messages.searchPlaceholder, {
						count: totalGroups,
					})
				"
				:icon="SearchIcon"
				input-class="h-[40px]"
			/>
			<div>
				<Combobox
					v-model="currentSortType"
					class="!w-full flex-grow sm:!w-[150px] sm:flex-grow-0 lg:!w-[150px]"
					:options="sortTypes"
					:placeholder="formatMessage(commonMessages.sortByLabel)"
				>
					<template #selected>
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<SortAscIcon
								v-if="currentSortType === 'Oldest'"
								class="size-5 flex-shrink-0 text-secondary"
							/>
							<SortDescIcon v-else class="size-5 flex-shrink-0 text-secondary" />
							<span class="truncate text-contrast">{{ currentSortType }}</span>
						</span>
					</template>
				</Combobox>
			</div>
		</div>
		<div class="mt-4 flex flex-col gap-3">
			<TransitionGroup name="list">
				<ExternalProjectPermissionsCard
					v-for="group in filteredGroups"
					:key="group.id"
					:project-id="project.id"
					:group="group"
				/>
				<EmptyState
					v-if="filteredGroups.length === 0"
					:heading="formatMessage(messages.noResults)"
					type="no-search-result"
				/>
			</TransitionGroup>
		</div>

		<p v-if="attributionError" class="mt-4 text-sm text-red">
			{{ String(attributionError) }}
		</p>
	</template>
	<template v-else>
		<EmptyState
			:heading="formatMessage(messages.emptyStateHeading)"
			:description="formatMessage(messages.emptyStateDescription)"
			type="done"
		/>
	</template>
</template>
<style scoped>
.list-enter-from {
	opacity: 0;
	transform: translateY(-10px);
}

.list-leave-to {
	opacity: 0;
	transform: translateY(10px);
}

.list-move {
	transition: transform 200ms ease-in-out;
}
</style>
