<template>
	<div>
		<NewModal ref="editLinksModal" :header="formatMessage(messages.editLinksHeader)">
			<div class="universal-modal links-modal !p-0">
				<p>{{ formatMessage(messages.editLinksDescription) }}</p>
				<section class="links">
					<label for="issue-tracker-input" :title="formatMessage(messages.issueTrackerDescription)">
						<span class="label__title">{{ formatMessage(messages.issueTrackerLabel) }}</span>
					</label>
					<div class="flex gap-2">
						<StyledInput
							id="issue-tracker-input"
							v-model="editLinks.issues.val"
							:disabled="editLinks.issues.clear"
							type="url"
							class="w-full"
							:placeholder="getLinkInputPlaceholder(editLinks.issues.clear)"
							:maxlength="2048"
						/>
						<ButtonStyled circular>
							<button
								v-tooltip="formatMessage(messages.clearLinkLabel)"
								class="label-button"
								:aria-label="formatMessage(messages.clearLinkLabel)"
								:data-active="editLinks.issues.clear"
								@click="editLinks.issues.clear = !editLinks.issues.clear"
							>
								<TrashIcon />
							</button>
						</ButtonStyled>
					</div>
					<label for="source-code-input" :title="formatMessage(messages.sourceCodeDescription)">
						<span class="label__title">{{ formatMessage(messages.sourceCodeLabel) }}</span>
					</label>
					<div class="flex gap-2">
						<StyledInput
							id="source-code-input"
							v-model="editLinks.source.val"
							:disabled="editLinks.source.clear"
							type="url"
							class="w-full"
							:maxlength="2048"
							:placeholder="getLinkInputPlaceholder(editLinks.source.clear)"
						/>
						<ButtonStyled circular>
							<button
								v-tooltip="formatMessage(messages.clearLinkLabel)"
								class="label-button"
								:aria-label="formatMessage(messages.clearLinkLabel)"
								:data-active="editLinks.source.clear"
								@click="editLinks.source.clear = !editLinks.source.clear"
							>
								<TrashIcon />
							</button>
						</ButtonStyled>
					</div>
					<label for="wiki-page-input" :title="formatMessage(messages.wikiPageDescription)">
						<span class="label__title">{{ formatMessage(messages.wikiPageLabel) }}</span>
					</label>
					<div class="flex gap-2">
						<StyledInput
							id="wiki-page-input"
							v-model="editLinks.wiki.val"
							:disabled="editLinks.wiki.clear"
							type="url"
							class="w-full"
							:maxlength="2048"
							:placeholder="getLinkInputPlaceholder(editLinks.wiki.clear)"
						/>
						<ButtonStyled circular>
							<button
								v-tooltip="formatMessage(messages.clearLinkLabel)"
								class="label-button"
								:aria-label="formatMessage(messages.clearLinkLabel)"
								:data-active="editLinks.wiki.clear"
								@click="editLinks.wiki.clear = !editLinks.wiki.clear"
							>
								<TrashIcon />
							</button>
						</ButtonStyled>
					</div>
					<label
						for="discord-invite-input"
						:title="formatMessage(messages.discordInviteDescription)"
					>
						<span class="label__title">{{ formatMessage(messages.discordInviteLabel) }}</span>
					</label>
					<div class="flex gap-2">
						<StyledInput
							id="discord-invite-input"
							v-model="editLinks.discord.val"
							:disabled="editLinks.discord.clear"
							class="w-full"
							type="url"
							:maxlength="2048"
							:placeholder="getLinkInputPlaceholder(editLinks.discord.clear, true)"
						/>
						<ButtonStyled circular>
							<button
								v-tooltip="formatMessage(messages.clearLinkLabel)"
								class="label-button"
								:aria-label="formatMessage(messages.clearLinkLabel)"
								:data-active="editLinks.discord.clear"
								@click="editLinks.discord.clear = !editLinks.discord.clear"
							>
								<TrashIcon />
							</button>
						</ButtonStyled>
					</div>
				</section>
				<p>
					<IntlFormatted
						:message-id="messages.changesAppliedTo"
						:values="{ count: selectedProjects.length }"
					>
						<template #strong="{ children }">
							<strong><component :is="() => children" /></strong>
						</template>
					</IntlFormatted>
				</p>
				<ul>
					<li
						v-for="project in selectedProjects.slice(
							0,
							editLinks.showAffected ? selectedProjects.length : 3,
						)"
						:key="project.id"
					>
						{{ project.title }}
					</li>
					<li v-if="!editLinks.showAffected && selectedProjects.length > 3">
						<strong>{{
							formatMessage(messages.andMore, { count: selectedProjects.length - 3 })
						}}</strong>
					</li>
				</ul>
				<Checkbox
					v-if="selectedProjects.length > 3"
					v-model="editLinks.showAffected"
					:label="formatMessage(messages.showAllProjects)"
					:description="formatMessage(messages.showAllProjects)"
				/>
				<div class="input-group ml-auto mt-4">
					<ButtonStyled type="outlined">
						<button @click="$refs.editLinksModal.hide()">
							<XIcon />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button @click="bulkEditLinks()">
							<SaveIcon />
							{{ formatMessage(commonMessages.saveChangesButton) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>
		<ModalCreation ref="modal_creation" />
		<section class="relative overflow-hidden rounded-2xl">
			<Table
				v-model:sort-column="sortColumn"
				v-model:sort-direction="sortDirection"
				:columns="projectTableColumns"
				:data="sortedProjects"
				row-key="id"
				table-min-width="50rem"
				table-layout="auto"
			>
				<template #header>
					<div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
						<h2 class="m-0 text-2xl font-semibold text-contrast">
							{{ formatMessage(messages.headTitle) }}
						</h2>
						<div class="flex w-full flex-wrap items-center gap-2 md:w-auto">
							<ButtonStyled>
								<button
									:disabled="selectedProjects.length === 0"
									@click="$refs.editLinksModal.show()"
								>
									<EditIcon />
									{{ formatMessage(messages.editLinksButton) }}
								</button>
							</ButtonStyled>
							<ButtonStyled color="brand">
								<button @click="$refs.modal_creation.show($event)">
									<PlusIcon />
									{{ formatMessage(commonMessages.createAProjectButton) }}
								</button>
							</ButtonStyled>
						</div>
					</div>
				</template>
				<template #empty-state>
					<div class="flex h-64 items-center justify-center text-secondary">
						{{ formatMessage(messages.noProjectsYet) }}
					</div>
				</template>
				<template #header-select>
					<div v-tooltip="formatMessage(messages.selectAllBulkEditableProjects)">
						<Checkbox
							:model-value="allBulkEditableProjectsSelected"
							@update:model-value="toggleAllBulkEditableProjects()"
						/>
					</div>
				</template>
				<template #cell-select="{ row: project }">
					<div>
						<Checkbox
							v-tooltip="getBulkEditDisabledTooltip(project)"
							:disabled="isProjectBulkEditDisabled(project)"
							:model-value="isProjectSelected(project)"
							@update:model-value="toggleProjectSelection(project)"
						/>
					</div>
				</template>
				<template #cell-name="{ row: project }">
					<nuxt-link class="project-name-cell" :to="getProjectUrl(project)">
						<Avatar
							class="shrink-0"
							:src="project.icon_url"
							aria-hidden="true"
							:alt="formatMessage(messages.projectIconAlt, { title: project.title })"
							no-shadow
						/>
						<span class="project-title">
							<IssuesIcon
								v-if="project.moderator_message"
								:aria-label="formatMessage(messages.projectModeratorMessageAriaLabel)"
							/>

							<span class="project-title-link wrap-as-needed">
								{{ project.title }}
							</span>
						</span>
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
				<template #cell-status="{ row: project }">
					<div class="flex items-center">
						<ProjectStatusBadge v-if="project.status" :status="project.status" />
					</div>
				</template>
				<template #cell-actions="{ row: project }">
					<div class="flex !flex-row items-center !justify-end gap-2">
						<ButtonStyled
							v-if="projectsWithMigrationWarning.includes(project.id)"
							circular
							color="orange"
						>
							<nuxt-link
								v-tooltip="formatMessage(messages.reviewEnvironmentMetadata)"
								:to="`${getProjectUrl(project)}?showEnvironmentMigrationWarning=true`"
							>
								<TriangleAlertIcon />
							</nuxt-link>
						</ButtonStyled>
						<ButtonStyled circular>
							<nuxt-link
								v-tooltip="formatMessage(commonMessages.settingsLabel)"
								:to="`${getProjectUrl(project)}/settings`"
							>
								<SettingsIcon />
							</nuxt-link>
						</ButtonStyled>
					</div>
				</template>
			</Table>
		</section>
	</div>
</template>

<script setup>
import {
	EditIcon,
	IssuesIcon,
	PlusIcon,
	SaveIcon,
	SettingsIcon,
	TrashIcon,
	TriangleAlertIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	Checkbox,
	commonMessages,
	CopyCode,
	defineMessages,
	injectNotificationManager,
	IntlFormatted,
	NewModal,
	ProjectStatusBadge,
	StyledInput,
	Table,
	useVIntl,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'

import ModalCreation from '~/components/ui/create/ProjectCreateModal.vue'
import { getProjectTypeForUrl } from '~/helpers/projects.js'

// const UPLOAD_VERSION = 1 << 0
// const DELETE_VERSION = 1 << 1
const EDIT_DETAILS = 1 << 2
// const EDIT_BODY = 1 << 3
// const MANAGE_INVITES = 1 << 4
// const REMOVE_MEMBER = 1 << 5
// const EDIT_MEMBER = 1 << 6
// const DELETE_PROJECT = 1 << 7

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	headTitle: {
		id: 'dashboard.projects.head-title',
		defaultMessage: 'Projects',
	},
	editLinksButton: {
		id: 'dashboard.projects.links.button.edit',
		defaultMessage: 'Edit links',
	},
	editLinksHeader: {
		id: 'dashboard.projects.links.header.edit',
		defaultMessage: 'Edit links',
	},
	editLinksDescription: {
		id: 'dashboard.projects.links.description',
		defaultMessage:
			'Any links you specify below will be overwritten on each of the selected projects. Any you leave blank will be ignored. You can clear a link from all selected projects using the trash can button.',
	},
	issueTrackerLabel: {
		id: 'dashboard.projects.links.issue-tracker.label',
		defaultMessage: 'Issue tracker',
	},
	issueTrackerDescription: {
		id: 'dashboard.projects.links.issue-tracker.description',
		defaultMessage: 'A place for users to report bugs, issues, and concerns about your project.',
	},
	sourceCodeLabel: {
		id: 'dashboard.projects.links.source-code.label',
		defaultMessage: 'Source code',
	},
	sourceCodeDescription: {
		id: 'dashboard.projects.links.source-code.description',
		defaultMessage: 'A page/repository containing the source code for your project',
	},
	wikiPageLabel: {
		id: 'dashboard.projects.links.wiki-page.label',
		defaultMessage: 'Wiki page',
	},
	wikiPageDescription: {
		id: 'dashboard.projects.links.wiki-page.description',
		defaultMessage: 'A page containing information, documentation, and help for the project',
	},
	discordInviteLabel: {
		id: 'dashboard.projects.links.discord-invite.label',
		defaultMessage: 'Discord invite',
	},
	discordInviteDescription: {
		id: 'dashboard.projects.links.discord-invite.description',
		defaultMessage: 'An invitation link to your Discord server.',
	},
	existingLinkWillBeCleared: {
		id: 'dashboard.projects.links.placeholder.cleared',
		defaultMessage: 'Existing link will be cleared',
	},
	enterValidUrl: {
		id: 'dashboard.projects.links.placeholder.valid-url',
		defaultMessage: 'Enter a valid URL',
	},
	enterValidDiscordInviteUrl: {
		id: 'dashboard.projects.links.placeholder.valid-discord-url',
		defaultMessage: 'Enter a valid Discord invite URL',
	},
	clearLinkLabel: {
		id: 'dashboard.projects.links.button.clear-link',
		defaultMessage: 'Clear link',
	},
	changesAppliedTo: {
		id: 'dashboard.projects.links.changes-applied',
		defaultMessage:
			'Changes will be applied to <strong>{count}</strong> {count, plural, one {project} other {projects}}.',
	},
	andMore: {
		id: 'dashboard.projects.links.and-more',
		defaultMessage: 'and {count} more...',
	},
	showAllProjects: {
		id: 'dashboard.projects.links.show-all-projects',
		defaultMessage: 'Show all projects',
	},
	noProjectsYet: {
		id: 'dashboard.projects.empty',
		defaultMessage: "You don't have any projects yet. Click the green button above to begin.",
	},
	nameHeader: {
		id: 'dashboard.projects.table.name',
		defaultMessage: 'Name',
	},
	idHeader: {
		id: 'dashboard.projects.table.id',
		defaultMessage: 'ID',
	},
	typeHeader: {
		id: 'dashboard.projects.table.type',
		defaultMessage: 'Type',
	},
	statusHeader: {
		id: 'dashboard.projects.table.status',
		defaultMessage: 'Status',
	},
	selectAllBulkEditableProjects: {
		id: 'dashboard.projects.table.select-all-bulk-editable',
		defaultMessage: 'Select all projects that support bulk editing',
	},
	projectIconAlt: {
		id: 'dashboard.projects.project.icon-alt',
		defaultMessage: 'Icon for {title}',
	},
	projectModeratorMessageAriaLabel: {
		id: 'dashboard.projects.project.moderator-message-aria',
		defaultMessage: 'Project has a message from the moderators. View the project to see more.',
	},
	reviewEnvironmentMetadata: {
		id: 'dashboard.projects.project.review-environment-metadata',
		defaultMessage: 'Please review environment metadata',
	},
	serverBulkEditDisabled: {
		id: 'dashboard.projects.bulk-edit.server-disabled',
		defaultMessage: 'Server projects do not support bulk editing',
	},
	bulkEditSuccessText: {
		id: 'dashboard.projects.notification.bulk-edit-success',
		defaultMessage: "Bulk edited selected project's links.",
	},
})

useHead({ title: () => `${formatMessage(messages.headTitle)} - Modrinth` })

const user = await useUser()
const projects = ref([])
const projectsWithMigrationWarning = ref([])
const selectedProjectIds = ref([])
const sortColumn = ref('name')
const sortDirection = ref('asc')
const editLinks = reactive({
	showAffected: false,
	source: { val: '', clear: false },
	discord: { val: '', clear: false },
	wiki: { val: '', clear: false },
	issues: { val: '', clear: false },
})

const editLinksModal = ref(null)
const sortCollator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' })

const projectTableColumns = computed(() => [
	{
		key: 'select',
		width: '3rem',
		headerClass: '!text-center',
		cellClass: '!overflow-visible',
	},
	{
		key: 'name',
		label: formatMessage(messages.nameHeader),
		enableSorting: true,
		defaultSortDirection: 'asc',
		width: '22rem',
	},
	{
		key: 'id',
		label: formatMessage(messages.idHeader),
		width: '13rem',
		cellClass: '!overflow-visible',
	},
	{
		key: 'type',
		label: formatMessage(messages.typeHeader),
		enableSorting: true,
		width: '11rem',
	},
	{
		key: 'status',
		label: formatMessage(messages.statusHeader),
		enableSorting: true,
		width: '10rem',
		cellClass: '!overflow-visible',
	},
	{
		key: 'actions',
		width: '6rem',
		align: 'right',
		cellClass: '!overflow-visible',
	},
])

const sortedProjects = computed(() => {
	const direction = sortDirection.value === 'desc' ? -1 : 1

	return projects.value.slice().sort((left, right) => {
		const result = sortCollator.compare(
			getProjectSortValue(left, sortColumn.value),
			getProjectSortValue(right, sortColumn.value),
		)

		return result * direction
	})
})

function getLinkInputPlaceholder(clearLink, isDiscord = false) {
	if (clearLink) {
		return formatMessage(messages.existingLinkWillBeCleared)
	}

	return isDiscord
		? formatMessage(messages.enterValidDiscordInviteUrl)
		: formatMessage(messages.enterValidUrl)
}

function isProjectBulkEditDisabled(project) {
	return (
		(project.permissions & EDIT_DETAILS) === EDIT_DETAILS ||
		project.project_type === 'minecraft_java_server'
	)
}

const bulkEditableProjects = computed(() =>
	projects.value.filter((project) => !isProjectBulkEditDisabled(project)),
)

const selectedProjects = computed(() =>
	projects.value.filter((project) => selectedProjectIds.value.includes(project.id)),
)

const allBulkEditableProjectsSelected = computed(
	() =>
		bulkEditableProjects.value.length > 0 &&
		bulkEditableProjects.value.every((project) => selectedProjectIds.value.includes(project.id)),
)

function toggleAllBulkEditableProjects() {
	selectedProjectIds.value = allBulkEditableProjectsSelected.value
		? []
		: bulkEditableProjects.value.map((project) => project.id)
}

function isProjectSelected(project) {
	return selectedProjectIds.value.includes(project.id)
}

function toggleProjectSelection(project) {
	if (isProjectBulkEditDisabled(project)) {
		return
	}

	if (isProjectSelected(project)) {
		selectedProjectIds.value = selectedProjectIds.value.filter((id) => id !== project.id)
		return
	}

	selectedProjectIds.value = [...selectedProjectIds.value, project.id]
}

function getBulkEditDisabledTooltip(project) {
	if (project.project_type === 'minecraft_java_server') {
		return formatMessage(messages.serverBulkEditDisabled)
	}

	return ''
}

function getProjectUrlType(project) {
	return getProjectTypeForUrl(project.project_type, project.loaders)
}

function getProjectDisplayType(project) {
	return formatProjectType(getProjectUrlType(project))
}

function getProjectUrl(project) {
	return `/${getProjectUrlType(project)}/${project.slug ? project.slug : project.id}`
}

function getProjectSortValue(project, column) {
	switch (column) {
		case 'type':
			return getProjectDisplayType(project)
		case 'status':
			return project.status ?? ''
		case 'name':
		default:
			return project.title ?? ''
	}
}

async function bulkEditLinks() {
	try {
		const baseData = {
			issues_url: editLinks.issues.clear ? null : editLinks.issues.val.trim(),
			source_url: editLinks.source.clear ? null : editLinks.source.val.trim(),
			wiki_url: editLinks.wiki.clear ? null : editLinks.wiki.val.trim(),
			discord_url: editLinks.discord.clear ? null : editLinks.discord.val.trim(),
		}
		const filteredData = Object.fromEntries(Object.entries(baseData).filter(([, v]) => v !== ''))

		await useBaseFetch(`projects?ids=${JSON.stringify(selectedProjects.value.map((x) => x.id))}`, {
			method: 'PATCH',
			body: filteredData,
		})

		editLinksModal.value?.hide()
		addNotification({
			title: formatMessage(commonMessages.successLabel),
			text: formatMessage(messages.bulkEditSuccessText),
			type: 'success',
		})
		selectedProjectIds.value = []

		editLinks.issues.val = ''
		editLinks.source.val = ''
		editLinks.wiki.val = ''
		editLinks.discord.val = ''
		editLinks.issues.clear = false
		editLinks.source.clear = false
		editLinks.wiki.clear = false
		editLinks.discord.clear = false
	} catch (e) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: e,
			type: 'error',
		})
	}
}

await initUserProjects()
if (user.value?.projects) {
	projects.value = user.value.projects.slice()

	// minecraft_java_server type determined from component on projectV3
	projects.value = projects.value.map((project) => {
		const projectV3 = user.value?.projectsV3?.find((p) => p.id === project.id)
		if (projectV3?.minecraft_server != null)
			return { ...project, project_type: 'minecraft_java_server' }
		return project
	})
	user.value?.projectsV3?.forEach((project) => {
		if (
			project.side_types_migration_review_status === 'pending' &&
			(project.project_types.includes('mod') || project.project_types.includes('modpack')) &&
			project.environment?.length === 1
		) {
			projectsWithMigrationWarning.value.push(project.id)
		}
	})
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

.project-title {
	display: flex;
	flex-direction: row;
	min-width: 0;
	gap: var(--spacing-card-xs);

	svg {
		color: var(--color-orange);
	}
}

.project-name-cell:hover .project-title-link,
.project-name-cell:focus-visible .project-title-link {
	text-decoration: underline;
}

.label-button[data-active='true'] {
	--background-color: var(--color-red);
	--text-color: var(--color-brand-inverted);
}

.links-modal {
	.links {
		display: grid;
		gap: var(--spacing-card-sm);
		grid-template-columns: 1fr 2fr;

		.input-group {
			flex-wrap: nowrap;
		}

		@media screen and (max-width: 530px) {
			grid-template-columns: 1fr;
			.input-group {
				flex-wrap: wrap;
			}
		}
	}

	ul {
		margin: 0 0 var(--spacing-card-sm) 0;
	}
}
</style>
