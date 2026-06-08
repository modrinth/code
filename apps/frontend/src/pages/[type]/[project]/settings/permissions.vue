<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ArrowDown10Icon,
	ArrowDownWideNarrowIcon,
	ClockArrowDownIcon,
	FoldVerticalIcon,
	RightArrowIcon,
	SearchIcon,
	UnfoldVerticalIcon,
} from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	Combobox,
	type ComboboxOption,
	commonMessages,
	createAttributionGroupTitle,
	defineMessage,
	defineMessages,
	EmptyState,
	ExternalProjectPermissionsCard,
	injectModrinthClient,
	injectProjectPageContext,
	IntlFormatted,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { setupAttributionModerationProvider } from '~/providers/setup/attribution-moderation'

setupAttributionModerationProvider()

const auth = await useAuth()
const flags = useFeatureFlags()

const isModerator = computed(() => {
	return isStaff(auth.value?.user) && !flags.value.showModeratorProjectMemberUi
})

const { formatMessage } = useVIntl()
const { projectV2: project } = injectProjectPageContext()
const { labrinth } = injectModrinthClient()

type SortType = 'status' | 'most_files' | 'recently_edited' | 'rejected'

const searchQuery = ref('')
const currentSortType = ref<SortType>('status')
const cardCollapsedById = ref<Record<string, boolean>>({})

const {
	data: attributionData,
	error: attributionError,
	isPending: pending,
} = useQuery<Labrinth.Attribution.Internal.AttributionGroup[]>({
	queryKey: ['project-attribution', project.value.id],
	queryFn: () => labrinth.attribution_internal.listProjectAttribution(project.value.id),
})

const sortRejectedLabel = defineMessage({
	id: 'project.settings.permissions.sort.rejected',
	defaultMessage: 'Rejected',
})

const sortTypes = computed<ComboboxOption<SortType>[]>(() => {
	const options: ComboboxOption<SortType>[] = [
		{
			value: 'status',
			label: formatMessage(
				defineMessage({
					id: 'project.settings.permissions.sort.status',
					defaultMessage: 'Status',
				}),
			),
		},
		{
			value: 'most_files',
			label: formatMessage(
				defineMessage({
					id: 'project.settings.permissions.sort.most-files',
					defaultMessage: 'Most files',
				}),
			),
		},
		{
			value: 'recently_edited',
			label: formatMessage(
				defineMessage({
					id: 'project.settings.permissions.sort.recently-edited',
					defaultMessage: 'Recently edited',
				}),
			),
		},
	]
	if (hasMixedModerationStatus.value) {
		options.push({
			value: 'rejected',
			label: formatMessage(sortRejectedLabel),
		})
	}
	return options
})

const currentSortLabel = computed(() => {
	return sortTypes.value.find((option) => option.value === currentSortType.value)?.label ?? ''
})

function isAttributed(group: Labrinth.Attribution.Internal.AttributionGroup): boolean {
	return !!group.attribution && !isNoPermission(group)
}

function isNoPermission(group: Labrinth.Attribution.Internal.AttributionGroup): boolean {
	return group.attribution?.kind === 'no_permission'
}

function isModerationRejected(group: Labrinth.Attribution.Internal.AttributionGroup): boolean {
	const kind = group.attribution?.moderation_status?.kind
	return kind === 'bad_proof' || kind === 'not_allowed'
}

function rejectedSortRank(group: Labrinth.Attribution.Internal.AttributionGroup): number {
	if (isModerationRejected(group)) {
		return 0
	}
	if (group.attribution?.moderation_status?.kind === 'approved') {
		return 1
	}
	return 2
}

const hasMixedModerationStatus = computed(() => {
	const groups = attributionData.value ?? []
	let hasRejected = false
	let hasNonRejected = false
	for (const group of groups) {
		if (isModerationRejected(group)) {
			hasRejected = true
		} else {
			hasNonRejected = true
		}
		if (hasRejected && hasNonRejected) {
			return true
		}
	}
	return false
})

watch(hasMixedModerationStatus, (mixed) => {
	if (!mixed && currentSortType.value === 'rejected') {
		currentSortType.value = 'status'
	}
})

function statusSortRank(group: Labrinth.Attribution.Internal.AttributionGroup): number {
	if (isNoPermission(group)) {
		return 0
	} else if (!group.attribution) {
		return 1
	} else if (group.attribution?.kind === 'globally_allowed') {
		return 3
	} else {
		return 2
	}
}

function alphabetSortKey(group: Labrinth.Attribution.Internal.AttributionGroup): string {
	return createAttributionGroupTitle(group, formatMessage)
}

function compareAlphabetical(
	a: Labrinth.Attribution.Internal.AttributionGroup,
	b: Labrinth.Attribution.Internal.AttributionGroup,
): number {
	return alphabetSortKey(a).localeCompare(alphabetSortKey(b), undefined, { sensitivity: 'base' })
}

function attributedTimestamp(group: Labrinth.Attribution.Internal.AttributionGroup): number {
	if (!group.attributed_at) return Number.NEGATIVE_INFINITY
	const t = Date.parse(group.attributed_at)
	return Number.isNaN(t) ? Number.NEGATIVE_INFINITY : t
}

function groupMatchesPermissionsSearch(
	group: Labrinth.Attribution.Internal.AttributionGroup,
	queryTrimmed: string,
): boolean {
	const query = queryTrimmed.toLowerCase()
	if (group.flame_project?.title?.toLowerCase().includes(query)) {
		return true
	}
	return (group.files ?? []).some((file) => file.name.toLowerCase().includes(query))
}

const filteredGroups = computed(() => {
	const groups = attributionData.value ?? []
	const queryTrimmed = searchQuery.value.trim()
	const filtered = queryTrimmed
		? groups.filter((group) => groupMatchesPermissionsSearch(group, queryTrimmed))
		: [...groups]
	const sortMode = currentSortType.value
	filtered.sort(compareAlphabetical)
	filtered.sort((a, b) => {
		if (sortMode === 'status') {
			return statusSortRank(a) - statusSortRank(b)
		}
		if (sortMode === 'most_files') {
			const ac = a.files?.length ?? 0
			const bc = b.files?.length ?? 0
			return bc - ac
		}
		if (sortMode === 'rejected') {
			return rejectedSortRank(a) - rejectedSortRank(b)
		}
		const at = attributedTimestamp(a)
		const bt = attributedTimestamp(b)
		return bt - at
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
		defaultMessage: 'Search {count} {count, plural, one {project} other {projects}}...',
	},
	infoBannerTitle: {
		id: 'project.settings.permissions.info-banner.title',
		defaultMessage: 'Learn about distribution permissions',
	},
	infoBannerDescription: {
		id: 'project.settings.permissions.info-banner.description',
		defaultMessage: `If you include content that isn’t hosted on Modrinth, you need to let us know where it’s from and verify that you have permission to distribute the files. Check out <link>our guide</link> to learn more and get started!`,
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
		defaultMessage: `None of your project's versions contain external content, so you don't need to worry about obtaining permissions.`,
	},
	completedTitle: {
		id: 'project.settings.permissions.completed.title',
		defaultMessage: `Permissions completed!`,
	},
	completedDescription: {
		id: 'project.settings.permissions.completed.description',
		defaultMessage:
			'All external content has permission information and attributions have been provided.',
	},
	failTitle: {
		id: 'project.settings.permissions.fail.title',
		defaultMessage: `Some content can't be included`,
	},
	failDescription: {
		id: 'project.settings.permissions.fail.description',
		defaultMessage: `You may not have permission to redistribute some of the external content in your project. In order to publish on Modrinth, please remove this content or provide proof that you do have permission to use it.`,
	},
	attentionNeededTitle: {
		id: 'project.settings.permissions.attention-needed.title',
		defaultMessage: `Unknown external content`,
	},
	attentionNeededDescriptionApproved: {
		id: 'project.settings.permissions.attention-needed.description.proj-approved',
		defaultMessage: `Please provide proof that you have permission to redistribute all of the following files. Once completed, withheld versions will be automatically published.`,
	},
	attentionNeededDescriptionDraft: {
		id: 'project.settings.permissions.attention-needed.description.proj-draft',
		defaultMessage: `Please provide proof that you have permission to redistribute all of the following files before submitting your project for review.`,
	},
	noResults: {
		id: 'project.settings.permissions.no-results',
		defaultMessage: 'No external files match your search.',
	},
	collapseAll: {
		id: 'project.settings.permissions.collapse-all',
		defaultMessage: 'Collapse all',
	},
	expandAll: {
		id: 'project.settings.permissions.expand-all',
		defaultMessage: 'Expand all',
	},
})

function defaultCardCollapsed(group: Labrinth.Attribution.Internal.AttributionGroup): boolean {
	if (group?.attribution?.kind === 'globally_allowed') {
		return true
	}
	if (!isModerator.value) {
		const hasAttribution = !!group.attribution
		const rejectedProof = group.attribution?.moderation_status?.kind === 'bad_proof'
		return hasAttribution && !rejectedProof
	}
	return false
}

function getCardCollapsed(group: Labrinth.Attribution.Internal.AttributionGroup): boolean {
	return cardCollapsedById.value[group.id] ?? defaultCardCollapsed(group)
}

function setCardCollapsed(groupId: string, collapsed: boolean) {
	cardCollapsedById.value = { ...cardCollapsedById.value, [groupId]: collapsed }
}

const allCardsCollapsed = computed(() => {
	const groups = filteredGroups.value
	if (groups.length === 0) {
		return true
	}
	return groups.every((group) => getCardCollapsed(group))
})

const expandCollapseAllLabel = computed(() =>
	formatMessage(allCardsCollapsed.value ? messages.expandAll : messages.collapseAll),
)

function toggleAllCardsCollapsed() {
	const collapsed = !allCardsCollapsed.value
	const next = { ...cardCollapsedById.value }
	for (const group of filteredGroups.value) {
		next[group.id] = collapsed
	}
	cardCollapsedById.value = next
}

function dismissInfoBanner() {
	flags.value.dismissedExternalProjectsInfo = true
	saveFeatureFlags()
}
</script>
<template>
	<Admonition
		v-if="!flags.dismissedExternalProjectsInfo && !isModerator"
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
		<template v-if="!isModerator">
			<Admonition
				v-if="stats.pending === 0 && stats.noPermission === 0"
				type="success"
				class="mb-4"
				:header="formatMessage(messages.completedTitle)"
				:body="formatMessage(messages.completedDescription)"
			/>
			<Admonition
				v-else-if="stats.noPermission > 0"
				type="critical"
				class="mb-4"
				:header="formatMessage(messages.failTitle)"
				:body="formatMessage(messages.failDescription)"
			/>
			<Admonition
				v-else-if="stats.pending > 0"
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
		</template>
		<div class="grid grid-cols-[1fr_auto_auto] gap-2">
			<StyledInput
				v-model="searchQuery"
				type="text"
				autocomplete="off"
				clearable
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
					class="!w-full flex-grow sm:!w-[220px] sm:flex-grow-0 [&>span]:h-[40px]"
					:options="sortTypes"
					:placeholder="formatMessage(commonMessages.sortByLabel)"
				>
					<template #selected>
						<span class="flex flex-row gap-2 align-middle font-semibold">
							<ArrowDownWideNarrowIcon
								v-if="currentSortType === 'status'"
								class="size-5 flex-shrink-0 text-secondary"
							/>
							<ArrowDown10Icon
								v-else-if="currentSortType === 'most_files'"
								class="size-5 flex-shrink-0 text-secondary"
							/>
							<ClockArrowDownIcon
								v-else-if="currentSortType === 'recently_edited'"
								class="size-5 flex-shrink-0 text-secondary"
							/>
							<ArrowDownWideNarrowIcon
								v-else-if="currentSortType === 'rejected'"
								class="size-5 flex-shrink-0 text-secondary"
							/>
							<span class="truncate text-contrast">{{ currentSortLabel }}</span>
						</span>
					</template>
				</Combobox>
			</div>
			<ButtonStyled>
				<button type="button" class="!h-[40px]" @click="toggleAllCardsCollapsed">
					<UnfoldVerticalIcon
						v-if="allCardsCollapsed"
						class="size-5 flex-shrink-0 text-secondary"
					/>
					<FoldVerticalIcon v-else class="size-5 flex-shrink-0 text-secondary" />
					{{ expandCollapseAllLabel }}
				</button>
			</ButtonStyled>
		</div>
		<div class="mt-4 flex flex-col gap-3">
			<TransitionGroup name="list">
				<ExternalProjectPermissionsCard
					v-for="group in filteredGroups"
					:key="group.id"
					:collapsed="getCardCollapsed(group)"
					:project-id="project.id"
					:group="group"
					:is-moderator="isModerator"
					@update:collapsed="setCardCollapsed(group.id, $event)"
				/>
				<EmptyState
					v-if="filteredGroups.length === 0"
					:heading="formatMessage(messages.noResults)"
					type="no-search-result"
				/>
			</TransitionGroup>
		</div>
	</template>
	<div v-else-if="attributionError">
		<p v-if="attributionError" class="my-12 text-center text-red">
			{{ attributionError }}
		</p>
	</div>
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
