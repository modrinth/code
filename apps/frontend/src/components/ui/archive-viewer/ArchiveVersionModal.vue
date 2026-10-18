<template>
	<NewModal
		ref="modal"
		:max-width="'min(928px, calc(95vw - 10rem))'"
		:width="'min(928px, calc(95vw - 10rem))'"
		no-padding
		:on-show="onShow"
	>
		<template #title>
			<Avatar :src="project.icon_url" alt="" size="2.5rem" no-shadow />
			<span class="text-lg font-extrabold text-contrast">{{ formatMessage(messages.title) }}</span>
		</template>
		<div class="flex h-[min(550px,calc(95vh-12rem))]">
			<div class="flex w-[300px] shrink-0 flex-col bg-surface-3">
				<div class="flex flex-col gap-2 p-4 pb-2" @keydown="onListKeydown">
					<Input
						v-model="search"
						:icon="SearchIcon"
						:placeholder="formatMessage(messages.searchPlaceholder)"
						:aria-label="formatMessage(messages.searchPlaceholder)"
						:aria-controls="listId"
						wrapper-class="w-full"
						clearable
					/>
					<div class="flex flex-wrap gap-1">
						<Button
							v-for="channel in CHANNELS"
							:key="channel"
							type="quiet"
							size="xs"
							:aria-pressed="channels.includes(channel)"
							:class="{ '!bg-surface-5 !text-contrast': channels.includes(channel) }"
							@click="toggleChannel(channel)"
						>
							<VersionChannelIndicator :channel="channel" size="xs" />
							{{ formatMessage(channelMessages[channel]) }}
						</Button>
					</div>
				</div>
				<span v-if="loading && !versions.length" class="flex items-center gap-2 p-4 text-secondary">
					<SpinnerIcon class="size-4 animate-spin" aria-hidden="true" />
					{{ formatMessage(messages.loading) }}
				</span>
				<span v-else-if="!filtered.length" class="p-4 text-center text-sm text-secondary">
					{{ formatMessage(messages.noVersions) }}
				</span>
				<div
					v-else
					:id="listId"
					v-bind="containerProps"
					class="min-h-0 flex-1 overscroll-contain px-2 pb-2"
					role="listbox"
					:aria-label="formatMessage(messages.title)"
					:aria-activedescendant="highlighted ? optionId(highlighted.id) : undefined"
					tabindex="0"
					@keydown="onListKeydown"
				>
					<div v-bind="wrapperProps">
						<div
							v-for="{ data: entry } in list"
							:id="optionId(entry.id)"
							:key="entry.id"
							role="option"
							:aria-selected="entry.id === highlighted?.id"
							class="flex h-[52px] cursor-pointer items-center gap-2 rounded-xl px-3"
							:class="entry.id === highlighted?.id ? 'bg-brand-highlight' : 'hover:bg-surface-4'"
							@click="highlightedId = entry.id"
							@dblclick="choose(entry)"
						>
							<VersionChannelIndicator :channel="entry.version_type" size="sm" class="shrink-0" />
							<div class="flex min-w-0 flex-1 flex-col">
								<span class="truncate font-semibold text-contrast" :title="entry.version_number">
									{{ entry.version_number }}
								</span>
								<span class="truncate text-xs text-secondary">
									{{ formatShortDate(entry.date_published) }}
								</span>
							</div>
							<CheckIcon
								v-if="entry.id === currentVersionId"
								v-tooltip="formatMessage(messages.current)"
								class="size-4 shrink-0 text-green"
								:aria-label="formatMessage(messages.current)"
							/>
						</div>
					</div>
				</div>
			</div>

			<div class="flex min-w-0 flex-1 flex-col bg-surface-2" aria-live="polite">
				<template v-if="highlighted">
					<div class="flex flex-col gap-2 border-0 border-b border-solid border-surface-4 p-4">
						<div class="flex flex-wrap items-center gap-2">
							<span class="text-xl font-semibold text-contrast">{{
								highlighted.version_number
							}}</span>
							<TagItem>{{ formatMessage(channelMessages[highlighted.version_type]) }}</TagItem>
							<TagItem v-if="highlighted.status !== 'listed'" class="font-semibold">
								{{
									formatMessage(
										versionStatusMessages[highlighted.status] ?? versionStatusMessages.unknown,
									)
								}}
							</TagItem>
							<span
								v-tooltip="formatFullDate(highlighted.date_published)"
								class="ml-auto text-sm text-secondary"
							>
								{{ formatShortDate(highlighted.date_published) }}
							</span>
						</div>
						<span v-if="highlighted.name !== highlighted.version_number" class="text-secondary">
							{{ highlighted.name }}
						</span>
						<div class="flex flex-wrap gap-1">
							<TagTagItem v-for="loader in highlighted.loaders" :key="loader" :tag="loader" />
							<TagItem v-for="gameVersion in shownGameVersions" :key="gameVersion">
								{{ gameVersion }}
							</TagItem>
							<TagItem v-if="hiddenGameVersions.length" v-tooltip="hiddenGameVersions.join(', ')">
								{{ formatMessage(messages.moreGameVersions, { count: hiddenGameVersions.length }) }}
							</TagItem>
						</div>
						<ul class="m-0 flex list-none flex-col gap-1 p-0 text-sm">
							<li
								v-for="file in highlighted.files"
								:key="file.url"
								class="flex min-w-0 items-center gap-2"
							>
								<FileIcon class="size-4 shrink-0 text-secondary" aria-hidden="true" />
								<span class="truncate text-primary" :title="file.filename">{{
									file.filename
								}}</span>
								<span class="shrink-0 text-secondary">{{ formatBytes(file.size) }}</span>
								<TagItem v-if="file.primary" class="shrink-0">
									{{ formatMessage(messages.primary) }}
								</TagItem>
							</li>
						</ul>
					</div>
					<div class="min-h-0 flex-1 overflow-y-auto p-4">
						<span
							v-if="changelogQuery.isPending.value"
							class="flex items-center gap-2 text-secondary"
						>
							<SpinnerIcon class="size-4 animate-spin" aria-hidden="true" />
							{{ formatMessage(messages.loadingChangelog) }}
						</span>
						<div
							v-else-if="changelog"
							class="markdown-body"
							v-html="renderHighlightedString(changelog)"
						/>
						<span v-else class="italic text-secondary">{{
							formatMessage(messages.noChangelog)
						}}</span>
					</div>
				</template>
				<span v-else class="m-auto text-secondary">{{ formatMessage(messages.selectPrompt) }}</span>
			</div>
		</div>
		<div
			class="flex items-center justify-end gap-2 border-0 border-t border-solid border-surface-4 p-4"
		>
			<Button type="outlined" @click="hide">
				<XIcon aria-hidden="true" />
				{{ formatMessage(commonMessages.cancelButton) }}
			</Button>
			<Button
				type="colored"
				color="brand"
				:disabled="!highlighted || highlighted.id === currentVersionId"
				@click="highlighted && choose(highlighted)"
			>
				<CheckIcon aria-hidden="true" />
				{{ formatMessage(messages.open, { version: highlighted?.version_number ?? '...' }) }}
			</Button>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, FileIcon, SearchIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	Input,
	NewModal,
	TagItem,
	TagTagItem,
	useFormatBytes,
	useFormatDateTime,
	useVIntl,
	VersionChannelIndicator,
} from '@modrinth/ui'
import { renderHighlightedString } from '@modrinth/utils/highlightjs/index'
import { useQuery } from '@tanstack/vue-query'
import { useVirtualList } from '@vueuse/core'
import { computed, nextTick, ref, useId, useTemplateRef, watch } from 'vue'

import { channelMessages, versionStatusMessages } from './messages'

type Version = Labrinth.Versions.v3.Version
type Channel = Version['version_type']

/**
 * Picks a version of the project to view, from a searchable list (only the rows in view are
 * rendered, as projects can have thousands of versions) beside the highlighted version's
 * details and changelog, which is fetched once it's highlighted.
 */
const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	/** The project's versions, newest first. */
	versions: Version[]
	loading?: boolean
	currentVersionId?: string
}>()

const emit = defineEmits<{
	select: [id: string]
}>()

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const formatBytes = useFormatBytes()
const formatShortDate = useFormatDateTime({ dateStyle: 'medium' })
const formatFullDate = useFormatDateTime({ dateStyle: 'full', timeStyle: 'long' })

const messages = defineMessages({
	title: { id: 'archive.version-modal.title', defaultMessage: 'Select a version' },
	searchPlaceholder: {
		id: 'archive.version-modal.search-placeholder',
		defaultMessage: 'Search versions...',
	},
	loading: { id: 'archive.version-modal.loading', defaultMessage: 'Loading versions...' },
	noVersions: { id: 'archive.version-modal.no-versions', defaultMessage: 'No versions found' },
	current: { id: 'archive.version-modal.current', defaultMessage: 'Currently open' },
	primary: { id: 'archive.version-modal.primary', defaultMessage: 'Primary' },
	moreGameVersions: {
		id: 'archive.version-modal.more-game-versions',
		defaultMessage: '+{count} more',
	},
	loadingChangelog: {
		id: 'archive.version-modal.loading-changelog',
		defaultMessage: 'Loading changelog...',
	},
	noChangelog: { id: 'archive.version-modal.no-changelog', defaultMessage: 'No changelog' },
	selectPrompt: {
		id: 'archive.version-modal.select-prompt',
		defaultMessage: 'Select a version to see its details',
	},
	open: { id: 'archive.version-modal.open', defaultMessage: 'Open {version}' },
})

const CHANNELS: Channel[] = ['release', 'beta', 'alpha']
const ROW_HEIGHT = 52
const SHOWN_GAME_VERSIONS = 8

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const listId = useId()
const search = ref('')
const channels = ref<Channel[]>([])
const highlightedId = ref<string | null>(null)

function optionId(id: string) {
	return `${listId}-${id}`
}

function toggleChannel(channel: Channel) {
	channels.value = channels.value.includes(channel)
		? channels.value.filter((entry) => entry !== channel)
		: [...channels.value, channel]
}

const filtered = computed(() => {
	const term = search.value.trim().toLocaleLowerCase()
	return props.versions.filter(
		(entry) =>
			(!channels.value.length || channels.value.includes(entry.version_type)) &&
			(!term ||
				entry.version_number.toLocaleLowerCase().includes(term) ||
				entry.name.toLocaleLowerCase().includes(term) ||
				entry.id.toLocaleLowerCase() === term ||
				entry.game_versions.some((gameVersion) => gameVersion.toLocaleLowerCase() === term)),
	)
})

const { list, containerProps, wrapperProps, scrollTo } = useVirtualList(filtered, {
	itemHeight: ROW_HEIGHT,
	overscan: 6,
})

watch([search, channels], () => scrollTo(0))

const highlighted = computed(
	() => props.versions.find((entry) => entry.id === highlightedId.value) ?? null,
)

const shownGameVersions = computed(
	() => highlighted.value?.game_versions.slice(0, SHOWN_GAME_VERSIONS) ?? [],
)
const hiddenGameVersions = computed(
	() => highlighted.value?.game_versions.slice(SHOWN_GAME_VERSIONS) ?? [],
)

/** The project's versions are listed without changelogs, so each is fetched as it's previewed. */
const changelogQuery = useQuery({
	queryKey: computed(() => ['version', 'v3', highlightedId.value]),
	queryFn: () => client.labrinth.versions_v3.getVersion(highlightedId.value!),
	enabled: computed(() => !!highlightedId.value),
	staleTime: 5 * 60 * 1000,
})
const changelog = computed(() => changelogQuery.data.value?.changelog ?? null)

/** Moves the highlight with the arrow keys and opens it with Enter, from the search or the list. */
function onListKeydown(event: KeyboardEvent) {
	const index = filtered.value.findIndex((entry) => entry.id === highlightedId.value)
	let next = index
	if (event.key === 'ArrowDown') next = Math.min(index + 1, filtered.value.length - 1)
	else if (event.key === 'ArrowUp') next = Math.max(index - 1, 0)
	else if (event.key === 'Home' && event.target === event.currentTarget) next = 0
	else if (event.key === 'End' && event.target === event.currentTarget)
		next = filtered.value.length - 1
	else if (event.key === 'Enter' && highlighted.value) {
		event.preventDefault()
		choose(highlighted.value)
		return
	} else return
	event.preventDefault()
	const entry = filtered.value[next]
	if (!entry) return
	highlightedId.value = entry.id
	revealRow(next)
}

/** Scrolls the row into view only when it's outside it, keeping the list still otherwise. */
function revealRow(index: number) {
	const container = containerProps.ref.value
	if (!container) return
	const top = index * ROW_HEIGHT
	if (top < container.scrollTop) container.scrollTop = top
	else if (top + ROW_HEIGHT > container.scrollTop + container.clientHeight) {
		container.scrollTop = top + ROW_HEIGHT - container.clientHeight
	}
}

function choose(entry: Version) {
	if (entry.id !== props.currentVersionId) emit('select', entry.id)
	hide()
}

async function onShow() {
	search.value = ''
	channels.value = []
	highlightedId.value = props.currentVersionId ?? props.versions[0]?.id ?? null
	await nextTick()
	const index = filtered.value.findIndex((entry) => entry.id === highlightedId.value)
	if (index > 0) scrollTo(index)
}

function show(event?: MouseEvent) {
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
