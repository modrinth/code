<template>
	<div>
		<CreateProjectVersionModal ref="modal"></CreateProjectVersionModal>
		<div class="universal-card py-4">
			<div class="markdown-disclaimer">
				<div class="flex items-center justify-between">
					<span class="text-xl font-semibold text-contrast">Versions</span>

					<ButtonStyled color="green">
						<button @click="openModal"><PlusIcon /> Create version</button>
					</ButtonStyled>
				</div>
			</div>

			<ConfirmModal
				v-if="currentMember"
				ref="deleteVersionModal"
				title="Are you sure you want to delete this version?"
				description="This will remove this version forever (like really forever)."
				:has-to-type="false"
				proceed-label="Delete"
				@proceed="deleteVersion()"
			/>
		</div>

		<ProjectPageVersions
			:project="project"
			:versions="versions.map((v) => ({ ...v, displayUrlEnding: v.version_number }))"
			:show-files="flags.showVersionFilesInTable"
			:current-member="!!currentMember"
			:loaders="tags.loaders"
			:game-versions="tags.gameVersions"
			:base-id="baseDropdownId"
			:version-link="
				(version: any) =>
					`/${project.project_type}/${
						project.slug ? project.slug : project.id
					}/version/${encodeURI(version.displayUrlEnding)}`
			"
		>
			<template #actions="{ version }">
				<ButtonStyled circular type="transparent">
					<a
						v-tooltip="`Download`"
						:href="getPrimaryFile(version).url"
						class="group-hover:!bg-brand group-hover:[&>svg]:!text-brand-inverted"
						aria-label="Download"
						@click="emit('onDownload')"
					>
						<DownloadIcon aria-hidden="true" />
					</a>
				</ButtonStyled>
				<ButtonStyled circular type="transparent">
					<OverflowMenu
						class="group-hover:!bg-button-bg"
						:dropdown-id="`${baseDropdownId}-${version.id}`"
						:options="[
							{
								id: 'download',
								color: 'primary',
								hoverFilled: true,
								link: getPrimaryFile(version).url,
								action: () => {
									emit('onDownload')
								},
							},
							{
								id: 'new-tab',
								action: () => {},
								link: `/${project.project_type}/${
									project.slug ? project.slug : project.id
								}/version/${encodeURI(version.displayUrlEnding)}`,
								external: true,
							},
							{
								id: 'copy-link',
								action: () =>
									copyToClipboard(
										`https://modrinth.com/${project.project_type}/${
											project.slug ? project.slug : project.id
										}/version/${encodeURI(version.displayUrlEnding)}`,
									),
							},
							{
								id: 'share',
								action: () => {},
								shown: false,
							},
							{
								id: 'report',
								color: 'red',
								hoverFilled: true,
								action: () => (auth.user ? reportVersion(version.id) : navigateTo('/auth/sign-in')),
								shown: !currentMember,
							},
							{ divider: true, shown: !!currentMember || flags.developerMode },
							{
								id: 'copy-id',
								action: () => {
									copyToClipboard(version.id)
								},
								shown: !!currentMember || flags.developerMode,
							},
							{
								id: 'copy-maven',
								action: () => {
									copyToClipboard(`maven.modrinth:${project.slug}:${version.id}`)
								},
								shown: flags.developerMode,
							},
							{ divider: true, shown: !!currentMember },
							{
								id: 'edit',
								link: `/${project.project_type}/${
									project.slug ? project.slug : project.id
								}/version/${encodeURI(version.displayUrlEnding)}/edit`,
								shown: !!currentMember,
							},
							{
								id: 'delete',
								color: 'red',
								hoverFilled: true,
								action: () => {
									selectedVersion = version.id
									deleteVersionModal?.show()
								},
								shown: !!currentMember,
							},
						]"
						aria-label="More options"
					>
						<MoreVerticalIcon aria-hidden="true" />
						<template #download>
							<DownloadIcon aria-hidden="true" />
							Download
						</template>
						<template #new-tab>
							<ExternalIcon aria-hidden="true" />
							Open in new tab
						</template>
						<template #copy-link>
							<LinkIcon aria-hidden="true" />
							Copy link
						</template>
						<template #share>
							<ShareIcon aria-hidden="true" />
							Share
						</template>
						<template #report>
							<ReportIcon aria-hidden="true" />
							Report
						</template>
						<template #edit>
							<EditIcon aria-hidden="true" />
							Edit
						</template>
						<template #delete>
							<TrashIcon aria-hidden="true" />
							Delete
						</template>
						<template #copy-id>
							<ClipboardCopyIcon aria-hidden="true" />
							Copy ID
						</template>
						<template #copy-maven>
							<ClipboardCopyIcon aria-hidden="true" />
							Copy Maven coordinates
						</template>
					</OverflowMenu>
				</ButtonStyled>
			</template>
		</ProjectPageVersions>
	</div>
</template>

<script lang="ts" setup>
import { ButtonStyled, ConfirmModal, ProjectPageVersions } from '@modrinth/ui'

import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'
import { provideVersionsContext } from '~/providers/versions'

import {
	ClipboardCopyIcon,
	DownloadIcon,
	EditIcon,
	ExternalIcon,
	LinkIcon,
	MoreVerticalIcon,
	ReportIcon,
	ShareIcon,
	TrashIcon,
} from '@modrinth/assets'
import { OverflowMenu } from '@modrinth/ui'

import type { Labrinth } from '@modrinth/api-client'
import { reportVersion } from '~/utils/report-helpers.ts'
import { PlusIcon } from '../../../../../../../packages/assets/generated-icons'

interface Props {
	project: Labrinth.Projects.v2.Project
	versions: Labrinth.Versions.v3.Version[]
	currentMember?: Object
}

const { project, versions, currentMember } = defineProps<Props>()

provideVersionsContext({
	project,
	versions,
})

const modal = ref<InstanceType<typeof CreateProjectVersionModal>>()

function openModal() {
	modal.value?.show?.()
}

const tags = useGeneratedState()
const flags = useFeatureFlags()
const auth = await useAuth()

const deleteVersionModal = ref<InstanceType<typeof ConfirmModal>>()
const selectedVersion = ref<string | null>(null)

const emit = defineEmits(['onDownload', 'deleteVersion'])

const baseDropdownId = useId()

function getPrimaryFile(version: Labrinth.Versions.v3.Version) {
	return version.files.find((x) => x.primary) || version.files[0]
}

async function copyToClipboard(text: string) {
	await navigator.clipboard.writeText(text)
}

function deleteVersion() {
	emit('deleteVersion', selectedVersion.value)
	selectedVersion.value = null
}
</script>
