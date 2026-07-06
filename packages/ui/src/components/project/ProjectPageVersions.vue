<template>
	<div class="mb-3 flex flex-col gap-3">
		<div class="flex flex-wrap justify-between gap-2">
			<VersionFilterControl
				ref="versionFilters"
				:versions="normalizedVersions"
				:game-versions="gameVersions"
				:base-id="`${baseId}-filter`"
				@update:query="updateQuery"
			/>

			<ButtonStyled v-if="openModal" :color="createVersionButtonSecondary ? 'standard' : 'green'">
				<button @click="openModal"><PlusIcon /> Create version</button>
			</ButtonStyled>

			<Pagination
				v-if="!openModal"
				:page="currentPage"
				class="mt-auto"
				:count="Math.ceil(filteredVersions.length / pageSize)"
				@switch-page="switchPage"
			/>
		</div>

		<div
			v-if="openModal && filteredVersions.length > pageSize"
			class="flex flex-wrap items-center justify-between gap-2"
		>
			<span>
				Showing {{ (currentPage - 1) * pageSize + 1 }} to
				{{ Math.min(currentPage * pageSize, filteredVersions.length) }} of
				{{ filteredVersions.length }}
			</span>

			<Pagination
				:page="currentPage"
				class="mt-auto"
				:count="Math.ceil(filteredVersions.length / pageSize)"
				@switch-page="switchPage"
			/>
		</div>
	</div>

	<Table
		v-if="versions.length > 0"
		class="hidden sm:block"
		:columns="versionColumns"
		:data="currentVersionRows"
		row-key="id"
		:row-class="getVersionRowClass"
		:row-clickable="!!versionLink"
		:row-below-visible="isFileRowVisible"
		table-layout="auto"
		@row-click="openVersionRow"
	>
		<template #cell-channel="{ row: version }">
			<div class="flex items-center justify-center">
				<VersionChannelIndicator
					v-tooltip="getFilterTooltip(version.version_type)"
					:channel="version.version_type"
					class="cursor-pointer"
					data-no-row-click
					@click.stop="versionFilters?.toggleFilter('channel', version.version_type)"
				/>
			</div>
		</template>

		<template #cell-name="{ row: version }">
			<div class="flex min-w-0 flex-col gap-2">
				<AutoLink
					v-tooltip="`${version.version_number} - ${version.name}`"
					:to="versionLink?.(version)"
					class="flex min-w-0 flex-col gap-1 w-fit"
					:link-class="versionLink ? 'focus-visible:underline' : ''"
					:title="`${version.version_number} - ${version.name}`"
				>
					<div class="flex min-w-0 items-center gap-2">
						<div
							class="overflow-hidden text-ellipsis font-medium text-contrast"
							:class="versionLink ? 'version-row-name' : ''"
						>
							{{ version.version_number }}
						</div>
						<div
							v-if="version.files_missing_attribution"
							v-tooltip="formatMessage(messages.withheldTooltip)"
							:style="{
								'--_bg-color': 'var(--color-orange-bg)',
								'--_color': 'var(--color-orange)',
							}"
						>
							<TagItem class="w-fit max-w-full truncate">
								<CircleAlertIcon />
								<span class="min-w-0 truncate">{{ formatMessage(messages.withheld) }}</span>
							</TagItem>
						</div>
					</div>
				</AutoLink>
			</div>
		</template>

		<template #row-below="{ row: version }">
			<div class="tag-list px-4 pb-3 -mt-0.5">
				<div
					v-for="(file, fileIdx) in version.files"
					:key="`file-tag-${fileIdx}`"
					:class="`flex items-center gap-1 text-wrap rounded-full bg-button-bg px-2 py-0.5 text-xs font-medium ${file.primary || fileIdx === 0 ? 'text-contrast' : 'text-primary'}`"
				>
					<StarIcon v-if="file.primary || fileIdx === 0" class="shrink-0" />
					{{ file.filename }} - {{ formatBytes(file.size) }}
				</div>
			</div>
		</template>

		<template #cell-gameVersions="{ row: version }">
			<div class="flex min-w-0 w-full max-w-[12rem] flex-wrap gap-1">
				<TagItem
					v-for="gameVersion in getDisplayGameVersions(version).slice(0, MAX_GAME_VERSION_TAGS)"
					:key="`version-tag-${gameVersion}`"
					v-tooltip="getFilterTooltip(gameVersion)"
					data-no-row-click
					class="w-fit max-w-full truncate"
					:action="() => versionFilters?.toggleFilters('gameVersion', version.game_versions)"
				>
					<span class="min-w-0 truncate">{{ gameVersion }}</span>
				</TagItem>
				<Menu
					v-if="getDisplayGameVersions(version).length > MAX_GAME_VERSION_TAGS"
					data-no-row-click
					:delay="{ hide: 50, show: 0 }"
					no-auto-focus
					class="w-full min-w-0 cursor-default"
				>
					<TagItem class="w-fit max-w-full truncate" tabindex="0">
						<span class="min-w-0 truncate">
							+{{ getDisplayGameVersions(version).length - MAX_GAME_VERSION_TAGS }}
						</span>
					</TagItem>
					<template #popper>
						<div class="flex max-w-[20rem] flex-wrap gap-1">
							<TagItem
								v-for="gameVersion in getDisplayGameVersions(version).slice(MAX_GAME_VERSION_TAGS)"
								:key="`overflow-version-tag-${gameVersion}`"
								class="w-fit max-w-full truncate"
								:action="() => versionFilters?.toggleFilters('gameVersion', version.game_versions)"
							>
								<span class="min-w-0 truncate">{{ gameVersion }}</span>
							</TagItem>
						</div>
					</template>
				</Menu>
			</div>
		</template>

		<template #cell-platforms="{ row: version }">
			<div class="flex min-w-0 w-full max-w-[12rem] flex-wrap gap-1">
				<template v-if="version.noModLoader">
					<TagItem class="w-fit max-w-full truncate border !border-solid border-surface-5">
						<span class="min-w-0 truncate">No mod loader</span>
					</TagItem>
				</template>
				<template v-else>
					<TagItem
						v-for="platform in version.loaders.slice(0, MAX_PLATFORM_TAGS)"
						:key="`platform-tag-${platform}`"
						v-tooltip="getPlatformTooltip(platform)"
						data-no-row-click
						class="w-fit max-w-full truncate"
						:style="`--_color: var(--color-platform-${platform})`"
						:action="() => versionFilters?.toggleFilter('platform', platform)"
					>
						<component :is="getLoaderIcon(platform)" v-if="getLoaderIcon(platform)" />
						<span class="min-w-0 truncate">{{ getPlatformLabel(platform) }}</span>
					</TagItem>
					<Menu
						v-if="version.loaders.length > MAX_PLATFORM_TAGS"
						data-no-row-click
						:delay="{ hide: 50, show: 0 }"
						no-auto-focus
						class="w-full min-w-0 cursor-default"
					>
						<TagItem class="w-fit max-w-full truncate" tabindex="0">
							<span class="min-w-0 truncate">
								+{{ version.loaders.length - MAX_PLATFORM_TAGS }}
							</span>
						</TagItem>
						<template #popper>
							<div class="flex max-w-[20rem] flex-wrap gap-1">
								<TagItem
									v-for="platform in version.loaders.slice(MAX_PLATFORM_TAGS)"
									:key="`overflow-platform-tag-${platform}`"
									v-tooltip="getPlatformTooltip(platform)"
									class="w-fit max-w-full truncate"
									:style="`--_color: var(--color-platform-${platform})`"
									:action="() => versionFilters?.toggleFilter('platform', platform)"
								>
									<component :is="getLoaderIcon(platform)" v-if="getLoaderIcon(platform)" />
									<span class="min-w-0 truncate">{{ getPlatformLabel(platform) }}</span>
								</TagItem>
							</div>
						</template>
					</Menu>
				</template>
			</div>
		</template>

		<template v-if="showEnvironmentColumn" #cell-environment="{ row: version }">
			<div class="flex min-w-0 w-full max-w-[12rem] flex-wrap gap-1">
				<TagItem
					v-for="(tag, tagIdx) in getEnvironmentTags(version.environment)"
					:key="`env-tag-${tagIdx}`"
					data-no-row-click
					class="w-fit max-w-full truncate text-center"
				>
					<component :is="tag.icon" />
					<span class="min-w-0 truncate">{{ formatMessage(tag.label).replace('and', '&') }}</span>
				</TagItem>
			</div>
		</template>

		<template #cell-published="{ row: version }">
			<div
				v-tooltip="formatDateTime(version.date_published)"
				class="flex items-center gap-1 text-nowrap font-medium w-max cursor-default"
				data-no-row-click
			>
				{{ formatRelativeTime(new Date(version.date_published)) }}
			</div>
		</template>

		<template #cell-downloads="{ row: version }">
			<div
				v-tooltip="`${version.downloads} downloads`"
				class="flex items-center gap-1 font-medium w-max text-nowrap cursor-default"
				data-no-row-click
			>
				{{ formatCompactNumber(version.downloads) }}
			</div>
		</template>

		<template #cell-actions="{ row: version }">
			<div
				class="flex h-full w-max items-center justify-end gap-0.5 whitespace-nowrap cursor-default"
				data-no-row-click
			>
				<slot name="actions" :version="version"></slot>
			</div>
		</template>
	</Table>

	<!-- MOBILE VERSIONS TABLE/LIST -->
	<div
		v-if="versions.length > 0"
		class="flex flex-col gap-4 rounded-2xl bg-bg-raised p-5 sm:hidden"
	>
		<template v-for="(version, index) in currentVersions" :key="version.id ?? index">
			<div
				class="h-px w-[calc(100%+2.5rem)] bg-surface-5 -ml-5"
				:class="{
					hidden: index === 0,
				}"
			></div>
			<SmartClickable class="group">
				<template v-if="versionLink" #clickable>
					<AutoLink
						:to="versionLink(version)"
						class="rounded-xl outline-none no-click-animation custom-focus-indicator"
						:title="`${version.version_number} - ${version.name}`"
					></AutoLink>
				</template>
				<div
					class="flex flex-col justify-center gap-1.5 rounded-xl transition-colors smart-clickable:outline-on-focus"
					:class="{
						'cursor-pointer': !!versionLink,
					}"
				>
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-1.5">
							<div class="self-center">
								<VersionChannelIndicator
									v-tooltip="getFilterTooltip(version.version_type)"
									:channel="version.version_type"
									class="cursor-pointer smart-clickable:allow-pointer-events"
									size="sm"
									@click="versionFilters?.toggleFilter('channel', version.version_type)"
								/>
							</div>

							<div class="overflow-hidden text-ellipsis font-medium text-base text-contrast">
								{{ version.version_number }}
							</div>
							<div
								v-if="version.files_missing_attribution"
								v-tooltip="formatMessage(messages.withheldTooltip)"
								:style="{
									'--_bg-color': 'var(--color-orange-bg)',
									'--_color': 'var(--color-orange)',
								}"
							>
								<TagItem> <CircleAlertIcon /> {{ formatMessage(messages.withheld) }}</TagItem>
							</div>
						</div>

						<div
							class="flex items-start justify-end gap-1 max-[350px]:flex-col max-[350px]:justify-start smart-clickable:allow-pointer-events"
						>
							<slot name="actions" :version="version"></slot>
						</div>
					</div>

					<div class="flex flex-col justify-center gap-3">
						<div class="flex flex-row flex-wrap items-center gap-1.5">
							<TagItem
								v-for="gameVersion in getDisplayGameVersions(version).slice(
									0,
									MAX_GAME_VERSION_TAGS,
								)"
								:key="`version-tag-${gameVersion}`"
								v-tooltip="getFilterTooltip(gameVersion)"
								class="smart-clickable:allow-pointer-events"
								:action="() => versionFilters?.toggleFilters('gameVersion', version.game_versions)"
							>
								{{ gameVersion }}
							</TagItem>
							<Menu
								v-if="getDisplayGameVersions(version).length > MAX_GAME_VERSION_TAGS"
								:delay="{ hide: 50, show: 0 }"
								no-auto-focus
								class="cursor-default smart-clickable:allow-pointer-events"
							>
								<TagItem tabindex="0">
									+{{ getDisplayGameVersions(version).length - MAX_GAME_VERSION_TAGS }}
								</TagItem>
								<template #popper>
									<div class="flex max-w-[20rem] flex-wrap gap-1">
										<TagItem
											v-for="gameVersion in getDisplayGameVersions(version).slice(
												MAX_GAME_VERSION_TAGS,
											)"
											:key="`overflow-version-tag-${gameVersion}`"
											:action="
												() => versionFilters?.toggleFilters('gameVersion', version.game_versions)
											"
										>
											{{ gameVersion }}
										</TagItem>
									</div>
								</template>
							</Menu>
							<template v-if="version.noModLoader">
								<TagItem class="border !border-solid border-surface-5"> No mod loader </TagItem>
							</template>
							<template v-else>
								<TagItem
									v-for="platform in version.loaders.slice(0, MAX_PLATFORM_TAGS)"
									:key="`platform-tag-${platform}`"
									v-tooltip="getPlatformTooltip(platform)"
									class="smart-clickable:allow-pointer-events"
									:style="`--_color: var(--color-platform-${platform})`"
									:action="() => versionFilters?.toggleFilter('platform', platform)"
								>
									<component :is="getLoaderIcon(platform)" v-if="getLoaderIcon(platform)" />
									{{ getPlatformLabel(platform) }}
								</TagItem>
								<Menu
									v-if="version.loaders.length > MAX_PLATFORM_TAGS"
									:delay="{ hide: 50, show: 0 }"
									no-auto-focus
									class="cursor-default smart-clickable:allow-pointer-events"
								>
									<TagItem tabindex="0">
										+{{ version.loaders.length - MAX_PLATFORM_TAGS }}
									</TagItem>
									<template #popper>
										<div class="flex max-w-[20rem] flex-wrap gap-1">
											<TagItem
												v-for="platform in version.loaders.slice(MAX_PLATFORM_TAGS)"
												:key="`overflow-platform-tag-${platform}`"
												v-tooltip="getPlatformTooltip(platform)"
												:style="`--_color: var(--color-platform-${platform})`"
												:action="() => versionFilters?.toggleFilter('platform', platform)"
											>
												<component :is="getLoaderIcon(platform)" v-if="getLoaderIcon(platform)" />
												{{ getPlatformLabel(platform) }}
											</TagItem>
										</div>
									</template>
								</Menu>
							</template>
							<template v-if="showEnvironmentColumn">
								<TagItem
									v-for="(tag, tagIdx) in getEnvironmentTags(version.environment)"
									:key="`env-tag-${tagIdx}`"
									class="text-center"
								>
									<component :is="tag.icon" />
									{{ formatMessage(tag.label).replace('and', '&') }}
								</TagItem>
							</template>
						</div>
						<div class="flex flex-row justify-start gap-3">
							<div class="flex cursor-help items-center gap-1 text-nowrap font-medium">
								<CalendarIcon />
								{{ formatRelativeTime(new Date(version.date_published)) }}
							</div>
							<div class="flex items-center gap-1 font-medium">
								<DownloadIcon />
								{{ formatCompactNumber(version.downloads) }}
							</div>
						</div>
					</div>
					<div v-if="showFiles" class="tag-list">
						<div
							v-for="(file, fileIdx) in version.files"
							:key="`file-tag-${fileIdx}`"
							:class="`flex items-center gap-1 text-wrap rounded-full bg-button-bg px-2 py-0.5 text-xs font-medium ${file.primary || fileIdx === 0 ? 'bg-brand-highlight text-contrast' : 'text-primary'}`"
						>
							<StarIcon v-if="file.primary || fileIdx === 0" class="shrink-0" />
							{{ file.filename }} - {{ formatBytes(file.size) }}
						</div>
					</div>
				</div>
			</SmartClickable>
		</template>
	</div>

	<div class="mt-3 flex">
		<Pagination
			:page="currentPage"
			class="ml-auto"
			:count="Math.ceil(filteredVersions.length / pageSize)"
			@switch-page="switchPage"
		/>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CalendarIcon,
	CircleAlertIcon,
	DownloadIcon,
	getLoaderIcon,
	PlusIcon,
	StarIcon,
} from '@modrinth/assets'
import {
	AutoLink,
	ButtonStyled,
	Pagination,
	SmartClickable,
	Table,
	type TableColumn,
	TagItem,
	useCompactNumber,
	useFormatBytes,
	useFormatDateTime,
	VersionChannelIndicator,
	VersionFilterControl,
} from '@modrinth/ui'
import { formatVersionsForDisplay, type GameVersionTag, type Version } from '@modrinth/utils'
import { Menu } from 'floating-vue'
import { computed, type Ref, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useRelativeTime } from '../../composables'
import { defineMessages, useVIntl } from '../../composables/i18n'
import { formatTag } from '../../utils/tag-messages'
import { getEnvironmentTags } from './settings/environment/environments'

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime({ style: 'narrow' })
const { formatCompactNumber } = useCompactNumber()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})
const formatBytes = useFormatBytes()

const MAX_GAME_VERSION_TAGS = 5
const MAX_PLATFORM_TAGS = 3

type VersionWithDisplayUrlEnding = Version & {
	displayUrlEnding: string
	environment?: Labrinth.Projects.v3.Environment
	mrpack_loaders?: string[]
}

type DisplayVersion = VersionWithDisplayUrlEnding & {
	noModLoader: boolean
	files_missing_attribution?: boolean
}

type VersionTableColumn =
	| 'channel'
	| 'name'
	| 'gameVersions'
	| 'platforms'
	| 'environment'
	| 'published'
	| 'downloads'
	| 'actions'
type VersionTableRow = DisplayVersion & Record<string, unknown>

const props = withDefaults(
	defineProps<{
		baseId?: string
		project: {
			project_type: string
			slug?: string
			id: string
		}
		versions: VersionWithDisplayUrlEnding[]
		showFiles?: boolean
		showEnvironmentColumn?: boolean
		currentMember?: boolean
		loaders: Labrinth.Tags.v2.Loader[]
		gameVersions: GameVersionTag[]
		versionLink?: (version: Version) => string
		openModal?: () => void
		createVersionButtonSecondary?: boolean
	}>(),
	{
		baseId: undefined,
		showFiles: false,
		showEnvironmentColumn: false,
		currentMember: false,
		versionLink: undefined,
	},
)

const visibleCellClass = '!overflow-visible py-3 align-middle pr-2.5'

const versionColumns = computed<TableColumn<VersionTableColumn>[]>(() => {
	const columns: TableColumn<VersionTableColumn>[] = [
		{
			key: 'channel',
			width: '4.5rem',
			headerClass: 'text-secondary',
			cellClass: visibleCellClass,
		},
		{
			key: 'name',
			label: 'Version',
			cellClass: '!overflow-visible py-3 pr-4 min-w-[7rem]',
		},
		{
			key: 'gameVersions',
			label: 'Game version',
			cellClass: '!overflow-visible py-3 align-middle pr-2.5 min-w-0 max-w-[12rem]',
		},
		{
			key: 'platforms',
			label: 'Platform',
			cellClass: '!overflow-visible py-3 align-middle pr-2.5 min-w-0 max-w-[12rem]',
		},
	]

	if (props.showEnvironmentColumn) {
		columns.push({
			key: 'environment',
			label: 'Environment',
			cellClass: `${visibleCellClass} min-w-0 max-w-[12rem]`,
		})
	}

	columns.push(
		{
			key: 'published',
			label: 'Published',
			cellClass: '!overflow-visible align-middle pr-2.5 w-max',
			width: '12%',
		},
		{
			key: 'downloads',
			label: 'Downloads',
			cellClass: '!overflow-visible align-middle',
			width: '12%',
		},
		{
			key: 'actions',
			align: 'right',
			headerClass: 'text-secondary',
			width: '1%',
			cellClass: '!overflow-visible align-middle',
		},
	)

	return columns
})

function getModpackLoaders(version: VersionWithDisplayUrlEnding): string[] {
	const loaders = Array.isArray(version.loaders) ? version.loaders : []

	if (props.project.project_type !== 'modpack') {
		return loaders
	}

	const mrpackLoaders = Array.isArray(version.mrpack_loaders) ? version.mrpack_loaders : []
	if (mrpackLoaders.length) {
		return mrpackLoaders
	}

	return loaders.filter((loader) => loader !== 'mrpack')
}

function getGameVersions(version: VersionWithDisplayUrlEnding): string[] {
	return Array.isArray(version.game_versions) ? version.game_versions : []
}

function hasNoModLoader(loaders: string[]): boolean {
	return (
		(props.project.project_type === 'modpack' &&
			loaders.length === 1 &&
			loaders[0] === 'minecraft') ||
		loaders.length === 0
	)
}

function getDisplayGameVersions(version: DisplayVersion): string[] {
	return formatVersionsForDisplay(version.game_versions, props.gameVersions)
}

function getFilterTooltip(filter: string): string {
	return formatMessage(messages.toggleFilterTooltip, { filter })
}

function getPlatformLabel(platform: string): string {
	if (platform === 'modloader') {
		return formatMessage(messages.modloaderShort)
	}

	return formatTag(formatMessage, platform, 'loader')
}

function getPlatformTooltip(platform: string): string {
	return getFilterTooltip(formatTag(formatMessage, platform, 'loader'))
}

function isFileRowVisible(version: VersionTableRow): boolean {
	return props.showFiles && Array.isArray(version.files) && version.files.length > 0
}

const normalizedVersions = computed<DisplayVersion[]>(() =>
	props.versions.map((version) => {
		const loaders = getModpackLoaders(version)
		const gameVersions = getGameVersions(version)
		const noModLoader = hasNoModLoader(loaders)

		return {
			...version,
			game_versions: gameVersions,
			loaders: noModLoader ? [] : loaders,
			noModLoader,
		}
	}),
)

const currentPage: Ref<number> = ref(1)
const pageSize: Ref<number> = ref(20)
const versionFilters: Ref<InstanceType<typeof VersionFilterControl> | null> = ref(null)

const selectedGameVersions: Ref<string[]> = computed(
	() => versionFilters.value?.selectedGameVersions ?? [],
)
const selectedPlatforms: Ref<string[]> = computed(
	() => versionFilters.value?.selectedPlatforms ?? [],
)
const selectedChannels: Ref<string[]> = computed(() => versionFilters.value?.selectedChannels ?? [])

const filteredVersions = computed(() => {
	return normalizedVersions.value.filter(
		(version) =>
			hasAnySelected(version.game_versions, selectedGameVersions.value) &&
			hasAnySelected(version.loaders, selectedPlatforms.value) &&
			isAnySelected(version.version_type, selectedChannels.value),
	)
})

function hasAnySelected(values: string[], selected: string[]) {
	return selected.length === 0 || selected.some((value) => values.includes(value))
}

function isAnySelected(value: string, selected: string[]) {
	return selected.length === 0 || selected.includes(value)
}

const currentVersions = computed(() =>
	filteredVersions.value.slice(
		(currentPage.value - 1) * pageSize.value,
		currentPage.value * pageSize.value,
	),
)
const currentVersionRows = computed<VersionTableRow[]>(
	() => currentVersions.value as VersionTableRow[],
)

const route = useRoute()
const router = useRouter()

if (route.query.page) {
	currentPage.value = Number(route.query.page) || 1
}

function switchPage(page: number) {
	currentPage.value = page

	router.replace({
		query: {
			...route.query,
			page: currentPage.value !== 1 ? currentPage.value : undefined,
		},
	})

	window.scrollTo({ top: 0, behavior: 'smooth' })
}

function getVersionRowClass(): string {
	return props.versionLink ? 'group version-row-link cursor-pointer transition-[filter]' : 'group'
}

function openVersionRow(version: VersionTableRow) {
	const link = props.versionLink?.(version)
	if (!link) return
	router.push(link)
}

function updateQuery(newQueries: Record<string, string | string[] | undefined | null>) {
	if (newQueries.page) {
		currentPage.value = Number(newQueries.page)
	} else if (newQueries.page === undefined) {
		currentPage.value = 1
	}

	router.replace({
		query: {
			...route.query,
			...newQueries,
		},
	})
}

const messages = defineMessages({
	withheld: {
		id: 'project.versions.version.withheld',
		defaultMessage: 'Withheld',
	},
	withheldTooltip: {
		id: 'project.versions.version.withheld.tooltip',
		defaultMessage: 'Version withheld due to missing permissions',
	},
	toggleFilterTooltip: {
		id: 'project.versions.filter.toggle-tooltip',
		defaultMessage: 'Toggle filter for {filter}',
	},
	modloaderShort: {
		id: 'project.versions.platform.modloader.short',
		defaultMessage: 'ModLoader',
	},
})
</script>

<style scoped>
:deep(.version-row-link:hover:not(:has([data-no-row-click]:hover))),
:deep(.version-row-link:hover:not(:has([data-no-row-click]:hover)) + .table-row-below),
:deep(.version-row-link:has(+ .table-row-below:hover)),
:deep(.version-row-link:has(+ .table-row-below:hover) + .table-row-below) {
	filter: brightness(115%);
}

:deep(.version-row-link:hover:not(:has([data-no-row-click]:hover)) .version-row-name),
:deep(.version-row-link:has(+ .table-row-below:hover) .version-row-name) {
	text-decoration-line: underline;
}
</style>
