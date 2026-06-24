<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, FileIcon, SearchIcon } from '@modrinth/assets'
import { capitalizeString, renderHighlightedString } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { useFormatBytes } from '#ui/composables/format-bytes.ts'
import { useFormatDateTime } from '#ui/composables/format-date-time.ts'
import { useCompactNumber, useFormatNumber } from '#ui/composables/format-number.ts'
import { useRelativeTime } from '#ui/composables/how-ago.ts'
import { defineMessage, defineMessages, useVIntl } from '#ui/composables/i18n.ts'
import { injectModrinthClient } from '#ui/providers/api-client.ts'
import {
	commonMessages,
	fileTypeMessages,
	projectCompatibilityMessages,
} from '#ui/utils/common-messages.ts'

import AutoLink from '../base/AutoLink.vue'
import Avatar from '../base/Avatar.vue'
import StyledInput from '../base/StyledInput.vue'
import Table from '../base/Table.vue'
import TagItem from '../base/TagItem.vue'
import TagTagItem from '../base/TagTagItem.vue'
import EnvironmentTags from '../project/EnvironmentTags.vue'
import VersionChannelTag from './VersionChannelTag.vue'
import VersionDependencyItem from './VersionDependencyItem.vue'

const { formatMessage } = useVIntl()

const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})
const formatBytes = useFormatBytes()
const formatNumber = useFormatNumber()
const { formatCompactNumber } = useCompactNumber()

const props = defineProps<{
	version: Labrinth.Versions.v3.Version
	enrichment?: Labrinth.Projects.v2.DependencyInfo
	dependencyLinkCreator: (context: DependencyContext) => string | undefined
	members?: Labrinth.Projects.v3.TeamMember[]
	userLinkCreator?: (user: Labrinth.Users.v3.User) => string | undefined
}>()

const api = injectModrinthClient()

const versionNumber = computed(() => props.version.version_number)
const versionSubtitle = computed(() => props.version.name)
const publishDate = computed(() => formatRelativeTime(props.version.date_published))
const publishDateTooltip = computed(() => formatDateTime(props.version.date_published))

const isModpack = computed(() => props.version.loaders.includes('mrpack'))
const platforms = computed(() =>
	isModpack.value ? props.version.mrpack_loaders : props.version.loaders,
)
const noModpackLoader = computed(
	() =>
		(isModpack.value &&
			props.version.mrpack_loaders?.length === 1 &&
			props.version.mrpack_loaders?.[0] === 'minecraft') ||
		props.version.mrpack_loaders?.length === 0,
)
const primaryFile = computed(
	() => props.version.files?.find((file) => file.primary) ?? props.version.files?.[0] ?? {},
)
const promotedSupplementaryFiles = computed(() =>
	props.version.files.filter(
		(file) =>
			file.file_type &&
			['required-resource-pack', 'optional-resource-pack'].includes(file.file_type),
	),
)

type DependencyContext = {
	dependency: Labrinth.Versions.v3.Dependency
	project?: Labrinth.Projects.v2.Project
	version?: Labrinth.Versions.v2.Version
}

const dependencies = computed<DependencyContext[]>(() =>
	props.version.dependencies.map(
		(dep) =>
			({
				dependency: dep,
				project: props.enrichment?.projects.find((x) => x.id === dep.project_id),
				version: props.enrichment?.versions.find((x) => x.id === dep.version_id),
			}) satisfies DependencyContext,
	),
)

const requiredContent = computed(() =>
	dependencies.value.filter((dep) => dep.dependency.dependency_type === 'required'),
)
const optionalContent = computed(() =>
	dependencies.value.filter((dep) => dep.dependency.dependency_type === 'optional'),
)
const incompatibleContent = computed(() =>
	dependencies.value.filter((dep) => dep.dependency.dependency_type === 'incompatible'),
)
const includedContent = computed(() =>
	dependencies.value
		.filter((context) => context.dependency.dependency_type === 'embedded')
		.map((context) => ({
			icon_url: context.project
				? context.project.icon_url
				: context.dependency.attribution?.icon_url,
			name: context.project ? context.project.title : context.dependency.file_name,
			version: context.version ? context.version.version_number : undefined,
			link: context.project
				? props.dependencyLinkCreator(context)
				: context.dependency.attribution?.link,
			hasProject: !!context.project,
		})),
)

const contentTableColumns = computed(() => {
	const cols = [
		{
			key: 'icon',
			width: '3.75rem',
		},
		{
			key: 'name',
			label: formatMessage(defineMessage({ id: 'version.content.name', defaultMessage: 'Name' })),
		},
	]
	if (includedContent.value.some((x) => x.version)) {
		cols.push({
			key: 'version',
			label: formatMessage(
				defineMessage({ id: 'version.content.version', defaultMessage: 'Version' }),
			),
		})
	}
	return cols
})

const supplementaryResourcesTableColumns = computed(() => [
	{
		key: 'file',
		label: formatMessage(
			defineMessage({ id: 'version.supplementary-resources.file', defaultMessage: 'File' }),
		),
		width: '20rem',
	},
	{
		key: 'type',
		label: formatMessage(
			defineMessage({ id: 'version.supplementary-resources.type', defaultMessage: 'Type' }),
		),
		width: '14rem',
	},
	{
		key: 'size',
		label: formatMessage(
			defineMessage({ id: 'version.supplementary-resources.size', defaultMessage: 'Size' }),
		),
		width: '5rem',
	},
	{
		key: 'actions',
		label: '',
		width: '14rem',
	},
])

const supplementaryResources = computed(() =>
	props.version.files
		.filter((file) => file.primary === false && file !== primaryFile.value)
		.map((file) => ({
			file: file,
		})),
)

const messages = defineMessages({
	compatibility: {
		id: 'version.section.compatibility',
		defaultMessage: 'Compatibility',
	},
	required: {
		id: 'version.section.required-content',
		defaultMessage: 'Required content',
	},
	optionalDeps: {
		id: 'version.section.optional-dependencies',
		defaultMessage: 'Optional dependencies',
	},
	knownIncompatibilities: {
		id: 'version.section.known-incompatibilities',
		defaultMessage: 'Known incompatibilities',
	},
	content: {
		id: 'version.section.content',
		defaultMessage: 'Content',
	},
	includedContent: {
		id: 'version.section.included-content',
		defaultMessage: 'Included content',
	},
	noModpackLoader: {
		id: 'version.section.no-modpack-mod-loader',
		defaultMessage: 'No mod loader',
	},
	changes: {
		id: 'version.section.changes',
		defaultMessage: 'Changes',
	},
	noChanges: {
		id: 'version.section.no-changes',
		defaultMessage: 'No changelog was provided.',
	},
	files: {
		id: 'version.section.files',
		defaultMessage: 'Files',
	},
	supplementaryResources: {
		id: 'version.section.supplementary-resources',
		defaultMessage: 'Supplementary resources',
	},
	searchContent: {
		id: 'version.section.content.search-placeholder',
		defaultMessage: 'Search content...',
	},
})

const contentSearchQuery = ref('')

const formattedDownloads = computed(() => formatNumber(props.version.downloads))
const compactDownloads = computed(() => formatCompactNumber(props.version.downloads))

const authorMember = computed(
	() => props.members?.find((member) => member.user.id === props.version.author_id)?.user,
)
const { data: externalAuthor, isLoading: loadingAuthor } = useQuery({
	queryKey: ['user', props.version.author_id],
	queryFn: () => api.labrinth.users_v3.get(props.version.author_id),
	enabled: computed(() => !authorMember.value),
})

const author = computed(() => authorMember.value ?? externalAuthor.value)
const authorLink = computed(() =>
	author.value ? props.userLinkCreator?.(author.value) : undefined,
)
</script>
<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-wrap gap-4 justify-between items-center">
			<div class="flex flex-col gap-1.5">
				<div class="flex flex-wrap gap-2 items-center">
					<h2 class="m-0 leading-tight font-semibold">{{ versionNumber }}</h2>
					<div>
						<VersionChannelTag :channel="version.version_type" />
					</div>
				</div>
				<div class="flex items-center gap-2 flex-col sm:flex-row">
					<span>{{ versionSubtitle }}</span>
					<span class="bg-surface-5 size-1.5 rounded-full hidden sm:block" />
					<span class="flex items-center gap-2 sm:content">
						<span v-tooltip="publishDateTooltip">{{ publishDate }}</span>
						<span class="bg-surface-5 size-1.5 rounded-full" />
						<span
							v-tooltip="
								compactDownloads !== formattedDownloads
									? capitalizeString(
											formatMessage(commonMessages.projectDownloads, {
												count: props.version.downloads,
											}),
										)
									: undefined
							"
							class="flex items-center gap-1"
						>
							<DownloadIcon class="size-5" /> {{ compactDownloads }}
						</span>
					</span>
				</div>
				<div
					v-if="(author && authorLink) || loadingAuthor"
					class="flex items-center gap-1 text-secondary"
				>
					Uploaded by
					<AutoLink
						v-if="author && authorLink"
						:to="authorLink"
						class="flex items-center gap-1 hover:underline"
					>
						<Avatar :src="author?.avatar_url" size="1.5rem" circle />
						{{ author?.username }}
					</AutoLink>
					<div
						v-else-if="loadingAuthor"
						class="w-32 h-6 bg-surface-3 rounded-md animate-pulse flex"
					></div>
				</div>
			</div>
			<div class="flex gap-2 flex-wrap items-center">
				<slot
					name="headerActions"
					:primary-file="primaryFile"
					:promoted-files="promotedSupplementaryFiles"
				/>
			</div>
		</div>
		<hr class="w-full border-none h-[1px] bg-surface-4 m-0" />
		<section v-if="requiredContent.length > 0" id="dependencies">
			<h3 class="mt-0 mb-2 text-lg font-semibold">{{ formatMessage(messages.required) }}</h3>
			<div class="grid md:grid-cols-2 gap-4">
				<VersionDependencyItem
					v-for="depContext in requiredContent"
					:key="`required-dep-${depContext.dependency.version_id ?? depContext.dependency.project_id ?? depContext.dependency.file_name}`"
					:context="depContext"
					:dependency-link="dependencyLinkCreator(depContext)"
					target="_blank"
					class="bg-surface-3 border border-solid border-surface-4 p-4 rounded-2xl"
				>
					<slot name="dependencyActions" :dependency="depContext" />
				</VersionDependencyItem>
			</div>
		</section>
		<section id="compatibility">
			<h3 class="mt-0 mb-2 text-lg font-semibold">{{ formatMessage(messages.compatibility) }}</h3>
			<div
				class="grid gap-3 md:gap-4 bg-surface-3 border border-solid border-surface-4 p-4 rounded-2xl md:p-0 md:border-0 md:bg-transparent"
				:class="version.environment ? 'md:grid-cols-3' : 'md:grid-cols-2'"
			>
				<div
					class="md:bg-surface-3 md:border md:border-solid md:border-surface-4 md:p-4 md:rounded-2xl"
				>
					{{ formatMessage(projectCompatibilityMessages.minecraftJava) }}
					<div class="flex gap-1 flex-wrap mt-2">
						<TagItem
							v-for="gameVersion in version.game_versions"
							:key="`version-compat-game-version-${gameVersion}`"
							>{{ gameVersion }}</TagItem
						>
					</div>
				</div>
				<div
					class="md:bg-surface-3 md:border md:border-solid md:border-surface-4 md:p-4 md:rounded-2xl"
				>
					{{
						formatMessage(projectCompatibilityMessages.platformsPlural, {
							count: platforms?.length ?? 0,
						})
					}}
					<div class="flex gap-1 flex-wrap mt-2">
						<template v-if="isModpack && noModpackLoader">
							<TagItem class="border !border-solid border-surface-5 hover:no-underline">
								{{ formatMessage(messages.noModpackLoader) }}
							</TagItem>
						</template>
						<template v-else>
							<TagTagItem
								v-for="platform in platforms"
								:key="`version-compat-platform-${platform}`"
								:tag="platform"
							/>
						</template>
					</div>
				</div>
				<div
					v-if="version.environment"
					class="md:bg-surface-3 md:border md:border-solid md:border-surface-4 md:p-4 md:rounded-2xl"
				>
					{{ formatMessage(projectCompatibilityMessages.environments) }}
					<div class="flex gap-1 flex-wrap mt-2">
						<EnvironmentTags :environment="version.environment" />
					</div>
				</div>
			</div>
		</section>
		<section id="changes">
			<h3 class="mt-0 mb-2 text-lg font-semibold">{{ formatMessage(messages.changes) }}</h3>
			<div class="p-4 bg-surface-3 rounded-2xl border-solid border border-surface-4">
				<div
					v-if="version.changelog"
					class="markdown-body"
					v-html="renderHighlightedString(version.changelog)"
				/>
				<div v-else class="text-secondary">{{ formatMessage(messages.noChanges) }}</div>
			</div>
		</section>
		<section v-if="optionalContent.length > 0" id="optional-dependencies">
			<h3 class="mt-0 mb-2 text-lg font-semibold">{{ formatMessage(messages.optionalDeps) }}</h3>
			<div class="grid md:grid-cols-2 gap-4">
				<VersionDependencyItem
					v-for="depContext in optionalContent"
					:key="`optional-dep-${depContext.dependency.version_id ?? depContext.dependency.project_id ?? depContext.dependency.file_name}`"
					:context="depContext"
					:dependency-link="dependencyLinkCreator(depContext)"
					target="_blank"
					class="bg-surface-3 border border-solid border-surface-4 p-4 rounded-2xl"
				>
					<slot name="dependencyActions" :dependency="depContext" />
				</VersionDependencyItem>
			</div>
		</section>
		<section v-if="incompatibleContent.length > 0" id="known-incompatibilities">
			<h3 class="mt-0 mb-2 text-lg font-semibold">
				{{ formatMessage(messages.knownIncompatibilities) }}
			</h3>
			<div class="grid md:grid-cols-2 gap-4">
				<VersionDependencyItem
					v-for="depContext in incompatibleContent"
					:key="`incompatible-dep-${depContext.dependency.version_id ?? depContext.dependency.project_id ?? depContext.dependency.file_name}`"
					:context="depContext"
					:dependency-link="dependencyLinkCreator(depContext)"
					target="_blank"
					class="bg-surface-3 border border-solid border-surface-4 p-4 rounded-2xl"
				/>
			</div>
		</section>
		<section v-if="includedContent?.length > 0" id="content">
			<h3 class="mt-0 mb-2 text-lg font-semibold">
				{{
					formatMessage(
						version.loaders.includes('mrpack') ? messages.content : messages.includedContent,
					)
				}}
			</h3>
			<Table :columns="contentTableColumns" :data="includedContent">
				<template #header-actions>
					<StyledInput
						v-model="contentSearchQuery"
						:icon="SearchIcon"
						:placeholder="formatMessage(messages.searchContent)"
						clearable
						wrapper-class="w-full sm:w-64"
					/>
				</template>
				<template #cell-icon="{ row }">
					<AutoLink :to="row.link" tabindex="-1" class="flex" target="_blank">
						<Avatar v-if="row.icon_url" :src="row.icon_url" alt="" size="2rem" />
						<div v-else class="size-[2rem] flex items-center justify-center">
							<FileIcon class="size-5 text-secondary" />
						</div>
					</AutoLink>
				</template>
				<template #cell-name="{ row }">
					<AutoLink
						:to="row.link"
						class="flex w-fit"
						link-class="hover:underline hover:text-contrast"
						target="_blank"
					>
						{{ row.name }}
					</AutoLink>
				</template>
				<template #cell-version="{ row }">
					{{ (row.version ?? row.hasProject) ? formatMessage(commonMessages.unknownLabel) : '—' }}
				</template>
			</Table>
		</section>
		<section v-if="supplementaryResources?.length > 0" id="supplementary-resources">
			<h3 class="mt-0 mb-2 text-lg font-semibold">
				{{ formatMessage(messages.supplementaryResources) }}
			</h3>
			<Table :columns="supplementaryResourcesTableColumns" :data="supplementaryResources">
				<template #cell-file="{ row }">
					{{ row.file.filename }}
				</template>
				<template #cell-type="{ row }">
					{{
						formatMessage(
							!row.file.file_type || row.file.file_type === 'unknown'
								? commonMessages.unknownLabel
								: fileTypeMessages[row.file.file_type],
						)
					}}
				</template>
				<template #cell-size="{ row }">
					{{ formatBytes(row.file.size) }}
				</template>
				<template #cell-actions="{ row }">
					<div class="flex items-center justify-end gap-2">
						<slot name="supplementaryResourceActions" :file="row.file" />
					</div>
				</template>
			</Table>
		</section>
	</div>
</template>
<style>
.markdown-body ul {
	padding-left: 1.25rem;
}
</style>
