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
		table-layout="auto"
		@row-click="openVersionRow"
	>
		<template #cell-channel="{ row: version }">
			<div class="flex items-center justify-center">
				<VersionChannelIndicator
					v-tooltip="`Toggle filter for ${version.version_type}`"
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
							<TagItem> <CircleAlertIcon /> {{ formatMessage(messages.withheld) }}</TagItem>
						</div>
					</div>
				</AutoLink>
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
		</template>

		<template #cell-gameVersions="{ row: version }">
			<div class="flex flex-wrap gap-1">
				<TagItem
					v-for="gameVersion in getDisplayGameVersions(version).slice(0, maxGameVersionTags)"
					:key="`version-tag-${gameVersion}`"
					v-tooltip="`Toggle filter for ${gameVersion}`"
					data-no-row-click
					:action="() => versionFilters?.toggleFilters('gameVersion', version.game_versions)"
				>
					{{ gameVersion }}
				</TagItem>
				<Menu
					v-if="getDisplayGameVersions(version).length > maxGameVersionTags"
					data-no-row-click
					:delay="{ hide: 50, show: 0 }"
					no-auto-focus
					class="cursor-default"
				>
					<TagItem tabindex="0">
						+{{ getDisplayGameVersions(version).length - maxGameVersionTags }}
					</TagItem>
					<template #popper>
						<div class="flex max-w-[20rem] flex-wrap gap-1">
							<TagItem
								v-for="gameVersion in getDisplayGameVersions(version).slice(maxGameVersionTags)"
								:key="`overflow-version-tag-${gameVersion}`"
								:action="() => versionFilters?.toggleFilters('gameVersion', version.game_versions)"
							>
								{{ gameVersion }}
							</TagItem>
						</div>
					</template>
				</Menu>
			</div>
		</template>

		<template #cell-platforms="{ row: version }">
			<div class="flex flex-wrap gap-1">
				<template v-if="version.noModLoader">
					<TagItem class="border !border-solid border-surface-5"> No mod loader </TagItem>
				</template>
				<template v-else>
					<TagItem
						v-for="platform in version.loaders"
						:key="`platform-tag-${platform}`"
						v-tooltip="`Toggle filter for ${platform}`"
						data-no-row-click
						:style="`--_color: var(--color-platform-${platform})`"
						:action="() => versionFilters?.toggleFilter('platform', platform)"
					>
						<component :is="getLoaderIcon(platform)" v-if="getLoaderIcon(platform)" />
						<FormattedTag :tag="platform" enforce-type="loader" />
					</TagItem>
				</template>
			</div>
		</template>

		<template v-if="showEnvironmentColumn" #cell-environment="{ row: version }">
			<div class="flex flex-wrap gap-1">
				<TagItem
					v-for="(tag, tagIdx) in getEnvironmentTags(version.environment)"
					:key="`env-tag-${tagIdx}`"
					data-no-row-click
					class="text-center"
				>
					<component :is="tag.icon" />
					{{ formatMessage(tag.label).replace('and', '&') }}
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
			<div class="flex items-center justify-end gap-0.5 h-full cursor-default" data-no-row-click>
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
									v-tooltip="`Toggle filter for ${version.version_type}`"
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
							class="flex items-start justify-end gap-1 max-[400px]:flex-col max-[400px]:justify-start smart-clickable:allow-pointer-events"
						>
							<slot name="actions" :version="version"></slot>
						</div>
					</div>

					<div class="flex flex-col justify-center gap-3">
						<div class="flex flex-row flex-wrap items-center gap-1.5">
							<TagItem
								v-for="gameVersion in getDisplayGameVersions(version).slice(0, maxGameVersionTags)"
								:key="`version-tag-${gameVersion}`"
								v-tooltip="`Toggle filter for ${gameVersion}`"
								class="smart-clickable:allow-pointer-events"
								:action="() => versionFilters?.toggleFilters('gameVersion', version.game_versions)"
							>
								{{ gameVersion }}
							</TagItem>
							<Menu
								v-if="getDisplayGameVersions(version).length > maxGameVersionTags"
								:delay="{ hide: 50, show: 0 }"
								no-auto-focus
								class="cursor-default smart-clickable:allow-pointer-events"
							>
								<TagItem tabindex="0">
									+{{ getDisplayGameVersions(version).length - maxGameVersionTags }}
								</TagItem>
								<template #popper>
									<div class="flex max-w-[20rem] flex-wrap gap-1">
										<TagItem
											v-for="gameVersion in getDisplayGameVersions(version).slice(
												maxGameVersionTags,
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
									v-for="platform in version.loaders"
									:key="`platform-tag-${platform}`"
									v-tooltip="`Toggle filter for ${platform}`"
									class="smart-clickable:allow-pointer-events"
									:style="`--_color: var(--color-platform-${platform})`"
									:action="() => versionFilters?.toggleFilter('platform', platform)"
								>
									<component :is="getLoaderIcon(platform)" v-if="getLoaderIcon(platform)" />
									<FormattedTag :tag="platform" enforce-type="loader" />
								</TagItem>
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
	FormattedTag,
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
import { getEnvironmentTags } from './settings/environment/environments'

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const { formatCompactNumber } = useCompactNumber()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})
const formatBytes = useFormatBytes()

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
			label: 'Name',
			cellClass: '!overflow-visible py-3 pr-4 min-w-[5rem]',
		},
		{
			key: 'gameVersions',
			width: '18%',
			label: 'Game version',
			cellClass: visibleCellClass,
		},
		{
			key: 'platforms',
			label: 'Platforms',
			cellClass: visibleCellClass,
		},
	]

	if (props.showEnvironmentColumn) {
		columns.push({
			key: 'environment',
			label: 'Environment',
			cellClass: visibleCellClass,
		})
	}

	columns.push(
		{
			key: 'published',
			label: 'Published',
			cellClass: '!overflow-visible align-middle pr-2.5 w-max',
		},
		{
			key: 'downloads',
			label: 'Downloads',
			cellClass: '!overflow-visible align-middle',
		},
		{
			key: 'actions',
			align: 'right',
			width: '4.5rem',
			headerClass: 'text-secondary',
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

const maxGameVersionTags = 6

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
	return props.versionLink
		? 'group version-row-link cursor-pointer transition-[filter] [&:hover:not(:has([data-no-row-click]:hover))]:brightness-[115%]'
		: 'group'
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
})
</script>

<style scoped>
:deep(.version-row-link:hover:not(:has([data-no-row-click]:hover)) .version-row-name) {
	text-decoration-line: underline;
}
</style>
