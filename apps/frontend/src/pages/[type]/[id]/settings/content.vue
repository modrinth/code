<template>
	<div>
		<CreateServerVersionModal
			v-if="currentMember"
			ref="create-server-version-modal"
			@submit="handleVersionSubmit"
		/>

		<ConfirmModal
			v-if="currentMember"
			ref="deleteVersionModal"
			title="Are you sure you want to delete this version?"
			description="This will remove this version forever (like really forever)."
			:has-to-type="false"
			proceed-label="Delete"
			@proceed="deleteVersion()"
		/>

		<Table
			v-if="mockVersions.length > 0"
			:columns="columns"
			:data="mockVersions"
			v-model:sort-column="sortColumn"
			v-model:sort-direction="sortDirection"
		>
			<template #cell-name="{ row }">
				<div class="flex items-center gap-2">
					<span class="font-semibold text-contrast">{{ row.name }}</span>
					<TagItem
						v-if="row.isActive"
						class="border !border-solid border-brand bg-highlight-green text-brand"
						:style="`--_color: var(--color-brand)`"
					>
						Active
					</TagItem>
				</div>
			</template>
			<template #cell-requiredContent="{ value }">
				<span>{{ value }}</span>
			</template>
			<template #cell-gameVersion="{ value }">
				<span>{{ value }}</span>
			</template>
			<template #cell-published="{ value }">
				<span>{{ value }}</span>
			</template>
			<template #cell-actions="{ row }">
				<div class="flex justify-end">
					<ButtonStyled circular type="transparent">
						<OverflowMenu
							v-tooltip="'More options'"
							class="hover:!bg-button-bg"
							:dropdown-id="`${baseDropdownId}-${row.id}`"
							:options="[
								{
									id: 'edit',
									action: () => handleOpenEditVersionModal(row.id),
								},
								{
									id: 'set-active',
									action: () => setActiveVersion(row.id),
									shown: !row.isActive,
								},
								{
									id: 'copy-id',
									action: () => copyToClipboard(row.id),
								},
								{ divider: true },
								{
									id: 'delete',
									color: 'red' as const,
									hoverFilled: true,
									action: () => handleDeleteVersion(row.id),
								},
							]"
							aria-label="More options"
						>
							<MoreVerticalIcon aria-hidden="true" />
							<template #edit>
								<EditIcon aria-hidden="true" />
								Edit
							</template>
							<template #set-active>
								<CheckIcon aria-hidden="true" />
								Set as active
							</template>
							<template #copy-id>
								<ClipboardCopyIcon aria-hidden="true" />
								Copy ID
							</template>
							<template #delete>
								<TrashIcon aria-hidden="true" />
								Delete
							</template>
						</OverflowMenu>
					</ButtonStyled>
				</div>
			</template>
		</Table>

		<template v-if="!mockVersions.length">
			<div class="grid place-items-center py-10">
				<svg
					width="250"
					height="200"
					viewBox="0 0 250 200"
					fill="none"
					xmlns="http://www.w3.org/2000/svg"
					class="h-[200px] w-[250px]"
				>
					<path
						d="M136 64C139.866 64 143 67.134 143 71C143 74.866 139.866 78 136 78H200C203.866 78 207 81.134 207 85C207 88.866 203.866 92 200 92H222C225.866 92 229 95.134 229 99C229 102.866 225.866 106 222 106H203C199.134 106 196 109.134 196 113C196 116.866 199.134 120 203 120H209C212.866 120 216 123.134 216 127C216 130.866 212.866 134 209 134H157C156.485 134 155.983 133.944 155.5 133.839C155.017 133.944 154.515 134 154 134H63C59.134 134 56 130.866 56 127C56 123.134 59.134 120 63 120H24C20.134 120 17 116.866 17 113C17 109.134 20.134 106 24 106H64C67.866 106 71 102.866 71 99C71 95.134 67.866 92 64 92H39C35.134 92 32 88.866 32 85C32 81.134 35.134 78 39 78H79C75.134 78 72 74.866 72 71C72 67.134 75.134 64 79 64H136ZM226 120C229.866 120 233 123.134 233 127C233 130.866 229.866 134 226 134C222.134 134 219 130.866 219 127C219 123.134 222.134 120 226 120Z"
						class="fill-surface-2"
					/>
					<path
						fill-rule="evenodd"
						clip-rule="evenodd"
						d="M113.119 112.307C113.04 112.86 113 113.425 113 114C113 120.627 118.373 126 125 126C131.627 126 137 120.627 137 114C137 113.425 136.96 112.86 136.881 112.307H166V139C166 140.657 164.657 142 163 142H87C85.3431 142 84 140.657 84 139V112.307H113.119Z"
						class="fill-surface-1"
					/>
					<path
						fill-rule="evenodd"
						clip-rule="evenodd"
						d="M138 112C138 119.18 132.18 125 125 125C117.82 125 112 119.18 112 112C112 111.767 112.006 111.536 112.018 111.307H84L93.5604 83.0389C93.9726 81.8202 95.1159 81 96.4023 81H153.598C154.884 81 156.027 81.8202 156.44 83.0389L166 111.307H137.982C137.994 111.536 138 111.767 138 112Z"
						class="fill-surface-1"
					/>
					<path
						fill-rule="evenodd"
						clip-rule="evenodd"
						d="M136.098 112.955C136.098 118.502 131.129 124 125 124C118.871 124 113.902 118.502 113.902 112.955C113.902 112.775 113.908 111.596 113.918 111.419H93L101.161 91.5755C101.513 90.6338 102.489 90 103.587 90H146.413C147.511 90 148.487 90.6338 148.839 91.5755L157 111.419H136.082C136.092 111.596 136.098 112.775 136.098 112.955Z"
						fill="#27292E"
						class="fill-surface-3"
					/>
					<path
						fill-rule="evenodd"
						clip-rule="evenodd"
						d="M85.25 111.512V138C85.25 138.966 86.0335 139.75 87 139.75H163C163.966 139.75 164.75 138.966 164.75 138V111.512L155.255 83.4393C155.015 82.7285 154.348 82.25 153.598 82.25H96.4023C95.6519 82.25 94.985 82.7285 94.7446 83.4393L85.25 111.512Z"
						stroke-width="2.5"
						class="stroke-surface-4"
					/>
					<path
						d="M98 111C101.937 111 106.185 111 110.745 111C112.621 111 112.621 112.319 112.621 113C112.621 119.627 118.117 125 124.897 125C131.677 125 137.173 119.627 137.173 113C137.173 112.319 137.173 111 139.05 111H164M90.5737 111H93H90.5737Z"
						stroke-width="2.5"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="stroke-surface-4"
					/>
					<path
						d="M150.1 58.3027L139 70.7559M124.1 54V70.7559V54ZM98 58.3027L109.1 70.7559L98 58.3027Z"
						stroke-width="2.5"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="stroke-surface-4"
					/>
				</svg>

				<div class="flex max-w-[320px] flex-col items-center gap-2 text-center">
					<div class="text-pretty text-2xl font-semibold text-contrast">
						Does your server require a modpack?
					</div>
					<div class="text-balance">
						Select the modpack and Modrinth installs it for players when they join.
					</div>
					<br />
					<ButtonStyled color="green">
						<button @click="() => createServerVersionModal?.show()">
							<PlusIcon /> Select modpack
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>
	</div>
</template>

<script lang="ts" setup>
import {
	CheckIcon,
	ClipboardCopyIcon,
	EditIcon,
	MoreVerticalIcon,
	PlusIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmModal,
	type CreateServerVersionData,
	CreateServerVersionModal,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	OverflowMenu,
	Table,
	type TableColumn,
	TagItem,
} from '@modrinth/ui'
import { ref, useTemplateRef } from 'vue'

const { projectV2: project, currentMember } = injectProjectPageContext()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { refreshVersions } = injectProjectPageContext()

const createServerVersionModal = useTemplateRef('create-server-version-modal')
const deleteVersionModal = ref<InstanceType<typeof ConfirmModal>>()
const selectedVersion = ref<string | null>(null)

interface ServerVersion {
	id: string
	name: string
	isActive: boolean
	requiredContent: string
	gameVersion: string
	published: string
}

const mockVersions = ref<ServerVersion[]>([
	{
		id: '1',
		name: '1.0.2',
		isActive: true,
		requiredContent: 'Complex Cobblemon Pack 1.6.4',
		gameVersion: '1.21.10',
		published: 'Last week',
	},
	{
		id: '2',
		name: '1.0.1',
		isActive: false,
		requiredContent: 'Complex Cobblemon Pack 1.6.4',
		gameVersion: '1.21.10',
		published: 'Last month',
	},
	{
		id: '3',
		name: '1.0.0',
		isActive: false,
		requiredContent: 'Complex Cobblemon Pack 1.6.4',
		gameVersion: '1.21.10',
		published: '2 months ago',
	},
])

const columns: TableColumn<keyof ServerVersion | 'actions'>[] = [
	{ key: 'name', label: 'Name', enableSorting: true },
	{ key: 'requiredContent', label: 'Required content' },
	{ key: 'gameVersion', label: 'Game version' },
	{ key: 'published', label: 'Published' },
	{ key: 'actions', label: 'Actions', align: 'right', width: '80px' },
]

const sortColumn = ref<string | undefined>('name')
const sortDirection = ref<'asc' | 'desc'>('asc')

const baseDropdownId = useId()

const handleDeleteVersion = (versionId: string) => {
	selectedVersion.value = versionId
	deleteVersionModal.value?.show()
}

const handleOpenCreateVersionModal = () => {
	if (!currentMember) return
	createServerVersionModal.value?.show()
}

const handleOpenEditVersionModal = (versionId: string) => {
	if (!currentMember) return
	console.log('Edit version:', versionId)
}

const setActiveVersion = (versionId: string) => {
	mockVersions.value = mockVersions.value.map((v) => ({
		...v,
		isActive: v.id === versionId,
	}))
}

async function copyToClipboard(text: string) {
	await navigator.clipboard.writeText(text)
}

function handleVersionSubmit(data: CreateServerVersionData) {
	console.log('Version submit:', data)
	createServerVersionModal.value?.hide()
}

async function deleteVersion() {
	const id = selectedVersion.value
	if (!id) return

	startLoading()

	try {
		await client.labrinth.versions_v3.deleteVersion(id)

		addNotification({
			title: 'Version deleted',
			text: 'The version has been successfully deleted.',
			type: 'success',
		})
	} catch (err: any) {
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}

	refreshVersions()
	selectedVersion.value = null

	stopLoading()
}
</script>
