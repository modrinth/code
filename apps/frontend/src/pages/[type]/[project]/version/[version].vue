<template>
	<div>
		<CreateProjectVersionModal v-if="currentMember" ref="editModal" @save="handleVersionSaved" />
		<ConfirmModal
			v-if="currentMember"
			ref="confirmModal"
			:title="formatMessage(messages.confirmTitle)"
			:description="formatMessage(messages.confirmDescription)"
			:has-to-type="false"
			:proceed-label="formatMessage(messages.proceedDeletion)"
			@proceed="deleteVersion()"
		/>
		<NewModal
			v-if="auth.user && currentMember"
			ref="packageModal"
			:header="formatMessage(messages.packageDataPackHeader)"
		>
			<div class="flex max-w-[35rem] flex-col">
				<p class="m-0 mb-6">
					{{ formatMessage(messages.packageDataPackDescription) }}
				</p>
				<div for="package-mod-loaders" class="mb-2 flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">{{
						formatMessage(messages.modLoadersLabel)
					}}</span>
					<MultiSelect
						id="package-mod-loaders"
						v-model="packageLoaders"
						:options="packageLoaderOptions"
						:searchable="false"
						:placeholder="formatMessage(messages.modLoadersPlaceholder)"
					>
						<template #input-content="{ selectedOptions, isOpen, openDirection }">
							<div class="flex min-h-8 min-w-0 flex-1 flex-wrap items-center gap-1.5 pr-1">
								<template v-if="selectedOptions.length > 0">
									<span
										v-for="{ value: loader, label } in selectedOptions"
										:key="`package-loader-tag-${loader}`"
										class="inline-flex cursor-pointer items-center gap-1 rounded-full border border-solid bg-surface-4 px-2 py-1 text-sm font-medium transition-all hover:brightness-[110%]"
										:style="`color: var(--color-platform-${loader})`"
										@click.stop="packageLoaders = packageLoaders.filter((x) => x !== loader)"
									>
										<component
											:is="getLoaderIcon(loader)"
											v-if="getLoaderIcon(loader)"
											class="size-3.5 shrink-0"
										/>
										{{ label }}
										<XIcon aria-hidden="true" class="size-3.5 shrink-0" />
									</span>
								</template>
								<span v-else class="text-base font-medium text-primary opacity-50">
									{{ formatMessage(messages.modLoadersPlaceholder) }}
								</span>
							</div>
							<ChevronLeftIcon
								class="ml-2 size-5 shrink-0 text-secondary transition-transform duration-150"
								:class="
									isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'
								"
							/>
						</template>
					</MultiSelect>
					<span>{{ formatMessage(messages.modLoadersDescription) }}</span>
				</div>

				<div class="ml-auto mt-4 flex items-center gap-2">
					<ButtonStyled type="outlined">
						<button @click="packageModal?.hide()">
							<XIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button :disabled="packageLoaders.length === 0" @click="createDataPackVersionHandler">
							{{ formatMessage(messages.packageDataPack) }}
							<RightArrowIcon aria-hidden="true" />
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>
		<div class="flex flex-col">
			<nuxt-link
				class="mb-4 flex w-fit items-center gap-2 rounded-lg px-2 py-0.5 pl-0 text-link"
				:to="`/${project.project_type}/${project.slug ? project.slug : project.id}/versions`"
			>
				<ChevronLeftIcon class="shrink-0" /> {{ formatMessage(messages.allVersions) }}
			</nuxt-link>
			<template v-if="version">
				<Admonition
					v-if="version.files_missing_attribution?.length"
					type="circle-warning"
					:header="formatMessage(messages.unknownEmbeddedContent)"
					:body="formatMessage(messages.unknownEmbeddedContentDescription)"
					class="mb-4"
				>
					<template #actions>
						<div class="flex">
							<ButtonStyled color="orange">
								<nuxt-link
									:to="`/${project.project_type}/${
										project.slug ? project.slug : project.id
									}/settings/permissions`"
								>
									{{ formatMessage(commonProjectSettingsMessages.withheldVersionsWarningResolve) }}
									<RightArrowIcon />
								</nuxt-link>
							</ButtonStyled>
						</div>
					</template>
				</Admonition>
				<VersionPage
					:version="version"
					:enrichment="enrichment"
					:enrichment-loading="dependenciesLoading"
					:members="members"
					:user-link-creator="(user) => (moderator ? `/user/${user.id}` : undefined)"
					:dependency-link-creator="createDependencyLink"
					class="mb-4"
				>
					<template #headerActions="{ primaryFile, promotedFiles }">
						<ButtonStyled color="brand">
							<a
								v-tooltip="
									primaryFile?.url
										? primaryFile.filename + ' (' + formatBytes(primaryFile.size) + ')'
										: formatMessage(messages.noPrimaryFile)
								"
								:href="decoratedPrimaryFileUrl"
								:download="primaryFile?.filename"
								:disabled="primaryFile?.url === undefined"
								@click="emit('onDownload')"
							>
								<DownloadIcon aria-hidden="true" />
								{{ formatMessage(commonMessages.downloadButton) }}
							</a>
						</ButtonStyled>
						<ButtonStyled
							v-for="file in promotedFiles.filter(
								(x) =>
									!!x &&
									(x.file_type === 'required-resource-pack' ||
										x.file_type === 'optional-resource-pack'),
							)"
							:key="`promoted-file-${file.hashes.sha1}`"
						>
							<a
								v-tooltip="file.filename + ' (' + formatBytes(file.size) + ')'"
								:href="
									createProjectDownloadUrl(file.url, {
										reason: 'dependency',
									})
								"
								:download="primaryFile?.filename"
								:disabled="primaryFile?.url === undefined"
								@click="emit('onDownload')"
							>
								<DownloadIcon aria-hidden="true" />
								<template v-if="file.file_type === 'required-resource-pack'">
									{{ formatMessage(messages.requiredResourcePack) }}
								</template>
								<template v-else-if="file.file_type === 'optional-resource-pack'">
									{{ formatMessage(messages.optionalResourcePack) }}
								</template>
							</a>
						</ButtonStyled>
						<template v-if="currentMember">
							<ButtonStyled
								v-if="
									version.loaders.some((x: string) => tags.loaderData.dataPackLoaders.includes(x))
								"
							>
								<button @click="packageModal?.show()">
									<PackageClosedIcon aria-hidden="true" />
									{{ formatMessage(messages.packageAsMod) }}
								</button>
							</ButtonStyled>
							<ButtonStyled>
								<OverflowMenu
									:dropdown-id="`${baseId}-edit-overflow`"
									class="btn-dropdown-animation"
									:options="[
										{
											id: 'edit-metadata',
											action: () => handleOpenEditVersionModal(version!.id, project.id, 'metadata'),
										},
										{
											id: 'edit-details',
											action: () =>
												handleOpenEditVersionModal(version!.id, project.id, 'add-details'),
										},
										{
											id: 'edit-files',
											action: () =>
												handleOpenEditVersionModal(version!.id, project.id, 'add-files'),
										},
										{
											id: 'delete',
											color: 'red',
											action: () => confirmModal?.show(),
										},
									]"
								>
									<SettingsIcon aria-hidden="true" /> {{ formatMessage(messages.edit) }}
									<DropdownIcon class="h-5 w-5 text-secondary" />
									<template #edit-metadata>
										<BoxIcon aria-hidden="true" />
										{{ formatMessage(messages.editMetadata) }}
									</template>
									<template #edit-details>
										<InfoIcon aria-hidden="true" />
										{{ formatMessage(messages.editDetails) }}
									</template>
									<template #edit-files>
										<FileIcon aria-hidden="true" />
										{{ formatMessage(messages.editFiles) }}
									</template>
									<template #delete>
										<TrashIcon aria-hidden="true" />
										{{ formatMessage(commonMessages.deleteLabel) }}
									</template>
								</OverflowMenu>
							</ButtonStyled>
						</template>
						<ButtonStyled type="outlined" circular>
							<OverflowMenu
								v-tooltip="formatMessage(commonMessages.moreOptionsButton)"
								:options="[
									{
										id: 'report',
										color: 'red',
										action: () =>
											auth.user ? reportVersion(version!.id) : navigateTo(signInRouteObj),
									},
									{ divider: true, shown: flags.developerMode },
									{
										id: 'copy-id',
										action: () => copyToClipboard(version!.id),
										shown: flags.developerMode,
									},
									{
										id: 'copy-permalink',
										action: () =>
											copyToClipboard(
												`https://modrinth.com/project/${project.id}/version/${version!.id}`,
											),
										shown: flags.developerMode,
									},
								]"
							>
								<MoreVerticalIcon />
								<template #report>
									<ReportIcon aria-hidden="true" />
									{{ formatMessage(commonMessages.reportButton) }}
								</template>
								<template #copy-link>
									<ReportIcon aria-hidden="true" />
									{{ formatMessage(commonMessages.reportButton) }}
								</template>
								<template #copy-id>
									<ClipboardCopyIcon aria-hidden="true" />
									{{ formatMessage(commonMessages.copyIdButton) }}
								</template>
								<template #copy-permalink>
									<ClipboardCopyIcon aria-hidden="true" />
									{{ formatMessage(commonMessages.copyPermalinkButton) }}
								</template>
							</OverflowMenu>
						</ButtonStyled>
					</template>
					<template #supplementaryResourceActions="{ file }">
						<ButtonStyled>
							<a
								:href="decorateDownloadUrl(file.url)"
								:title="`Download ${file.filename}`"
								:download="file.filename"
								tabindex="0"
							>
								<DownloadIcon aria-hidden="true" />
								{{ formatMessage(commonMessages.downloadButton) }}
							</a>
						</ButtonStyled>
						<ButtonStyled type="outlined" circular>
							<OverflowMenu
								:tooltip="formatMessage(commonMessages.moreOptionsButton)"
								:options="[
									{
										id: 'copy-sha1',
										action: () => copyFileHash(file, 'sha1'),
									},
									{
										id: 'copy-sha512',
										action: () => copyFileHash(file, 'sha512'),
									},
								]"
								:dropdown-id="`${baseId}-supplementary-resource-actions`"
							>
								<MoreVerticalIcon aria-hidden="true" />
								<template #copy-sha1>
									<CopyIcon aria-hidden="true" />
									{{ formatMessage(messages.copySha1) }}
								</template>
								<template #copy-sha512>
									<CopyIcon aria-hidden="true" />
									{{ formatMessage(messages.copySha512) }}
								</template>
							</OverflowMenu>
						</ButtonStyled>
					</template>
					<template #dependencyActions="{ dependency }">
						<ButtonStyled circular>
							<nuxt-link
								v-if="createDependencyLink(dependency)"
								v-tooltip="
									formatMessage(dependency.version ? messages.viewVersion : messages.viewProject)
								"
								:to="createDependencyLink(dependency)"
								target="_blank"
							>
								<ExternalIcon />
							</nuxt-link>
						</ButtonStyled>
						<ButtonStyled circular color="brand" color-fill="text">
							<a
								v-if="
									dependency.version && dependency.dependency.dependency_type !== 'incompatible'
								"
								v-tooltip="
									dependencyVersionPrimaryFiles[dependency.version.id]
										? dependencyVersionPrimaryFiles[dependency.version.id].filename +
											' (' +
											formatBytes(dependencyVersionPrimaryFiles[dependency.version.id].size) +
											')'
										: formatMessage(messages.noPrimaryFile)
								"
								:href="
									createProjectDownloadUrl(
										dependencyVersionPrimaryFiles[dependency.version.id].url,
										{
											reason: 'dependency',
										},
									)
								"
								:download="dependencyVersionPrimaryFiles[dependency.version.id].filename"
								:disabled="dependencyVersionPrimaryFiles[dependency.version.id].url === undefined"
							>
								<DownloadIcon />
							</a>
							<a
								v-else-if="dependency.project"
								v-tooltip="formatMessage(messages.downloadProject)"
								:href="`/project/${dependency.project.id}#download`"
								target="_blank"
							>
								<DownloadIcon />
							</a>
						</ButtonStyled>
					</template>
				</VersionPage>
				<section
					v-if="
						flags.alwaysShowVersionDevInfo ||
						projectV3.project_types.includes('mod') ||
						projectV3.project_types.includes('plugin')
					"
					class="flex flex-col overflow-hidden rounded-2xl border-[1px] border-solid border-surface-4 bg-surface-2 p-0"
				>
					<button
						class="group m-0 flex w-full min-w-0 appearance-none items-center gap-3 rounded-2xl rounded-b-none bg-surface-3 p-4 text-left outline-offset-[-3px]"
						@click="devInfoCollapsed = !devInfoCollapsed"
					>
						<DropdownIcon
							aria-hidden="true"
							class="size-5 text-contrast transition-transform"
							:class="{ 'rotate-180': !devInfoCollapsed }"
						/>
						<h3 class="m-0 flex items-center gap-2 text-base font-semibold">
							{{ formatMessage(messages.devInfo) }}
						</h3>
					</button>
					<Collapsible
						:collapsed="devInfoCollapsed"
						class="rounded-b-2xl border-0 border-t border-solid border-surface-4"
					>
						<div class="flex flex-col p-4">
							<p class="mb-3 mt-0 leading-normal">
								<IntlFormatted :message-id="messages.mavenDescription">
									<template #gradle-link="{ children }">
										<a href="https://gradle.org/" class="text-link" target="_blank" rel="noopener">
											<component :is="() => children" />
										</a>
									</template>
									<template #article-link="{ children }">
										<a
											href="https://support.modrinth.com/en/articles/8801191-modrinth-maven"
											class="text-link"
											target="_blank"
											rel="noopener"
										>
											<component :is="() => children" />
										</a>
									</template>
								</IntlFormatted>
							</p>
							<p class="mb-4 mt-0 leading-normal">
								{{ formatMessage(messages.mavenNote) }}
							</p>
							<h4 class="mb-2 mt-0 font-medium text-contrast">
								{{ formatMessage(messages.mavenCoordinates) }}
							</h4>
							<CopyCode :text="coordinatesSnippet" />
							<h4 class="mb-2 mt-4 font-medium text-contrast">
								{{ formatMessage(messages.versionId) }}
							</h4>
							<CopyCode :text="version.id" />
							<h4 class="mb-2 mt-4 font-medium text-contrast">
								{{ formatMessage(messages.gradleSnippet) }}
							</h4>
							<pre
								class="m-0 overflow-x-auto rounded-xl border border-solid border-surface-4 bg-surface-3 text-sm"
								>{{ gradleSnippet }}</pre
							>
						</div>
					</Collapsible>
				</section>
			</template>
			<template v-else-if="versionError">
				Uh oh, something went wrong.
				<pre>
					{{ versionError }}
				</pre
				>
			</template>
			<template v-else-if="showVersionSkeleton">
				<div class="flex flex-col gap-4 pb-[30rem]">
					<div
						class="mt-4 flex h-[8rem] w-full animate-pulse items-center justify-center rounded-2xl bg-surface-3"
					></div>
					<hr class="m-0 w-full animate-pulse border-surface-4" />
					<div class="grid gap-4 sm:grid-cols-2">
						<div
							class="flex h-[6rem] w-full animate-pulse items-center justify-center rounded-2xl bg-surface-3"
						></div>
						<div
							class="flex h-[6rem] w-full animate-pulse items-center justify-center rounded-2xl bg-surface-3"
						></div>
					</div>
					<div
						class="flex h-[18rem] w-full animate-pulse items-center justify-center rounded-2xl bg-surface-3"
					></div>
				</div>
			</template>
		</div>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BoxIcon,
	ChevronLeftIcon,
	ClipboardCopyIcon,
	CopyIcon,
	DownloadIcon,
	DropdownIcon,
	ExternalIcon,
	FileIcon,
	getLoaderIcon,
	InfoIcon,
	MoreVerticalIcon,
	PackageClosedIcon,
	ReportIcon,
	RightArrowIcon,
	SettingsIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	Collapsible,
	commonMessages,
	commonProjectSettingsMessages,
	ConfirmModal,
	CopyCode,
	defineMessages,
	formatLoader,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	IntlFormatted,
	MultiSelect,
	NewModal,
	OverflowMenu,
	useFormatBytes,
	useFormatDateTime,
	useVIntl,
	VersionPage,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { onServerPrefetch } from 'vue'

import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'
import { getSignInRouteObj } from '~/composables/auth.js'
import { projectQueryOptions, STALE_TIME } from '~/composables/queries/project'
import { versionQueryOptions } from '~/composables/queries/version'
import { createDataPackVersion } from '~/helpers/package.js'
import { reportVersion } from '~/utils/report-helpers.ts'

const emit = defineEmits<{
	onDownload: []
}>()

const data = useNuxtApp()
const route = useNativeRoute()
const router = useRouter()
const auth = await useAuth()
const tags = useGeneratedState()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { createProjectDownloadUrl } = useCdnDownloadContext()
const formatDate = useFormatDateTime({ dateStyle: 'medium' })
const formatBytes = useFormatBytes()
const { formatMessage } = useVIntl()

const {
	projectV2: project,
	projectV3,
	currentMember,
	allMembers: members,
	versions,
	versionsLoading,
	loadVersions,
	dependencies: contextDependencies,
	dependenciesLoading,
	loadDependencies,
	invalidate,
	cdnDownloadReason,
} = injectProjectPageContext()

loadVersions()
loadDependencies()

const flags = useFeatureFlags()
const devInfoCollapsed = computed({
	get: () => flags.value.versionDevInfoCollapsed,
	set: (value) => {
		flags.value.versionDevInfoCollapsed = value
		saveFeatureFlags()
	},
})

const baseId = useId()
const signInRouteObj = computed(() => getSignInRouteObj(route))

const versionRouteParam = computed(() => route.params.version as string)
const isLatestRoute = computed(() => versionRouteParam.value === 'latest')

function filterVersionsForLatestRoute(allVersions: Labrinth.Versions.v3.Version[]) {
	let filtered = allVersions

	const loaderFilter = route.query.loader
	if (typeof loaderFilter === 'string') {
		filtered = filtered.filter((x) => x.loaders.includes(loaderFilter))
	}

	const gameVersionFilter = route.query.version
	if (typeof gameVersionFilter === 'string') {
		filtered = filtered.filter((x) => x.game_versions.includes(gameVersionFilter))
	}

	return filtered
}

const latestVersionId = computed(() => {
	if (!isLatestRoute.value) {
		return null
	}

	const filtered = filterVersionsForLatestRoute(versions.value ?? [])
	if (filtered.length === 0) return null

	return filtered.reduce((a, b) => (a.date_published > b.date_published ? a : b)).id
})

const versionLookupKey = computed(() =>
	isLatestRoute.value ? latestVersionId.value : versionRouteParam.value,
)

const {
	data: version,
	refetch: refetchVersion,
	error: versionError,
	isPending: versionPending,
} = useQuery({
	queryKey: computed(
		() => ['project', project.value.id, 'version', 'v3', versionLookupKey.value] as const,
	),
	queryFn: () =>
		client.labrinth.versions_v3.getVersionFromIdOrNumber(project.value.id, versionLookupKey.value!),
	enabled: computed(() => !!project.value.id && !!versionLookupKey.value),
	staleTime: STALE_TIME,
})

const showVersionSkeleton = computed(() => import.meta.client && versionPending.value)

onServerPrefetch(async () => {
	if (!project.value.id) return

	let lookupKey = versionRouteParam.value

	if (isLatestRoute.value) {
		loadVersions()
		const versionsData = await queryClient.ensureQueryData(
			projectQueryOptions.versionsV3(project.value.id, client),
		)
		const filtered = filterVersionsForLatestRoute(versionsData ?? [])
		if (filtered.length === 0) return
		lookupKey = filtered.reduce((a, b) => (a.date_published > b.date_published ? a : b)).id
	}

	if (!lookupKey || lookupKey === 'latest') return

	await queryClient.ensureQueryData(
		versionQueryOptions.fromProject(project.value.id, lookupKey, client),
	)
})

watch(
	versionError,
	(error) => {
		if (error) {
			showError({
				fatal: true,
				statusCode: 404,
				message: 'Version not found',
			})
		}
	},
	{ immediate: true },
)

watch(
	[isLatestRoute, latestVersionId, versionsLoading, versions],
	() => {
		if (isLatestRoute.value && !versionsLoading.value && versions.value && !latestVersionId.value) {
			showError({
				fatal: true,
				statusCode: 404,
				message: 'No version matches the filters',
			})
		}
	},
	{ immediate: true },
)

const enrichment = computed(() => contextDependencies.value ?? undefined)

const primaryFile = computed(
	() => version.value?.files?.find((file) => file.primary) ?? version.value?.files?.[0],
)

const title = computed(() =>
	version.value ? ` ${version.value.version_number} - ${project.value.title}` : undefined,
)

const description = computed(() => {
	if (!version.value) return ''

	return `Download ${project.value.title} ${
		version.value.version_number
	} on Modrinth. Supports ${(data as any).$formatVersion(version.value.game_versions)} ${(
		version.value.loaders ?? []
	)
		.map((x: string) => x.charAt(0).toUpperCase() + x.slice(1))
		.join(
			' & ',
		)}. Published on ${formatDate(version.value.date_published)}. ${version.value.downloads} downloads.`
})

useSeoMeta({
	title,
	description,
	ogTitle: title,
	ogDescription: description,
})

const editModal = useTemplateRef('editModal')
const confirmModal = useTemplateRef('confirmModal')
const packageModal = useTemplateRef('packageModal')

const packageLoaders = ref(['forge', 'fabric', 'quilt', 'neoforge'])
const packageLoaderOptions = [
	{ value: 'fabric', label: formatLoader(formatMessage, 'fabric') },
	{ value: 'forge', label: formatLoader(formatMessage, 'forge') },
	{ value: 'quilt', label: formatLoader(formatMessage, 'quilt') },
	{ value: 'neoforge', label: formatLoader(formatMessage, 'neoforge') },
]

async function handleVersionSaved() {
	await Promise.all([
		invalidate(),
		queryClient.invalidateQueries({ queryKey: ['project', project.value.id, 'version', 'v3'] }),
		refetchVersion(),
	])
}

function handleOpenEditVersionModal(versionId: string, projectId: string, stageId: string) {
	if (!currentMember.value) return
	editModal.value?.openEditVersionModal(versionId, projectId, stageId)
}

const deleteVersionMutation = useMutation({
	mutationFn: () => client.labrinth.versions_v3.deleteVersion(version.value!.id),
	onSuccess: async () => {
		addNotification({
			title: 'Version deleted',
			text: 'The version has been successfully deleted.',
			type: 'success',
		})
		await invalidate()
		await router.replace(`/${project.value.project_type}/${project.value.id}/settings/versions`)
	},
	onError: (err: { data?: { description?: string } }) => {
		addNotification({
			title: 'An error occurred',
			text: err.data?.description ?? String(err),
			type: 'error',
		})
	},
})

async function deleteVersion() {
	startLoading()
	try {
		await deleteVersionMutation.mutateAsync()
	} finally {
		stopLoading()
	}
}

const createDataPackVersionMutation = useMutation({
	mutationFn: async () => {
		if (!version.value || !primaryFile.value) {
			throw new Error('Version data is not available')
		}

		const blob = await createDataPackVersion(
			project.value,
			version.value,
			primaryFile.value,
			members.value ?? [],
			tags.value.gameVersions,
			packageLoaders.value,
		)

		const file = new File([blob], `${project.value.slug}-${version.value.version_number}.jar`)

		const draftVersion: Labrinth.Versions.v3.DraftVersion = {
			project_id: project.value.id,
			name: version.value.name ?? version.value.version_number,
			version_number: `${version.value.version_number}+mod`,
			changelog: version.value.changelog ?? '',
			version_type: version.value.version_type,
			dependencies: version.value.dependencies ?? [],
			game_versions: version.value.game_versions,
			loaders: packageLoaders.value,
			featured: version.value.featured,
			environment: 'server_only',
		}

		const uploadHandle = client.labrinth.versions_v3.createVersion(draftVersion, [{ file }], 'mod')
		return uploadHandle.promise
	},
	onSuccess: async (newVersion) => {
		packageModal.value?.hide()

		addNotification({
			title: 'Packaging Success',
			text: 'Your data pack was successfully packaged as a mod! Make sure to playtest to check for errors.',
			type: 'success',
		})

		await invalidate()
		await router.push(
			`/${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}/version/${newVersion.id}`,
		)
	},
	onError: (err: { data?: { description?: string } }) => {
		addNotification({
			title: 'An error occurred',
			text: err.data?.description ?? String(err),
			type: 'error',
		})
	},
})

async function createDataPackVersionHandler() {
	if (packageLoaders.value.length === 0) return

	startLoading()
	try {
		await createDataPackVersionMutation.mutateAsync()
	} finally {
		stopLoading()
	}
}

const decoratedPrimaryFileUrl = computed(() => {
	const url = primaryFile.value?.url
	if (!url) return undefined
	return createProjectDownloadUrl(url, { reason: cdnDownloadReason.value })
})

function decorateDownloadUrl(url: string) {
	return createProjectDownloadUrl(url, { reason: cdnDownloadReason.value })
}

const messages = defineMessages({
	noPrimaryFile: {
		id: 'version.download.no-primary-file',
		defaultMessage: 'Error: No primary file found',
	},
	edit: {
		id: 'version.edit.button',
		defaultMessage: 'Edit',
	},
	editMetadata: {
		id: 'version.edit.metadata',
		defaultMessage: 'Edit metadata',
	},
	editDetails: {
		id: 'version.edit.details',
		defaultMessage: 'Edit details',
	},
	editFiles: {
		id: 'version.edit.files',
		defaultMessage: 'Edit files',
	},
	packageAsMod: {
		id: 'version.package-as-mod.button',
		defaultMessage: 'Package as mod',
	},
	copySha1: {
		id: 'version.supplementary-resources.copy-hash-sha1',
		defaultMessage: 'Copy SHA-1',
	},
	copySha512: {
		id: 'version.supplementary-resources.copy-hash-sha512',
		defaultMessage: 'Copy SHA-512',
	},
	allVersions: {
		id: 'version.all-versions',
		defaultMessage: 'All versions',
	},
	unknownEmbeddedContent: {
		id: 'version.unknown-embedded-content.title',
		defaultMessage: 'Withheld due to unknown embedded content',
	},
	unknownEmbeddedContentDescription: {
		id: 'version.unknown-embedded-content.description',
		defaultMessage: `This version is currently withheld and not publicly listed. Please provide proof that you have permission to redistribute certain files included.`,
	},
	viewProject: {
		id: 'version.dependency.view-project',
		defaultMessage: `View project`,
	},
	viewVersion: {
		id: 'version.dependency.view-version',
		defaultMessage: `View version`,
	},
	downloadProject: {
		id: 'version.download.download-dependency',
		defaultMessage: 'Download dependency',
	},
	requiredResourcePack: {
		id: 'version.download.required-resource-pack',
		defaultMessage: 'Required resource pack',
	},
	optionalResourcePack: {
		id: 'version.download.optional-resource-pack',
		defaultMessage: 'Optional resource pack',
	},
	packageDataPack: {
		id: 'version.package-as-mod.submit-button',
		defaultMessage: 'Package data pack',
	},
	packageDataPackHeader: {
		id: 'version.package-as-mod.header',
		defaultMessage: 'Package data pack as mod',
	},
	packageDataPackDescription: {
		id: 'version.package-as-mod.description',
		defaultMessage:
			'This will create a new version with support for the selected mod loaders. You will be redirected to the new version and can edit it to your liking.',
	},
	modLoadersLabel: {
		id: 'version.package-as-mod.mod-loaders',
		defaultMessage: 'Mod loaders',
	},
	modLoadersDescription: {
		id: 'version.package-as-mod.mod-loaders.description',
		defaultMessage: 'The mod loaders you would like to package your data pack for.',
	},
	modLoadersPlaceholder: {
		id: 'version.package-as-mod.mod-loaders.placeholder',
		defaultMessage: 'Choose mod loaders...',
	},
	confirmTitle: {
		id: 'version.confirm-delete.title',
		defaultMessage: 'Are you sure you want to delete this version?',
	},
	confirmDescription: {
		id: 'version.confirm-delete.description',
		defaultMessage: 'This version will be permanently deleted. This action cannot be undone.',
	},
	proceedDeletion: {
		id: 'version.confirm-delete.proceed',
		defaultMessage: 'Delete version',
	},
	devInfo: {
		id: 'version.section.content.dev-info',
		defaultMessage: 'Developer information',
	},
	mavenDescription: {
		id: 'version.section.content.dev-info.maven-description',
		defaultMessage:
			'Projects on Modrinth are automatically available through a Maven repository for use with JVM build tools such as <gradle-link>Gradle</gradle-link>. To learn more about the Modrinth Maven API, <article-link>click here</article-link>.',
	},
	mavenNote: {
		id: 'version.section.content.dev-info.maven-note',
		defaultMessage: `Note: When available, you should use the creator's maven repo instead as it will have transitive dependency information that the Modrinth Maven API does not. You may also end up with duplicate dependencies if you use a mix of Modrinth and non-Modrinth Maven repositories for your dependencies, because the group identifier will be different when served through the Modrinth Maven API.`,
	},
	mavenCoordinates: {
		id: 'version.section.content.dev-info.maven-coordinates',
		defaultMessage: `Maven coordinates:`,
	},
	versionId: {
		id: 'version.section.content.dev-info.version-id',
		defaultMessage: `Version ID:`,
	},
	gradleSnippet: {
		id: 'version.section.content.dev-info.gradle-snippet',
		defaultMessage: `build.gradle:`,
	},
})

const copyFileHash = async (
	file: Labrinth.Versions.v3.VersionFile,
	method: Labrinth.Versions.v3.FileHashType,
) => {
	await navigator.clipboard.writeText(file.hashes[method])
}

const dependencyVersionPrimaryFiles = computed(() => {
	const versions = enrichment.value?.versions
	const primaryFileMap: Record<string, Labrinth.Versions.v2.VersionFile> = {}
	versions?.forEach((depVersion) => {
		const depPrimaryFile = depVersion.files.find((file) => file.primary) ?? depVersion.files[0]

		primaryFileMap[depVersion.id] = depPrimaryFile
	})
	return primaryFileMap
})

function createDependencyLink(context: {
	project?: Labrinth.Projects.v2.Project
	version?: Labrinth.Versions.v2.Version
}) {
	const baseUrl = context.version
		? `/project/${context.version.project_id}/version/${context.version.id}`
		: context.project
			? `/project/${context.project.id}`
			: undefined
	return baseUrl
		? createProjectDownloadUrl(baseUrl, {
				reason: 'dependency',
			})
		: undefined
}

const coordinatesSnippet = computed(() => `maven.modrinth:${project.value.id}:${version.value?.id}`)
const gradleSnippet = computed(
	() => `repositories {
    exclusiveContent {
        forRepository {
            maven {
                name = "Modrinth"
                url = "https://api.modrinth.com/maven"
            }
        }
        // forRepositories(fg.repository) // Uncomment when using ForgeGradle
        filter {
            includeGroup "maven.modrinth"
        }
    }
}

// Standard Gradle dependency
dependencies {
    implementation "${coordinatesSnippet.value}"
}

// Legacy Loom dependency
dependencies {
    modImplementation "${coordinatesSnippet.value}"
}`,
)

const moderator = computed(() => isStaff(auth.value?.user))
</script>
