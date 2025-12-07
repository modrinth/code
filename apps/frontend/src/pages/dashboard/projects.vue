<template>
	<div>
		<NewModal ref="editLinksModal" header="Edit links">
			<div class="universal-modal links-modal !p-0">
				<p>
					Any links you specify below will be overwritten on each of the selected projects. Any you
					leave blank will be ignored. You can clear a link from all selected projects using the
					trash can button.
				</p>
				<section class="links">
					<label
						for="issue-tracker-input"
						title="A place for users to report bugs, issues, and concerns about your project."
					>
						<span class="label__title">Issue tracker</span>
					</label>
					<div class="input-group shrink-first">
						<input
							id="issue-tracker-input"
							v-model="editLinks.issues.val"
							:disabled="editLinks.issues.clear"
							type="url"
							:placeholder="
								editLinks.issues.clear ? 'Existing link will be cleared' : 'Enter a valid URL'
							"
							maxlength="2048"
						/>
						<button
							v-tooltip="'Clear link'"
							aria-label="Clear link"
							class="square-button label-button"
							:data-active="editLinks.issues.clear"
							@click="editLinks.issues.clear = !editLinks.issues.clear"
						>
							<TrashIcon />
						</button>
					</div>
					<label
						for="source-code-input"
						title="A page/repository containing the source code for your project"
					>
						<span class="label__title">Source code</span>
					</label>
					<div class="input-group shrink-first">
						<input
							id="source-code-input"
							v-model="editLinks.source.val"
							:disabled="editLinks.source.clear"
							type="url"
							maxlength="2048"
							:placeholder="
								editLinks.source.clear ? 'Existing link will be cleared' : 'Enter a valid URL'
							"
						/>
						<button
							v-tooltip="'Clear link'"
							aria-label="Clear link"
							class="square-button label-button"
							:data-active="editLinks.source.clear"
							@click="editLinks.source.clear = !editLinks.source.clear"
						>
							<TrashIcon />
						</button>
					</div>
					<label
						for="wiki-page-input"
						title="A page containing information, documentation, and help for the project."
					>
						<span class="label__title">Wiki page</span>
					</label>
					<div class="input-group shrink-first">
						<input
							id="wiki-page-input"
							v-model="editLinks.wiki.val"
							:disabled="editLinks.wiki.clear"
							type="url"
							maxlength="2048"
							:placeholder="
								editLinks.wiki.clear ? 'Existing link will be cleared' : 'Enter a valid URL'
							"
						/>
						<button
							v-tooltip="'Clear link'"
							aria-label="Clear link"
							class="square-button label-button"
							:data-active="editLinks.wiki.clear"
							@click="editLinks.wiki.clear = !editLinks.wiki.clear"
						>
							<TrashIcon />
						</button>
					</div>
					<label for="discord-invite-input" title="An invitation link to your Discord server.">
						<span class="label__title">Discord invite</span>
					</label>
					<div class="input-group shrink-first">
						<input
							id="discord-invite-input"
							v-model="editLinks.discord.val"
							:disabled="editLinks.discord.clear"
							type="url"
							maxlength="2048"
							:placeholder="
								editLinks.discord.clear
									? 'Existing link will be cleared'
									: 'Enter a valid Discord invite URL'
							"
						/>
						<button
							v-tooltip="'Clear link'"
							aria-label="Clear link"
							class="square-button label-button"
							:data-active="editLinks.discord.clear"
							@click="editLinks.discord.clear = !editLinks.discord.clear"
						>
							<TrashIcon />
						</button>
					</div>
				</section>
				<p>
					Changes will be applied to
					<strong>{{ selectedProjects.length }}</strong> project{{
						selectedProjects.length > 1 ? 's' : ''
					}}.
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
						<strong>and {{ selectedProjects.length - 3 }} more...</strong>
					</li>
				</ul>
				<Checkbox
					v-if="selectedProjects.length > 3"
					v-model="editLinks.showAffected"
					label="Show all projects"
					description="Show all projects"
				/>
				<div class="push-right input-group">
					<button class="iconified-button" @click="$refs.editLinksModal.hide()">
						<XIcon />
						Cancel
					</button>
					<button class="iconified-button brand-button" @click="bulkEditLinks()">
						<SaveIcon />
						Save changes
					</button>
				</div>
			</div>
		</NewModal>
		<ModalCreation ref="modal_creation" />
		<section class="universal-card">
			<div class="header__row">
				<h2 class="header__title text-2xl">Projects</h2>
				<div class="input-group">
					<button class="iconified-button brand-button" @click="$refs.modal_creation.show()">
						<PlusIcon />
						{{ formatMessage(commonMessages.createAProjectButton) }}
					</button>
				</div>
			</div>
			<p v-if="projects.length < 1">
				You don't have any projects yet. Click the green button above to begin.
			</p>
			<template v-else>
				<p>You can edit multiple projects at once by selecting them below.</p>
				<div class="input-group">
					<button
						class="iconified-button"
						:disabled="selectedProjects.length === 0"
						@click="$refs.editLinksModal.show()"
					>
						<EditIcon />
						Edit links
					</button>
					<div class="push-right">
						<div class="labeled-control-row">
							Sort by
							<Multiselect
								v-model="sortBy"
								:searchable="false"
								class="small-select"
								:options="['Name', 'Status', 'Type']"
								:close-on-select="true"
								:show-labels="false"
								:allow-empty="false"
								@update:model-value="projects = updateSort(projects, sortBy, descending)"
							/>
							<button
								v-tooltip="descending ? 'Descending' : 'Ascending'"
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
								:model-value="selectedProjects === projects"
								@update:model-value="
									selectedProjects === projects
										? (selectedProjects = [])
										: (selectedProjects = projects)
								"
							/>
						</div>
						<div>Icon</div>
						<div>Name</div>
						<div>ID</div>
						<div>Type</div>
						<div>Status</div>
						<div />
					</div>
					<div v-for="project in projects" :key="`project-${project.id}`" class="grid-table__row">
						<div>
							<Checkbox
								:disabled="(project.permissions & EDIT_DETAILS) === EDIT_DETAILS"
								:model-value="selectedProjects.includes(project)"
								@update:model-value="
									selectedProjects.includes(project)
										? (selectedProjects = selectedProjects.filter((it) => it !== project))
										: selectedProjects.push(project)
								"
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
									:alt="'Icon for ' + project.title"
									no-shadow
								/>
							</nuxt-link>
						</div>

						<div>
							<span class="project-title">
								<IssuesIcon
									v-if="project.moderator_message"
									aria-label="Project has a message from the moderators. View the project to see more."
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
							{{ formatProjectType(getProjectTypeForUrl(project.project_type, project.loaders)) }}
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
									v-tooltip="'Please review environment metadata'"
									:to="`/${getProjectTypeForUrl(project.project_type, project.loaders)}/${
										project.slug ? project.slug : project.id
									}/settings/environment`"
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
	commonMessages,
	CopyCode,
	injectNotificationManager,
	NewModal,
	ProjectStatusBadge,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import { Multiselect } from 'vue-multiselect'

import ModalCreation from '~/components/ui/create/ProjectCreateModal.vue'
import { getProjectTypeForUrl } from '~/helpers/projects.js'

useHead({ title: 'Projects - Modrinth' })

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

const user = await useUser()
const projects = ref([])
const projectsWithMigrationWarning = ref([])
const selectedProjects = ref([])
const sortBy = ref('Name')
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

function updateSort(list, sort, desc) {
	let sortedArray = list
	switch (sort) {
		case 'Name':
			sortedArray = list.slice().sort((a, b) => a.title.localeCompare(b.title))
			break
		case 'Status':
			sortedArray = list.slice().sort((a, b) => {
				if (a.status < b.status) return -1
				if (a.status > b.status) return 1
				return 0
			})
			break
		case 'Type':
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
			title: 'Success',
			text: "Bulk edited selected project's links.",
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
			title: 'An error occurred',
			text: e,
			type: 'error',
		})
	}
}

await initUserProjects()
if (user.value?.projects) {
	projects.value = updateSort(user.value.projects, 'Name', false)
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
