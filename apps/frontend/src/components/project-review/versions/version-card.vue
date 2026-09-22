<template>
	<article
		class="version-card flex min-w-0 flex-col gap-3 overflow-hidden rounded-2xl border border-solid bg-surface-2 p-2.5 pb-0.5"
		:class="withheld ? 'border-orange' : 'border-transparent'"
	>
		<div class="flex min-w-0 flex-1 flex-wrap items-start gap-2 gap-x-4">
			<div class="mt-0.5 flex min-w-0 flex-1 flex-wrap items-center gap-2">
				<VersionChannelIndicator
					:channel="version.version_type"
					:title="formatMessage(commonMessages[version.version_type])"
					size="sm"
				/>
				<TagTagItem v-for="loader in platforms" :key="loader" :tag="loader" />
				<TagItem v-for="game in gameVersions" :key="game">{{ game }}</TagItem>
				<EnvironmentTags v-if="version.environment" :environment="version.environment" />
				<TagItem
					v-if="withheld"
					class="!border-orange-highlight !bg-orange-highlight font-semibold capitalize !text-orange"
					>{{ formatMessage(messages.withheld) }}</TagItem
				>
				<TagItem v-if="version.status !== 'listed'">{{
					formatMessage(messages[version.status] ?? messages.unknown)
				}}</TagItem>
			</div>
			<div class="ml-auto flex flex-wrap items-center justify-end gap-2 text-sm text-secondary">
				<span>{{
					formatMessage(messages.dependencyCount, { count: version.dependencies.length })
				}}</span>
				<BulletDivider aria-hidden="true" />
				<span
					class="inline-flex items-center gap-1"
					:title="`${formatMessage(messages.downloads)}: ${formatNumber(version.downloads)}`"
				>
					<DownloadIcon class="size-4" aria-hidden="true" />{{
						formatCompactNumber(version.downloads)
					}}
				</span>
				<BulletDivider aria-hidden="true" />
				<time :datetime="version.date_published" :title="formatDateTime(version.date_published)">{{
					relativeTime(version.date_published)
				}}</time>
				<ButtonLink
					v-tooltip="formatMessage(messages.viewVersion)"
					:to="versionHref"
					:aria-label="formatMessage(messages.viewVersion)"
					target="_blank"
					type="quiet"
					circular
					size="sm"
					icon-only
				>
					<ExternalIcon />
				</ButtonLink>
			</div>
		</div>
		<div class="flex flex-col gap-1.5">
			<div
				v-for="file in version.files"
				:key="file.url"
				class="min-w-0 rounded-lg bg-surface-1 px-3 py-2"
			>
				<div class="flex w-full min-w-0 flex-wrap items-center justify-between gap-2.5">
					<div class="min-w-0 break-all font-medium">
						{{ file.filename }}
						<span class="ml-1 text-sm font-normal text-secondary">
							({{ formatBytes(file.size) }})
						</span>
					</div>
					<div class="flex flex-wrap items-center gap-2 text-sm text-secondary">
						<span v-if="file.file_type && file.file_type !== 'unknown'">{{
							formatMessage(fileTypeMessages[file.file_type])
						}}</span>
						<TeleportOverflowMenu
							type="quiet"
							size="sm"
							:label="formatMessage(commonMessages.moreOptionsButton)"
							:options="fileActions(file)"
						>
							<MoreVerticalIcon />
						</TeleportOverflowMenu>
					</div>
				</div>
			</div>
			<Accordion
				button-class="w-full cursor-pointer border-0 bg-transparent py-2 text-left text-sm font-medium hover:brightness-125 [&>div>svg]:ml-0 [&>div>svg]:size-4"
				:open-by-default="expanded"
				@on-open="!expanded && emit('toggle')"
				@on-close="expanded && emit('toggle')"
			>
				<template #title>
					<span class="text-primary">
						{{ formatMessage(messages.details) }}
					</span>
				</template>
				<div :id="`review-version-${version.id}`" class="mt-1 flex min-w-0 flex-col gap-5">
					<section>
						<dl
							class="m-0 grid min-w-0 grid-cols-[minmax(0,max-content)_minmax(0,1fr)] items-baseline gap-x-6 gap-y-3"
						>
							<dt class="text-secondary">{{ formatMessage(messages.versionNumber) }}</dt>
							<dd class="m-0 min-w-0 break-all">{{ version.version_number }}</dd>
							<dt class="text-secondary">{{ formatMessage(messages.versionSubtitle) }}</dt>
							<dd class="m-0 min-w-0 break-words">{{ version.name }}</dd>
							<dt class="text-secondary">{{ formatMessage(messages.publishedBy) }}</dt>
							<dd class="m-0 min-w-0 break-words">
								<NuxtLink
									:to="`/user/${version.author_id}`"
									target="_blank"
									class="hover:underline"
									>{{ author?.username ?? version.author_id }}</NuxtLink
								>
							</dd>
							<dt class="text-secondary">{{ formatMessage(messages.versionId) }}</dt>
							<dd class="m-0 min-w-0"><CopyCode :text="version.id" /></dd>
						</dl>
					</section>
					<section>
						<h3 class="mb-3 mt-0 text-sm font-semibold text-secondary">
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
								<div class="flex min-w-0 flex-wrap items-center gap-3">
									<AutoLink
										:to="dependencyHref(context)"
										class="flex min-w-0 flex-1 items-center gap-3 text-contrast hover:underline"
									>
										<Avatar
											:src="
												context.project?.icon_url ??
												context.dependency.attribution?.flame_project?.icon_url
											"
											alt=""
											size="1.5rem"
											no-shadow
										/>
										<span class="break-words">{{
											context.project?.title ??
											context.dependency.file_name ??
											context.dependency.project_id ??
											context.dependency.version_id
										}}</span>
									</AutoLink>
									<span v-if="context.version" class="break-all font-mono text-sm text-secondary">{{
										context.version.version_number
									}}</span>
									<TagItem class="text-xs">{{
										formatMessage(messages[context.dependency.dependency_type])
									}}</TagItem>
								</div>
							</div>
						</div>
						<Accordion
							:open-by-default="changelogOpen"
							button-class="w-full cursor-pointer border-0 bg-transparent mt-2 py-2 text-left text-sm font-medium hover:brightness-125 [&>div>svg]:ml-0 [&>div>svg]:size-4"
							@on-open="changelogOpen = true"
							@on-close="changelogOpen = false"
						>
							<template #title>
								<span class="text-primary">
									{{ formatMessage(messages.changelog) }}
								</span>
							</template>
							<div class="mb-2.5">
								<p v-if="detailQuery.isPending.value" role="status" class="m-0">
									{{ formatMessage(messages.loading) }}
								</p>
								<div v-else-if="detailQuery.isError.value" role="alert">
									<p class="m-0">{{ formatMessage(messages.loadError) }}</p>
									<Button @click="detailQuery.refetch()">{{
										formatMessage(messages.retry)
									}}</Button>
								</div>
								<ProjectPageDescription
									v-else-if="detailQuery.data.value?.changelog"
									:description="detailQuery.data.value.changelog"
								/>
								<p v-else class="m-0 text-secondary">
									{{ formatMessage(messages.emptyChangelog) }}
								</p>
							</div>
						</Accordion>
					</section>
				</div>
			</Accordion>
		</div>
	</article>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ClipboardCopyIcon, DownloadIcon, ExternalIcon, MoreVerticalIcon } from '@modrinth/assets'
import {
	Accordion,
	AutoLink,
	Avatar,
	BulletDivider,
	Button,
	ButtonLink,
	type ButtonMenuOption,
	commonMessages,
	CopyCode,
	type DependencyContext,
	fileTypeMessages,
	injectModrinthClient,
	injectTags,
	ProjectPageDescription,
	TagItem,
	TagTagItem,
	TeleportOverflowMenu,
	useCompactNumber,
	useFormatBytes,
	useFormatDateTime,
	useFormatNumber,
	useRelativeTime,
	useVIntl,
	VersionChannelIndicator,
} from '@modrinth/ui'
import EnvironmentTags from '@modrinth/ui/src/components/project/EnvironmentTags.vue'
import { formatVersionsForDisplay } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { projectQueryOptions } from '~/composables/queries/project'
import { versionQueryOptions } from '~/composables/queries/version'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../messages'

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
const { formatCompactNumber } = useCompactNumber()
const relativeTime = useRelativeTime({ style: 'narrow' })
const formatBytes = useFormatBytes()
const client = injectModrinthClient()
const tags = injectTags(null)
const { project, members } = injectProjectReviewPageContext()
const changelogOpen = ref(false)
const withheld = computed(() => !!props.version.files_missing_attribution?.length)
const platforms = computed(() =>
	props.version.loaders.includes('mrpack')
		? (props.version.mrpack_loaders ?? [])
		: props.version.loaders,
)
const gameVersions = computed(() =>
	tags?.gameVersions.value?.length
		? formatVersionsForDisplay(props.version.game_versions, tags.gameVersions.value)
		: props.version.game_versions,
)
const projectHref = computed(
	() =>
		`/${project.value?.project_types[0] ?? 'project'}/${project.value?.slug ?? props.version.project_id}`,
)
const versionHref = computed(() => `${projectHref.value}/version/${props.version.id}`)
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
function fileActions(file: Labrinth.Versions.v3.Version['files'][number]): ButtonMenuOption[] {
	const options: ButtonMenuOption[] = []
	for (const algorithm of ['sha1', 'sha512'] as const) {
		const hash = file.hashes[algorithm]
		if (!hash) continue
		options.push({
			id: `copy-${algorithm}`,
			label: formatMessage(messages.copyHash, {
				algorithm: algorithm === 'sha1' ? 'SHA-1' : 'SHA-512',
			}),
			icon: ClipboardCopyIcon,
			action: () => navigator.clipboard.writeText(hash),
		})
	}
	options.push({
		id: 'download',
		type: 'link',
		label: formatMessage(messages.download),
		icon: DownloadIcon,
		href: file.url,
		download: file.filename,
	})
	return options
}

function dependencyHref(context: DependencyContext) {
	if (context.project) {
		const path = `/${context.project.project_type}/${context.project.slug ?? context.project.id}`
		return context.version ? `${path}/version/${context.version.id}` : path
	}
	const resolution = context.dependency.attribution?.resolution
	return resolution && 'link_to_work' in resolution ? resolution.link_to_work : undefined
}
</script>
