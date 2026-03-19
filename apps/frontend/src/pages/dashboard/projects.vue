<template>
	<div>
		<NewModal ref="editLinksModal" :header="formatMessage(messages.editLinksButton)">
			<div class="universal-modal links-modal !p-0">
				<p>{{ formatMessage(messages.editLinksDescription) }}</p>
				<section class="links">
					<label for="issue-tracker-input" :title="formatMessage(messages.issueTrackerDescription)">
						<span class="label__title">{{ formatMessage(messages.issueTrackerLabel) }}</span>
					</label>
					<div class="input-group shrink-first">
						<StyledInput
							id="issue-tracker-input"
							v-model="editLinks.issues.val"
							:disabled="editLinks.issues.clear"
							type="url"
							:placeholder="getLinkInputPlaceholder(editLinks.issues.clear)"
							:maxlength="2048"
						/>
						<button
							v-tooltip="formatMessage(messages.clearLinkLabel)"
							:aria-label="formatMessage(messages.clearLinkLabel)"
							class="square-button label-button"
							:data-active="editLinks.issues.clear"
							@click="editLinks.issues.clear = !editLinks.issues.clear"
						>
							<TrashIcon />
						</button>
					</div>
					<label for="source-code-input" :title="formatMessage(messages.sourceCodeDescription)">
						<span class="label__title">{{ formatMessage(messages.sourceCodeLabel) }}</span>
					</label>
					<div class="input-group shrink-first">
						<StyledInput
							id="source-code-input"
							v-model="editLinks.source.val"
							:disabled="editLinks.source.clear"
							type="url"
							:maxlength="2048"
							:placeholder="getLinkInputPlaceholder(editLinks.source.clear)"
						/>
						<button
							v-tooltip="formatMessage(messages.clearLinkLabel)"
							:aria-label="formatMessage(messages.clearLinkLabel)"
							class="square-button label-button"
							:data-active="editLinks.source.clear"
							@click="editLinks.source.clear = !editLinks.source.clear"
						>
							<TrashIcon />
						</button>
					</div>
					<label for="wiki-page-input" :title="formatMessage(messages.wikiPageDescription)">
						<span class="label__title">{{ formatMessage(messages.wikiPageLabel) }}</span>
					</label>
					<div class="input-group shrink-first">
						<StyledInput
							id="wiki-page-input"
							v-model="editLinks.wiki.val"
							:disabled="editLinks.wiki.clear"
							type="url"
							:maxlength="2048"
							:placeholder="getLinkInputPlaceholder(editLinks.wiki.clear)"
						/>
						<button
							v-tooltip="formatMessage(messages.clearLinkLabel)"
							:aria-label="formatMessage(messages.clearLinkLabel)"
							class="square-button label-button"
							:data-active="editLinks.wiki.clear"
							@click="editLinks.wiki.clear = !editLinks.wiki.clear"
						>
							<TrashIcon />
						</button>
					</div>
					<label
						for="discord-invite-input"
						:title="formatMessage(messages.discordInviteDescription)"
					>
						<span class="label__title">{{ formatMessage(messages.discordInviteLabel) }}</span>
					</label>
					<div class="input-group shrink-first">
						<StyledInput
							id="discord-invite-input"
							v-model="editLinks.discord.val"
							:disabled="editLinks.discord.clear"
							type="url"
							:maxlength="2048"
							:placeholder="getLinkInputPlaceholder(editLinks.discord.clear, true)"
						/>
						<button
							v-tooltip="formatMessage(messages.clearLinkLabel)"
							:aria-label="formatMessage(messages.clearLinkLabel)"
							class="square-button label-button"
							:data-active="editLinks.discord.clear"
							@click="editLinks.discord.clear = !editLinks.discord.clear"
						>
							<TrashIcon />
						</button>
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
				<div class="push-right input-group">
					<button class="iconified-button" @click="$refs.editLinksModal.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
					<button class="iconified-button brand-button" @click="bulkEditLinks()">
						<SaveIcon />
						{{ formatMessage(commonMessages.saveChangesButton) }}
					</button>
				</div>
			</div>
		</NewModal>
		<ModalCreation ref="modal_creation" />
		<section class="universal-card">
			<div class="header__row">
				<h2 class="header__title text-2xl">{{ formatMessage(messages.headTitle) }}</h2>
				<div class="input-group">
					<button class="iconified-button brand-button" @click="$refs.modal_creation.show($event)">
						<PlusIcon />
						{{ formatMessage(commonMessages.createAProjectButton) }}
					</button>
				</div>
			</div>
			<p v-if="projects.length < 1">
				{{ formatMessage(messages.noProjectsYet) }}
			</p>
			<template v-else>
				<p>{{ formatMessage(messages.bulkEditHint) }}</p>
				<div class="input-group">
					<button
						class="iconified-button"
						:disabled="selectedProjects.length === 0"
						@click="$refs.editLinksModal.show()"
					>
						<EditIcon />
						{{ formatMessage(messages.editLinksButton) }}
					</button>
					<div class="push-right">
						<div class="labeled-control-row">
							{{ formatMessage(messages.sortByLabel) }}
							<Combobox
								v-model="sortBy"
								:searchable="false"
								class="small-select"
								:options="sortOptions"
								@update:model-value="projects = updateSort(projects, sortBy, descending)"
							/>
							<button
								v-tooltip="formatMessage(descending ? messages.descending : messages.ascending)"
								class="square-button"
								@click="updateDescending()"
							>
								<SortDescIcon v-if="descending" />
								<SortAscIcon v-else />
							</button>
						</div>
					</div>
				</div>
				<div class="grid-table">
					<div class="grid-table__row grid-table__header">
						<div>
							<Checkbox
								:model-value="allBulkEditableProjectsSelected"
								@update:model-value="toggleAllBulkEditableProjects()"
							/>
						</div>
						<div>{{ formatMessage(messages.iconHeader) }}</div>
						<div>{{ formatMessage(messages.nameHeader) }}</div>
						<div>{{ formatMessage(messages.idHeader) }}</div>
						<div>{{ formatMessage(messages.typeHeader) }}</div>
						<div>{{ formatMessage(messages.statusHeader) }}</div>
						<div />
					</div>
					<div v-for="project in projects" :key="`project-${project.id}`" class="grid-table__row">
						<div>
							<Checkbox
								v-tooltip="getBulkEditDisabledTooltip(project)"
								:disabled="isProjectBulkEditDisabled(project)"
								:model-value="selectedProjects.includes(project)"
								@update:model-value="toggleProjectSelection(project)"
							/>
						</div>
						<div>
							<nuxt-link
								tabindex="-1"
								:to="`/${getProjectTypeForUrl(project.project_type, project.loaders)}/${
									project.slug ? project.slug : project.id
								}`"
							>
								<Avatar
									:src="project.icon_url"
									aria-hidden="true"
									:alt="formatMessage(messages.projectIconAlt, { title: project.title })"
									no-shadow
								/>
							</nuxt-link>
						</div>

						<div>
							<span class="project-title">
								<IssuesIcon
									v-if="project.moderator_message"
									:aria-label="formatMessage(messages.projectModeratorMessageAriaLabel)"
								/>

								<nuxt-link
									class="hover-link wrap-as-needed"
									:to="`/${getProjectTypeForUrl(project.project_type, project.loaders)}/${
										project.slug ? project.slug : project.id
									}`"
								>
									{{ project.title }}
								</nuxt-link>
							</span>
						</div>

						<div>
							<CopyCode :text="project.id" />
						</div>

						<div>
							{{ formatProjectTypeLabel(project) }}
						</div>

						<div>
							<ProjectStatusBadge v-if="project.status" :status="project.status" />
						</div>

						<div class="flex !flex-row items-center !justify-end gap-2">
							<ButtonStyled
								v-if="projectsWithMigrationWarning.includes(project.id)"
								circular
								color="orange"
							>
								<nuxt-link
									v-tooltip="formatMessage(messages.reviewEnvironmentMetadata)"
									:to="`/${getProjectTypeForUrl(project.project_type, project.loaders)}/${
										project.slug ? project.slug : project.id
									}?showEnvironmentMigrationWarning=true`"
								>
									<TriangleAlertIcon />
								</nuxt-link>
							</ButtonStyled>
							<ButtonStyled circular>
								<nuxt-link
									v-tooltip="formatMessage(commonMessages.settingsLabel)"
									:to="`/${getProjectTypeForUrl(project.project_type, project.loaders)}/${
										project.slug ? project.slug : project.id
									}/settings`"
								>
									<SettingsIcon />
								</nuxt-link>
							</ButtonStyled>
						</div>
					</div>
				</div>
			</template>
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
	SortAscIcon,
	SortDescIcon,
	TrashIcon,
	TriangleAlertIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	Checkbox,
	Combobox,
	commonMessages,
	commonProjectTypeTitleMessages,
	CopyCode,
	defineMessages,
	injectNotificationManager,
	IntlFormatted,
	NewModal,
	ProjectStatusBadge,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'

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
		defaultMessage: 'A page containing information, documentation, and help for the project.',
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
	bulkEditHint: {
		id: 'dashboard.projects.bulk-edit-hint',
		defaultMessage: 'You can edit multiple projects at once by selecting them below.',
	},
	sortByLabel: {
		id: 'dashboard.projects.sort.label',
		defaultMessage: 'Sort by',
	},
	ascending: {
		id: 'dashboard.projects.sort.ascending',
		defaultMessage: 'Ascending',
	},
	descending: {
		id: 'dashboard.projects.sort.descending',
		defaultMessage: 'Descending',
	},
	sortOptionName: {
		id: 'dashboard.projects.sort.option.name',
		defaultMessage: 'Name',
	},
	sortOptionStatus: {
		id: 'dashboard.projects.sort.option.status',
		defaultMessage: 'Status',
	},
	sortOptionType: {
		id: 'dashboard.projects.sort.option.type',
		defaultMessage: 'Type',
	},
	iconHeader: {
		id: 'dashboard.projects.table.icon',
		defaultMessage: 'Icon',
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
const selectedProjects = ref([])
const sortBy = ref('name')
const sortOptions = computed(() => [
	{ value: 'name', label: formatMessage(messages.sortOptionName) },
	{ value: 'status', label: formatMessage(messages.sortOptionStatus) },
	{ value: 'type', label: formatMessage(messages.sortOptionType) },
])
const descending = ref(false)
const editLinks = reactive({
	showAffected: false,
	source: { val: '', clear: false },
	discord: { val: '', clear: false },
	wiki: { val: '', clear: false },
	issues: { val: '', clear: false },
})

const editLinksModal = ref(null)
const modal_creation = ref(null)

function getLinkInputPlaceholder(clearLink, isDiscord = false) {
	if (clearLink) {
		return formatMessage(messages.existingLinkWillBeCleared)
	}

	return isDiscord
		? formatMessage(messages.enterValidDiscordInviteUrl)
		: formatMessage(messages.enterValidUrl)
}

function getProjectTypeTitleMessage(type) {
	return commonProjectTypeTitleMessages[type] ?? commonProjectTypeTitleMessages.project
}

function formatProjectTypeLabel(project) {
	return formatMessage(
		getProjectTypeTitleMessage(getProjectTypeForUrl(project.project_type, project.loaders)),
		{ count: 1 },
	)
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

const allBulkEditableProjectsSelected = computed(
	() =>
		bulkEditableProjects.value.length > 0 &&
		bulkEditableProjects.value.every((project) => selectedProjects.value.includes(project)),
)

function toggleAllBulkEditableProjects() {
	selectedProjects.value = allBulkEditableProjectsSelected.value
		? []
		: bulkEditableProjects.value.slice()
}

function toggleProjectSelection(project) {
	if (isProjectBulkEditDisabled(project)) {
		return
	}

	if (selectedProjects.value.includes(project)) {
		selectedProjects.value = selectedProjects.value.filter((it) => it !== project)
		return
	}

	selectedProjects.value = [...selectedProjects.value, project]
}

function getBulkEditDisabledTooltip(project) {
	if (project.project_type === 'minecraft_java_server') {
		return formatMessage(messages.serverBulkEditDisabled)
	}

	return ''
}

function updateSort(list, sort, desc) {
	let sortedArray = list
	switch (sort) {
		case 'name':
			sortedArray = list.slice().sort((a, b) => a.title.localeCompare(b.title))
			break
		case 'status':
			sortedArray = list.slice().sort((a, b) => {
				if (a.status < b.status) return -1
				if (a.status > b.status) return 1
				return 0
			})
			break
		case 'type':
			sortedArray = list.slice().sort((a, b) => {
				if (a.project_type < b.project_type) return -1
				if (a.project_type > b.project_type) return 1
				return 0
			})
			break
		default:
			break
	}
	if (desc) sortedArray = sortedArray.reverse()
	return sortedArray
}

function resort() {
	projects.value = updateSort(projects.value, sortBy.value, descending.value)
}

function updateDescending() {
	descending.value = !descending.value
	resort()
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
		selectedProjects.value = []

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
	projects.value = updateSort(user.value.projects, 'name', false)

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
.grid-table {
	display: grid;
	grid-template-columns:
		min-content min-content minmax(min-content, 2fr)
		minmax(min-content, 1fr) minmax(min-content, 1fr) minmax(min-content, 1fr) min-content;
	border-radius: var(--size-rounded-sm);
	overflow: hidden;
	margin-top: var(--spacing-card-md);
	outline: 1px solid transparent;

	.grid-table__row {
		display: contents;

		> div {
			display: flex;
			flex-direction: column;
			justify-content: center;
			padding: var(--spacing-card-sm);

			// Left edge of table
			&:first-child {
				padding-left: var(--spacing-card-bg);
			}

			// Right edge of table
			&:last-child {
				padding-right: var(--spacing-card-bg);
			}
		}

		&:nth-child(2n + 1) > div {
			background-color: var(--color-table-alternate-row);
		}

		&.grid-table__header > div {
			background-color: var(--color-bg);
			font-weight: bold;
			color: var(--color-text-dark);
			padding-top: var(--spacing-card-bg);
			padding-bottom: var(--spacing-card-bg);
		}
	}

	@media screen and (max-width: 750px) {
		display: flex;
		flex-direction: column;

		.grid-table__row {
			display: grid;
			grid-template: 'checkbox icon name type settings' 'checkbox icon id status settings';
			grid-template-columns:
				min-content min-content minmax(min-content, 2fr)
				minmax(min-content, 1fr) min-content;

			:nth-child(1) {
				grid-area: checkbox;
			}

			:nth-child(2) {
				grid-area: icon;
			}

			:nth-child(3) {
				grid-area: name;
			}

			:nth-child(4) {
				grid-area: id;
				padding-top: 0;
			}

			:nth-child(5) {
				grid-area: type;
			}

			:nth-child(6) {
				grid-area: status;
				padding-top: 0;
			}

			:nth-child(7) {
				grid-area: settings;
			}
		}

		.grid-table__header {
			grid-template: 'checkbox settings';
			grid-template-columns: min-content minmax(min-content, 1fr);

			:nth-child(2),
			:nth-child(3),
			:nth-child(4),
			:nth-child(5),
			:nth-child(6) {
				display: none;
			}
		}
	}

	@media screen and (max-width: 560px) {
		.grid-table__row {
			display: grid;
			grid-template: 'checkbox icon name settings' 'checkbox icon id settings' 'checkbox icon type settings' 'checkbox icon status settings';
			grid-template-columns: min-content min-content minmax(min-content, 1fr) min-content;

			:nth-child(5) {
				padding-top: 0;
			}
		}

		.grid-table__header {
			grid-template: 'checkbox settings';
			grid-template-columns: min-content minmax(min-content, 1fr);
		}
	}
}

.project-title {
	display: flex;
	flex-direction: row;
	gap: var(--spacing-card-xs);

	svg {
		color: var(--color-orange);
	}
}

.status {
	margin-top: var(--spacing-card-xs);
}

.hover-link:hover {
	text-decoration: underline;
}

.labeled-control-row {
	flex: 1;
	display: flex;
	flex-direction: row;
	min-width: 0;
	align-items: center;
	gap: var(--spacing-card-md);
	white-space: nowrap;
}

.small-select {
	width: -moz-fit-content;
	width: fit-content;
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
