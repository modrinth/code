<template>
	<div>
		<NewModal
			ref="modalOpen"
			:header="formatMessage(messages.transferProjectsTitle)"
			no-padding
			width="60rem"
		>
			<div class="max-h-[70vh] overflow-y-auto">
				<Table
					class="!rounded-none !border-0"
					:columns="projectTableColumns"
					:data="props.projects"
					row-key="id"
					table-min-width="42rem"
					table-layout="auto"
				>
					<template #empty-state>
						<div class="flex h-48 items-center justify-center text-secondary">
							{{ formatMessage(messages.noProjectsAvailable) }}
						</div>
					</template>
					<template #header-select>
						<Checkbox
							class="h-full w-full justify-center"
							:model-value="allTransferableProjectsSelected"
							@update:model-value="toggleAllTransferableProjects()"
						/>
					</template>
					<template #cell-select="{ row: project }">
						<Checkbox
							class="h-full w-full justify-center"
							:disabled="isProjectTransferDisabled(project)"
							:model-value="isProjectSelected(project)"
							@update:model-value="toggleProjectSelection(project)"
						/>
					</template>
					<template #cell-name="{ row: project }">
						<nuxt-link class="project-name-cell" :to="getProjectUrl(project)">
							<span
								class="flex size-8 shrink-0 items-center justify-center overflow-hidden rounded text-primary"
							>
								<img
									v-if="project.icon_url"
									:src="project.icon_url"
									:alt="formatMessage(messages.projectIconAlt, { name: project.title })"
									class="h-full w-full rounded object-cover"
								/>
								<BoxIcon v-else class="h-full w-full" />
							</span>
							<span class="wrap-as-needed">{{ project.title }}</span>
						</nuxt-link>
					</template>
					<template #cell-id="{ row: project }">
						<div class="flex items-center">
							<CopyCode :text="project.id" />
						</div>
					</template>
					<template #cell-type="{ row: project }">
						<div class="flex items-center">
							{{ getProjectDisplayType(project) }}
						</div>
					</template>
				</Table>
			</div>
			<template #actions>
				<div class="flex justify-end gap-2">
					<ButtonStyled type="outlined">
						<button @click="hide()">
							<XIcon />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button :disabled="selectedProjects.length === 0" @click="submitTransfer()">
							<TransferIcon />
							{{
								formatMessage(messages.transferSelectedProjects, {
									count: selectedProjects.length,
								})
							}}
						</button>
					</ButtonStyled>
				</div>
			</template>
		</NewModal>
		<ButtonStyled>
			<button @click="show($event)">
				<TransferIcon />
				<span>{{ formatMessage(messages.transferProjectsTitle) }}</span>
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup>
import { BoxIcon, TransferIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Checkbox,
	commonMessages,
	CopyCode,
	defineMessages,
	NewModal,
	Table,
	useVIntl,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'

import { getProjectTypeForUrl } from '~/helpers/projects.js'

const EDIT_DETAILS = 1 << 2

const props = defineProps({
	projects: {
		type: Array,
		required: true,
	},
})

const emit = defineEmits(['submit'])
const { formatMessage } = useVIntl()

const messages = defineMessages({
	transferProjectsTitle: {
		id: 'organization.project-transfer.title',
		defaultMessage: 'Transfer projects',
	},
	noProjectsAvailable: {
		id: 'organization.project-transfer.no-projects-available',
		defaultMessage: 'No projects available to transfer.',
	},
	projectIconAlt: {
		id: 'organization.project-transfer.project-icon-alt',
		defaultMessage: 'Icon for {name}',
	},
	transferSelectedProjects: {
		id: 'organization.project-transfer.transfer-selected-projects',
		defaultMessage: 'Transfer {count, plural, one {# project} other {# projects}}',
	},
	nameColumn: {
		id: 'organization.project-transfer.name-column',
		defaultMessage: 'Name',
	},
	idColumn: {
		id: 'organization.project-transfer.id-column',
		defaultMessage: 'ID',
	},
	typeColumn: {
		id: 'organization.project-transfer.type-column',
		defaultMessage: 'Type',
	},
})

const modalOpen = ref(null)
const selectedProjectIds = ref([])

const projectTableColumns = computed(() => [
	{
		key: 'select',
		width: '3rem',
		headerClass: '!p-0',
		cellClass: '!overflow-visible !p-0',
	},
	{
		key: 'name',
		label: formatMessage(messages.nameColumn),
		width: '22rem',
	},
	{
		key: 'id',
		label: formatMessage(messages.idColumn),
		width: '13rem',
		cellClass: '!overflow-visible',
	},
	{
		key: 'type',
		label: formatMessage(messages.typeColumn),
		width: '10rem',
	},
])

const transferableProjects = computed(() =>
	props.projects.filter((project) => !isProjectTransferDisabled(project)),
)
const selectedProjects = computed(() =>
	props.projects.filter((project) => selectedProjectIds.value.includes(project.id)),
)
const allTransferableProjectsSelected = computed(
	() =>
		transferableProjects.value.length > 0 &&
		transferableProjects.value.every((project) => selectedProjectIds.value.includes(project.id)),
)

function isProjectTransferDisabled(project) {
	return (project.permissions & EDIT_DETAILS) === EDIT_DETAILS
}

function isProjectSelected(project) {
	return selectedProjectIds.value.includes(project.id)
}

function toggleProjectSelection(project) {
	if (isProjectTransferDisabled(project)) return

	if (isProjectSelected(project)) {
		selectedProjectIds.value = selectedProjectIds.value.filter((id) => id !== project.id)
		return
	}

	selectedProjectIds.value = [...selectedProjectIds.value, project.id]
}

function toggleAllTransferableProjects() {
	selectedProjectIds.value = allTransferableProjectsSelected.value
		? []
		: transferableProjects.value.map((project) => project.id)
}

function getProjectUrl(project) {
	const projectType = getProjectTypeForUrl(project.project_type, project.loaders)
	return '/' + projectType + '/' + (project.slug || project.id)
}

function getProjectDisplayType(project) {
	return formatProjectType(getProjectTypeForUrl(project.project_type, project.loaders))
}

function show(event) {
	modalOpen.value?.show(event)
}

function hide() {
	modalOpen.value?.hide()
}

function submitTransfer() {
	if (selectedProjects.value.length === 0) return

	emit('submit', selectedProjects.value)
	selectedProjectIds.value = []
	hide()
}
</script>

<style lang="scss" scoped>
.project-name-cell {
	display: flex;
	min-width: 0;
	min-height: 3.5rem;
	align-items: center;
	gap: var(--spacing-card-sm);
	color: inherit;
	text-decoration: none;
}

.project-name-cell:hover,
.project-name-cell:focus-visible {
	text-decoration: underline;
}
</style>
