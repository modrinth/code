<template>
	<div data-project-field="versions" data-project-field-scroll-only>
		<ProjectIssueCard location="versions" class="mb-4" />
		<CreateProjectVersionModal
			v-if="currentMember"
			ref="create-project-version-modal"
		></CreateProjectVersionModal>

		<ConfirmModal
			v-if="currentMember"
			ref="deleteVersionModal"
			:title="formatMessage(messages.deleteVersionConfirmTitle)"
			:description="formatMessage(messages.deleteVersionConfirmDescription)"
			:has-to-type="false"
			:proceed-label="formatMessage(messages.deleteButton)"
			@proceed="deleteVersion()"
		/>
		<Admonition
			v-if="withheldVersions.length > 0"
			type="circle-warning"
			class="mb-4"
			:header="
				formatMessage(messages.withheldVersionsWarningTitle, {
					count: withheldVersions.length,
					version_name:
						withheldVersions.length === 1 ? withheldVersions[0].version_number : undefined,
				})
			"
			:body="
				formatMessage(messages.withheldVersionsWarningDescription, {
					count: withheldVersions.length,
					version_name:
						withheldVersions.length === 1 ? withheldVersions[0].version_number : undefined,
				})
			"
		>
			<template #actions>
				<div class="flex">
					<ButtonLink
						type="colored"
						color="orange"
						:to="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/permissions`"
					>
						{{ formatMessage(commonProjectSettingsMessages.withheldVersionsWarningResolve) }}
						<RightArrowIcon />
					</ButtonLink>
				</div>
			</template>
		</Admonition>
		<ProjectPageVersions
			v-if="versions?.length"
			:project="project"
			:versions="versionsWithDisplayUrl"
			:show-files="flags.showVersionFilesInTable"
			:show-environment-column="flags.showVersionEnvironmentColumn"
			:current-member="!!currentMember"
			:loaders="tags.loaders"
			:game-versions="tags.gameVersions"
			:version-link="
				(version: any) =>
					`/${project.project_type}/${
						project.slug ? project.slug : project.id
					}/version/${encodeURI(version.displayUrlEnding ? version.displayUrlEnding : version.id)}`
			"
			:open-modal="currentMember ? () => handleOpenCreateVersionModal() : undefined"
		>
			<template #heading>
				<div class="flex w-full flex-col gap-3">
					<SettingsSection :title="formatMessage(commonProjectSettingsMessages.versions)" />
					<ValidationMessage :check="environmentValidation" />
				</div>
			</template>
			<template #version-issue="{ version }">
				<ProjectIssueCard target="version" :version-id="version.id" />
			</template>
			<template #actions="{ version }">
				<EditVersionMenu @edit="handleOpenEditVersionModal(version.id, project.id, $event)" />
				<TeleportOverflowMenu
					type="quiet"
					:label="formatMessage(commonMessages.moreOptionsButton)"
					:tooltip="formatMessage(commonMessages.moreOptionsButton)"
					class="hover:!bg-button-bg"
					:options="[
						{
							id: 'download',
							label: formatMessage(commonMessages.downloadButton),
							type: 'link',
							tone: 'brand',
							hoverFilled: true,
							href: createDownloadUrl(version),
							download: getPrimaryFile(version)?.filename,
							shown: !!getPrimaryFile(version),
						},
						{
							id: 'new-tab',
							label: formatMessage(messages.openInNewTabOption),
							type: 'link',
							href: `/${project.project_type}/${
								project.slug ? project.slug : project.id
							}/version/${encodeURI(version.displayUrlEnding ? version.displayUrlEnding : version.id)}`,
							target: '_blank',
						},
						{
							id: 'copy-link',
							label: formatMessage(messages.copyLinkOption),
							action: () =>
								copyToClipboard(
									`https://modrinth.com/${project.project_type}/${
										project.slug ? project.slug : project.id
									}/version/${encodeURI(version.displayUrlEnding ? version.displayUrlEnding : version.id)}`,
								),
						},
						{
							id: 'share',
							label: formatMessage(messages.shareOption),
							action: () => {},
							shown: false,
						},
						{
							id: 'report',
							label: formatMessage(commonMessages.reportButton),
							tone: 'red',
							hoverFilled: true,
							action: () =>
								auth.user ? reportVersion(version.id) : promptAuth(getSignInRouteObj(route)),
							shown: !currentMember,
						},
						{ type: 'divider', shown: !!currentMember || flags.developerMode },
						{
							id: 'copy-id',
							label: formatMessage(messages.copyIdOption),
							action: () => {
								copyToClipboard(version.id)
							},
							shown: !!currentMember || flags.developerMode,
						},
						{
							id: 'copy-maven',
							label: formatMessage(messages.copyMavenCoordinatesOption),
							action: () => {
								copyToClipboard(`maven.modrinth:${project.slug}:${version.id}`)
							},
							shown: flags.developerMode,
						},
						{ type: 'divider', shown: !!currentMember },
						{
							id: 'edit-details',
							label: formatMessage(messages.editDetailsOption),
							action: () => handleOpenEditVersionModal(version.id, project.id, 'add-details'),
							shown: !!currentMember,
						},
						{
							id: 'edit-metadata',
							label: formatMessage(messages.editMetadataOption),
							action: () => handleOpenEditVersionModal(version.id, project.id, 'metadata'),
							shown: !!currentMember,
						},
						{
							id: 'edit-files',
							label: formatMessage(messages.editFilesOption),
							action: () => handleOpenEditVersionModal(version.id, project.id, 'add-files'),
							shown: !!currentMember,
						},
						{
							id: 'delete',
							label: formatMessage(commonMessages.deleteLabel),
							tone: 'red',
							hoverFilled: true,
							action: () => {
								selectedVersion = version.id
								deleteVersionModal?.show()
							},
							shown: !!currentMember,
						},
					]"
					@select="$event.id === 'download' && emit('onDownload')"
				>
					<MoreVerticalIcon aria-hidden="true" />
					<template #download>
						<DownloadIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.downloadButton) }}
					</template>
					<template #new-tab>
						<ExternalIcon aria-hidden="true" />
						{{ formatMessage(messages.openInNewTabOption) }}
					</template>
					<template #copy-link>
						<LinkIcon aria-hidden="true" />
						{{ formatMessage(messages.copyLinkOption) }}
					</template>
					<template #share>
						<ShareIcon aria-hidden="true" />
						{{ formatMessage(messages.shareOption) }}
					</template>
					<template #report>
						<ReportIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.reportButton) }}
					</template>
					<template #edit-files>
						<FileIcon aria-hidden="true" />
						{{ formatMessage(messages.editFilesOption) }}
					</template>
					<template #edit-details>
						<InfoIcon aria-hidden="true" />
						{{ formatMessage(messages.editDetailsOption) }}
					</template>
					<template #edit-metadata>
						<BoxIcon aria-hidden="true" />
						{{ formatMessage(messages.editMetadataOption) }}
					</template>
					<template #delete>
						<TrashIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.deleteLabel) }}
					</template>
					<template #copy-id>
						<ClipboardCopyIcon aria-hidden="true" />
						{{ formatMessage(messages.copyIdOption) }}
					</template>
					<template #copy-maven>
						<ClipboardCopyIcon aria-hidden="true" />
						{{ formatMessage(messages.copyMavenCoordinatesOption) }}
					</template>
				</TeleportOverflowMenu>
			</template>
		</ProjectPageVersions>

		<template v-else>
			<ValidationMessage :check="environmentValidation" class="mb-4" />
			<VersionsEmptyState @create="createProjectVersionModal?.openCreateVersionModal()" />
		</template>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@modrinth/api-client'
import {
	BoxIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	ExternalIcon,
	FileIcon,
	InfoIcon,
	LinkIcon,
	MoreVerticalIcon,
	ReportIcon,
	RightArrowIcon,
	ShareIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Admonition,
	ButtonLink,
	commonMessages,
	commonProjectSettingsMessages,
	ConfirmModal,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	ProjectPageVersions,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { useTemplateRef, watch } from 'vue'

import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'
import EditVersionMenu from '~/components/ui/create-project-version/EditVersionMenu.vue'
import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { getSignInRouteObj } from '~/composables/auth.ts'
import { promptAuth } from '~/composables/auth-modal.ts'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { reportVersion } from '~/utils/report-helpers.ts'

import SettingsSection from '../settings-section.vue'
import VersionsEmptyState from './empty-state.vue'
import { messages } from './messages'

const route = useRoute()

const { createProjectDownloadUrl, updateVersionsFilterContext } = useCdnDownloadContext()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const {
	projectV2: project,
	currentMember,
	versions,
	invalidate,
	loadVersions,
	cdnDownloadReason,
} = injectProjectPageContext()
const environmentValidation = useProjectNagMessages('version-environment')

// Load versions on mount (client-side)
onMounted(() => {
	loadVersions()
})

const tags = useGeneratedState()
const flags = useFeatureFlags()
const auth = await useAuth()

const createProjectVersionModal = useTemplateRef('create-project-version-modal')
const deleteVersionModal = ref<InstanceType<typeof ConfirmModal>>()
const selectedVersion = ref<string | null>(null)

const handleOpenCreateVersionModal = () => {
	if (!currentMember.value) return
	createProjectVersionModal.value?.openCreateVersionModal()
}

const handleOpenEditVersionModal = (
	versionId: string,
	projectId: string,
	stageId?: string | null,
) => {
	if (!currentMember.value) return
	createProjectVersionModal.value?.openEditVersionModal(versionId, projectId, stageId)
}

const versionsWithDisplayUrl = computed(() =>
	(versions.value ?? []).map((v) => ({
		...v,
		displayUrlEnding: v.id,
	})),
)

const emit = defineEmits(['onDownload'])

function getPrimaryFile(version: Labrinth.Versions.v3.Version) {
	return version.files?.find((x) => x.primary) || version.files?.[0]
}

watch(
	() => [route.query.g, route.query.l],
	() => {
		updateVersionsFilterContext(
			queryAsStringArray(route.query.g),
			queryAsStringArray(route.query.l),
		)
	},
	{ immediate: true },
)

function createDownloadUrl(version: Labrinth.Versions.v3.Version) {
	const file = getPrimaryFile(version)
	if (!file?.url) return undefined

	return createProjectDownloadUrl(file.url, {
		reason: cdnDownloadReason.value,
	})
}

async function copyToClipboard(text: string) {
	await navigator.clipboard.writeText(text)
}

async function deleteVersion() {
	const id = selectedVersion.value
	if (!id) return

	startLoading()

	try {
		await client.labrinth.versions_v3.deleteVersion(id)

		addNotification({
			title: formatMessage(messages.versionDeletedTitle),
			text: formatMessage(messages.versionDeletedText),
			type: 'success',
		})
	} catch (err: any) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}

	await invalidate()
	selectedVersion.value = null

	stopLoading()
}

const withheldVersions = computed(() =>
	versions.value.filter((x) => x.files_missing_attribution?.length > 0),
)
</script>
