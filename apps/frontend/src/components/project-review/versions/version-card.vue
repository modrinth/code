<template>
	<article
		class="min-w-0 overflow-hidden rounded-xl border border-solid border-surface-4 bg-surface-2"
	>
		<div class="flex flex-wrap items-center gap-3">
			<button
				type="button"
				class="flex min-w-0 flex-1 cursor-pointer items-center gap-3 border-0 bg-transparent p-3 text-left text-primary"
				:aria-expanded="expanded"
				:aria-controls="`review-version-${version.id}`"
				@click="emit('toggle')"
			>
				<ChevronDownIcon
					class="shrink-0 transition-transform"
					:class="{ '-rotate-90': !expanded }"
				/>
				<div class="flex items-center gap-2">
					<div class="flex flex-wrap items-center gap-2">
						<VersionChannelTag :channel="version.version_type" />
						<strong :title="version.name" class="break-all text-contrast">{{
							version.version_number
						}}</strong>
						<TagItem v-if="version.status !== 'listed'">{{
							formatMessage(messages[version.status] ?? messages.unknown)
						}}</TagItem>
						<TagItem v-if="version.files_missing_attribution?.length" class="text-orange">{{
							formatMessage(messages.withheld)
						}}</TagItem>
					</div>
					<span class="break-words text-sm text-secondary"
						>{{ compatibility }} ·
						{{ formatMessage(messages.fileCount, { count: version.files.length }) }}</span
					>
				</div>
			</button>
			<div class="flex flex-wrap gap-2 pr-3">
				<ButtonLink :to="versionHref" target="_blank">{{
					formatMessage(messages.viewVersion)
				}}</ButtonLink>
				<ButtonLink v-if="primaryFile" :href="primaryFile.url" :download="primaryFile.filename"
					><DownloadIcon />{{ formatMessage(messages.download) }}</ButtonLink
				>
			</div>
		</div>
		<div
			v-if="expanded"
			:id="`review-version-${version.id}`"
			class="flex min-w-0 flex-col gap-4 border-0 border-t border-solid border-surface-4 p-3"
		>
			<div class="flex min-w-0 flex-wrap items-start gap-6">
				<section class="min-w-0 flex-[1_1_16rem]">
					<h3 class="mb-2 mt-0 text-sm font-semibold text-secondary">
						{{ formatMessage(messages.details) }}
					</h3>
					<table class="version-metadata w-full text-left">
						<tbody>
							<tr>
								<th scope="row" class="text-sm font-normal text-secondary">
									{{ formatMessage(messages.gameVersions) }}
								</th>
								<td>
									<div class="flex flex-wrap gap-1">
										<TagItem v-for="game in version.game_versions" :key="game">{{ game }}</TagItem>
									</div>
								</td>
							</tr>
							<tr>
								<th scope="row" class="text-sm font-normal text-secondary">
									{{ formatMessage(messages.platforms) }}
								</th>
								<td>
									<div class="flex flex-wrap gap-1">
										<TagTagItem v-for="loader in platforms" :key="loader" :tag="loader" />
									</div>
								</td>
							</tr>
							<tr v-if="version.environment">
								<th scope="row" class="text-sm font-normal text-secondary">
									{{ formatMessage(messages.environments) }}
								</th>
								<td class="space-y-1.5">
									<EnvironmentTags :environment="version.environment" />
								</td>
							</tr>
							<tr>
								<th scope="row" class="text-sm font-normal text-secondary">
									{{ formatMessage(messages.published) }}
								</th>
								<td>
									<time :datetime="version.date_published">{{
										formatDateTime(version.date_published)
									}}</time>
								</td>
							</tr>
							<tr>
								<th scope="row" class="text-sm font-normal text-secondary">
									{{ formatMessage(messages.author) }}
								</th>
								<td>
									<NuxtLink
										:to="`/user/${version.author_id}`"
										target="_blank"
										class="hover:underline"
										>{{ author?.username ?? version.author_id }}</NuxtLink
									>
								</td>
							</tr>
							<tr>
								<th scope="row" class="text-sm font-normal text-secondary">
									{{ formatMessage(messages.downloads) }}
								</th>
								<td>{{ formatNumber(version.downloads) }}</td>
							</tr>
							<tr>
								<th scope="row" class="text-sm font-normal text-secondary">
									{{ formatMessage(messages.versionId) }}
								</th>
								<td><CopyCode :text="version.id" /></td>
							</tr>
						</tbody>
					</table>
					<ButtonLink
						v-if="version.files_missing_attribution?.length"
						:to="permissionsHref"
						target="_blank"
						class="mt-3 w-fit"
						>{{ formatMessage(messages.resolvePermissions) }}</ButtonLink
					>
				</section>
				<div class="flex min-w-0 flex-[3_1_24rem] flex-col gap-4">
					<section>
						<h3 class="mb-2 mt-0 text-sm font-semibold text-secondary">
							{{ formatMessage(messages.files) }}
						</h3>
						<div class="flex flex-col gap-1.5">
							<div
								v-for="file in version.files"
								:key="file.url"
								class="min-w-0 rounded-lg bg-surface-1 px-3 py-2"
							>
								<div class="flex flex-wrap items-center justify-between gap-2">
									<div class="flex min-w-0 flex-1 flex-wrap items-center gap-x-3 gap-y-1">
										<strong class="min-w-0 flex-1 break-all text-contrast">{{
											file.filename
										}}</strong>
										<div class="flex flex-wrap items-center gap-2 text-sm text-secondary">
											<TagItem v-if="file.primary">{{ formatMessage(messages.primary) }}</TagItem
											><span>{{
												formatMessage(fileTypeMessages[file.file_type ?? 'unknown'])
											}}</span
											><span>{{ formatBytes(file.size) }}</span>
										</div>
									</div>
									<ButtonLink
										:href="file.url"
										:download="file.filename"
										:aria-label="formatMessage(messages.download)"
										icon-only
										><DownloadIcon
									/></ButtonLink>
								</div>
								<details class="mt-2">
									<summary class="cursor-pointer text-sm text-secondary">
										{{ formatMessage(messages.hashes) }}
									</summary>
									<div
										v-for="(hash, algorithm) in file.hashes"
										:key="algorithm"
										class="mt-2 flex min-w-0 flex-wrap items-center gap-2 text-xs"
									>
										<span class="uppercase">{{ algorithm }}</span
										><CopyCode :text="hash" class="min-w-0 max-w-full break-all [&_svg]:shrink-0" />
									</div>
								</details>
							</div>
						</div>
					</section>
					<section>
						<h3 class="mb-2 mt-0 text-sm font-semibold text-secondary">
							{{ formatMessage(messages.dependencies) }}
						</h3>
						<p v-if="!version.dependencies.length" class="m-0 text-secondary">
							{{ formatMessage(messages.emptyDependencies) }}
						</p>
						<p v-else-if="dependenciesQuery.isPending.value" role="status" class="m-0">
							{{ formatMessage(messages.loading) }}
						</p>
						<div v-else-if="dependenciesQuery.isError.value" role="alert">
							<p class="m-0">{{ formatMessage(messages.loadError) }}</p>
							<Button @click="dependenciesQuery.refetch()">{{
								formatMessage(messages.retry)
							}}</Button>
						</div>
						<div v-else class="flex flex-col gap-1.5">
							<div
								v-for="(context, index) in dependencies"
								:key="index"
								class="min-w-0 rounded-lg bg-surface-1 px-3 py-2"
							>
								<VersionDependencyItem
									:context="context"
									:dependency-link="dependencyHref(context)"
									link-tabbable
									class="min-w-0 break-words"
								>
									<TagItem>{{
										formatMessage(messages[context.dependency.dependency_type])
									}}</TagItem>
								</VersionDependencyItem>
								<CopyCode
									v-if="!context.project"
									:text="
										context.dependency.project_id ??
										context.dependency.version_id ??
										context.dependency.file_name ??
										''
									"
									class="mt-2"
								/>
							</div>
						</div>
					</section>
				</div>
			</div>
			<details
				:open="changelogOpen"
				@toggle="changelogOpen = ($event.target as HTMLDetailsElement).open"
			>
				<summary class="cursor-pointer font-semibold text-contrast">
					{{ formatMessage(messages.changelog) }}
				</summary>
				<div v-if="changelogOpen" class="mt-3 rounded-lg p-3">
					<p v-if="detailQuery.isPending.value" role="status" class="m-0">
						{{ formatMessage(messages.loading) }}
					</p>
					<div v-else-if="detailQuery.isError.value" role="alert">
						<p class="m-0">{{ formatMessage(messages.loadError) }}</p>
						<Button @click="detailQuery.refetch()">{{ formatMessage(messages.retry) }}</Button>
					</div>
					<ProjectPageDescription
						v-else-if="detailQuery.data.value?.changelog"
						:description="detailQuery.data.value.changelog"
					/>
					<p v-else class="m-0 text-secondary">
						{{ formatMessage(messages.emptyChangelog) }}
					</p>
				</div>
			</details>
			<ReviewPanel
				mode="inline"
				:target="{ kind: 'version', key: version.id }"
				:label="version.version_number"
			/>
		</div>
	</article>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ChevronDownIcon, DownloadIcon } from '@modrinth/assets'
import {
	Button,
	ButtonLink,
	CopyCode,
	type DependencyContext,
	fileTypeMessages,
	injectModrinthClient,
	injectTags,
	ProjectPageDescription,
	TagItem,
	TagTagItem,
	useFormatBytes,
	useFormatDateTime,
	useFormatNumber,
	useVIntl,
	VersionDependencyItem,
} from '@modrinth/ui'
import EnvironmentTags from '@modrinth/ui/src/components/project/EnvironmentTags.vue'
import VersionChannelTag from '@modrinth/ui/src/components/version/VersionChannelTag.vue'
import { formatVersionsForDisplay } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { projectQueryOptions } from '~/composables/queries/project'
import { versionQueryOptions } from '~/composables/queries/version'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../messages'
import ReviewPanel from '../review-panel/index.vue'

const props = defineProps<{
	version: Labrinth.Versions.v3.Version
	expanded: boolean
}>()
const emit = defineEmits<{ toggle: [] }>()
const { formatMessage } = useVIntl()
const formatDateTime = useFormatDateTime({
	dateStyle: 'long',
	timeStyle: 'short',
})
const formatNumber = useFormatNumber()
const formatBytes = useFormatBytes()
const client = injectModrinthClient()
const tags = injectTags(null)
const { project, members } = injectProjectReviewPageContext()
const changelogOpen = ref(false)
const platforms = computed(() =>
	props.version.loaders.includes('mrpack')
		? (props.version.mrpack_loaders ?? [])
		: props.version.loaders,
)
const compatibility = computed(() => {
	const games = tags?.gameVersions.value?.length
		? formatVersionsForDisplay(props.version.game_versions, tags.gameVersions.value)
		: props.version.game_versions
	return [...games, ...platforms.value].join(', ')
})
const primaryFile = computed(
	() => props.version.files.find((file) => file.primary) ?? props.version.files[0],
)
const projectHref = computed(
	() =>
		`/${project.value?.project_types[0] ?? 'project'}/${project.value?.slug ?? props.version.project_id}`,
)
const versionHref = computed(() => `${projectHref.value}/version/${props.version.id}`)
const permissionsHref = computed(() => `${projectHref.value}/settings/permissions`)
const detailQuery = useQuery(
	computed(() => ({
		...versionQueryOptions.v3(props.version.id, client),
		enabled: props.expanded && changelogOpen.value,
	})),
)
const dependenciesQuery = useQuery(
	computed(() => ({
		...projectQueryOptions.dependencies(props.version.project_id, client),
		enabled: props.expanded && props.version.dependencies.length > 0,
	})),
)
const memberAuthor = computed(
	() => members.value.find((member) => member.user.id === props.version.author_id)?.user,
)
const authorQuery = useQuery({
	queryKey: computed(() => ['user', props.version.author_id]),
	queryFn: () => client.labrinth.users_v3.get(props.version.author_id),
	enabled: computed(() => props.expanded && !memberAuthor.value),
})
const author = computed(() => memberAuthor.value ?? authorQuery.data.value)
const dependencies = computed<DependencyContext[]>(() =>
	props.version.dependencies.map((dependency) => {
		const version = dependenciesQuery.data.value?.versions.find(
			(version) => version.id === dependency.version_id,
		)
		return {
			dependency,
			version,
			project: dependenciesQuery.data.value?.projects.find(
				(project) => project.id === (dependency.project_id ?? version?.project_id),
			),
		}
	}),
)
function dependencyHref(context: DependencyContext) {
	if (context.project) {
		const path = `/${context.project.project_type}/${context.project.slug ?? context.project.id}`
		return context.version ? `${path}/version/${context.version.id}` : path
	}
	const resolution = context.dependency.attribution?.resolution
	return resolution && 'link_to_work' in resolution ? resolution.link_to_work : undefined
}
</script>

<style scoped>
.version-metadata {
	border-collapse: collapse;
}

.version-metadata th {
	width: 7rem;
	padding-right: 0.75rem;
}

.version-metadata th,
.version-metadata td {
	padding-top: 0.25rem;
	padding-bottom: 0.25rem;
	vertical-align: top;
	overflow-wrap: anywhere;
}
</style>
