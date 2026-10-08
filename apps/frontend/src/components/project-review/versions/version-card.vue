<template>
	<article
		ref="card"
		class="version-card flex min-w-0 flex-col gap-3 overflow-clip rounded-2xl border border-solid bg-surface-2 p-2.5 pb-0.5 @container/review-version"
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
			<div class="ml-auto flex flex-wrap items-center justify-end gap-1.5 text-sm text-secondary">
				<span>{{
					formatMessage(messages.dependencyCount, {
						count: version.dependencies.length,
					})
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
				<div class="flex gap-0">
					<EditVersionMenu v-if="editable" color="default" size="sm" @edit="emit('edit', $event)" />
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
					<div class="flex flex-wrap items-center gap-0.5 text-sm text-secondary">
						<span v-if="file.file_type && file.file_type !== 'unknown'">{{
							formatMessage(fileTypeMessages[file.file_type])
						}}</span>
						<ButtonLink
							v-tooltip="formatMessage(messages.openSlicer)"
							:href="`https://slicer.run/?url=${encodeURIComponent(file.url)}`"
							:aria-label="formatMessage(messages.openSlicer)"
							target="_blank"
							type="quiet"
							size="sm"
							circular
							icon-only
						>
							<CoffeeIcon aria-hidden="true" />
						</ButtonLink>
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
				ref="detailsAccordion"
				button-class="w-fit cursor-pointer border-0 bg-transparent py-2 text-left text-sm font-medium hover:[&>div]:brightness-125 [&>div>svg]:ml-0 [&>div>svg]:size-4"
				:open-by-default="expanded"
				overflow-visible
				@on-open="!expanded && emit('toggle')"
				@on-close="expanded && emit('toggle')"
			>
				<template #title>
					<span class="text-primary">
						{{ formatMessage(messages.details) }}
					</span>
				</template>
				<div
					:id="`review-version-${version.id}`"
					class="mb-3 grid min-w-0 grid-cols-1 items-start gap-5 @[40rem]/review-version:grid-cols-[minmax(0,2fr)_minmax(0,3fr)] @[48rem]/review-version:gap-8"
				>
					<section class="min-w-0">
						<dl
							class="m-0 mt-1 grid min-w-0 grid-cols-[minmax(0,max-content)_minmax(0,1fr)] items-baseline gap-x-4 gap-y-3"
						>
							<dt class="font-medium text-secondary">
								{{ formatMessage(messages.versionNumber) }}
							</dt>
							<dd class="m-0 min-w-0 break-all">
								{{ version.version_number }}
							</dd>
							<dt class="font-medium text-secondary">
								{{ formatMessage(messages.versionSubtitle) }}
							</dt>
							<dd class="m-0 min-w-0 break-words">{{ version.name }}</dd>
							<dt class="font-medium text-secondary">
								{{ formatMessage(messages.versionId) }}
							</dt>
							<dd class="m-0 min-w-0"><CopyCode :text="version.id" /></dd>
							<dt class="self-center font-medium text-secondary">
								{{ formatMessage(messages.publishedBy) }}
							</dt>
							<dd class="m-0 min-w-0 self-center break-words">
								<NuxtLink
									:to="`/user/${version.author_id}`"
									target="_blank"
									class="flex min-w-0 items-center gap-1 hover:underline"
								>
									<Avatar
										:src="author?.avatar_url"
										alt=""
										size="1.25rem"
										class="-my-1 shrink-0"
										circle
										no-shadow
									/>
									<span v-tooltip="author?.username ?? version.author_id" class="min-w-0 truncate">
										{{ author?.username ?? version.author_id }}
									</span>
								</NuxtLink>
							</dd>
						</dl>
					</section>
					<section class="min-w-0">
						<div ref="dependenciesSection" class="min-w-0">
							<h3 class="sticky top-0 z-10 m-0 bg-surface-2 py-2 text-sm font-medium text-primary">
								<button
									v-if="version.dependencies.length > 6"
									type="button"
									class="flex w-full cursor-pointer items-center gap-1 border-0 bg-transparent p-0 text-left text-sm font-medium text-primary hover:brightness-125"
									:aria-expanded="dependenciesOpen"
									:aria-controls="`review-version-dependencies-${version.id}`"
									@click="dependenciesOpen = !dependenciesOpen"
								>
									{{ formatMessage(messages.dependencies) }} ({{
										formatNumber(version.dependencies.length)
									}})
									<DropdownIcon
										class="size-4 shrink-0 transition-transform duration-150 motion-reduce:transition-none"
										:class="{ 'rotate-180': dependenciesOpen }"
										aria-hidden="true"
									/>
								</button>
								<span v-else>
									{{ formatMessage(messages.dependencies) }} ({{
										formatNumber(version.dependencies.length)
									}})
								</span>
							</h3>
							<div
								:id="`review-version-dependencies-${version.id}`"
								class="grid transition-[grid-template-rows] duration-150 ease-out motion-reduce:transition-none"
								:class="
									version.dependencies.length <= 6 || dependenciesOpen
										? 'grid-rows-[1fr]'
										: 'grid-rows-[0fr]'
								"
								:inert="version.dependencies.length > 6 && !dependenciesOpen"
							>
								<div class="min-h-0 overflow-hidden">
									<div class="pb-2.5">
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
													<span
														v-if="context.version"
														class="break-all font-mono text-sm text-secondary"
														>{{ context.version.version_number }}</span
													>
													<TagItem class="text-xs">{{
														formatMessage(messages[context.dependency.dependency_type])
													}}</TagItem>
												</div>
											</div>
										</div>
									</div>
								</div>
							</div>
						</div>
						<Accordion
							v-if="detailQuery.data.value?.changelog"
							ref="changelogAccordion"
							open-by-default
							button-class="sticky top-0 z-10 w-full cursor-pointer border-0 bg-surface-2 py-2 text-left text-sm font-medium hover:[&>div]:brightness-125 [&>div>svg]:ml-0 [&>div>svg]:size-4"
						>
							<template #title>
								<span class="text-primary">{{ formatMessage(messages.changelog) }}</span>
							</template>
							<div class="mb-4">
								<ProjectPageDescription :description="detailQuery.data.value.changelog" />
							</div>
						</Accordion>
						<div v-else class="mb-4">
							<h3 class="m-0 py-2 text-sm font-medium text-primary">
								{{ formatMessage(messages.changelog) }}
							</h3>
							<p v-if="detailQuery.isPending.value" role="status" class="m-0">
								{{ formatMessage(messages.loading) }}
							</p>
							<div v-else-if="detailQuery.isError.value" role="alert">
								<p class="m-0">{{ formatMessage(messages.loadError) }}</p>
								<Button @click="detailQuery.refetch()">{{ formatMessage(messages.retry) }}</Button>
							</div>
							<p v-else class="m-0 text-secondary">
								{{ formatMessage(messages.emptyChangelog) }}
							</p>
						</div>
					</section>
				</div>
			</Accordion>
		</div>
	</article>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ClipboardCopyIcon,
	CoffeeIcon,
	DownloadIcon,
	DropdownIcon,
	ExternalIcon,
	MoreVerticalIcon,
} from '@modrinth/assets'
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

import EditVersionMenu from '~/components/ui/create-project-version/EditVersionMenu.vue'
import { projectQueryOptions } from '~/composables/queries/project'
import { versionQueryOptions } from '~/composables/queries/version'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import type { EditVersionStage } from '~/providers/version/manage-version-modal'

import { projectReviewMessages as messages } from '../messages'
import { useReviewInteraction } from '../shortcuts'

const props = defineProps<{
	version: Labrinth.Versions.v3.Version
	expanded: boolean
	editable: boolean
}>()
const emit = defineEmits<{ toggle: []; edit: [stage: EditVersionStage] }>()
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
const dependenciesOpen = ref(false)
const card = ref<HTMLElement>()
const dependenciesSection = ref<HTMLElement>()
const detailsAccordion = ref<InstanceType<typeof Accordion>>()
const changelogAccordion = ref<InstanceType<typeof Accordion>>()
function toggleAccordion(accordion: InstanceType<typeof Accordion> | undefined) {
	if (!accordion) return
	if (accordion.isOpen) accordion.close()
	else accordion.open()
}
useReviewInteraction({
	element: () => card.value,
	editable: () => props.editable,
	edit: () => emit('edit', 'metadata'),
	collapse: () => toggleAccordion(detailsAccordion.value),
})
useReviewInteraction({
	element: () =>
		props.expanded && props.version.dependencies.length > 6 ? dependenciesSection.value : null,
	collapse: () => {
		dependenciesOpen.value = !dependenciesOpen.value
	},
})
useReviewInteraction({
	element: () => (props.expanded ? changelogAccordion.value?.$el : null),
	editable: () => props.editable,
	edit: () => emit('edit', 'add-details'),
	collapse: () => toggleAccordion(changelogAccordion.value),
})
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
		enabled: props.expanded,
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
const dependencyTypeOrder = { required: 0, optional: 1, incompatible: 2, embedded: 3 }
function dependencyName(context: DependencyContext) {
	return (
		context.project?.title ??
		context.dependency.file_name ??
		context.dependency.project_id ??
		context.dependency.version_id ??
		''
	)
}
const dependencies = computed<DependencyContext[]>(() =>
	props.version.dependencies
		.map((dependency) => {
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
		})
		.sort((a, b) => {
			const typeDifference =
				dependencyTypeOrder[a.dependency.dependency_type] -
				dependencyTypeOrder[b.dependency.dependency_type]
			return typeDifference || dependencyName(a).localeCompare(dependencyName(b))
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
