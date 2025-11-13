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
						{{ project.name }}
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
					<button class="iconified-button brand-button" @click="onBulkEditLinks">
						<SaveIcon />
						Save changes
					</button>
				</div>
			</div>
		</NewModal>
		<ModalCreation ref="modal_creation" :organization-id="organization.id" />
		<section class="universal-card">
			<div class="header__row">
				<h2 class="header__title text-2xl">Projects</h2>
				<div class="input-group">
					<button class="iconified-button brand-button" @click="$refs.modal_creation.show()">
						<PlusIcon />
						{{ formatMessage(commonMessages.createAProjectButton) }}
					</button>
					<OrganizationProjectTransferModal
						:projects="usersOwnedProjects || []"
						@submit="onProjectTransferSubmit"
					/>
				</div>
			</div>
			<p v-if="sortedProjects.length < 1">
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
								@update:model-value="
									sortedProjects = updateSort(sortedProjects, sortBy, descending)
								"
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
								:model-value="selectedProjects === sortedProjects"
								@update:model-value="
									selectedProjects === sortedProjects
										? (selectedProjects = [])
										: (selectedProjects = sortedProjects)
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
					<div
						v-for="project in sortedProjects"
						:key="`project-${project.id}`"
						class="grid-table__row"
					>
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
								:to="`/${getProjectTypeForUrl(project.project_types[0] ?? 'project', project.loaders)}/${
									project.slug ? project.slug : project.id
								}`"
							>
								<Avatar
									:src="project.icon_url"
									aria-hidden="true"
									:alt="'Icon for ' + project.name"
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
									:to="`/${getProjectTypeForUrl(project.project_types[0] ?? 'project', project.loaders)}/${
										project.slug ? project.slug : project.id
									}`"
								>
									{{ project.name }}
								</nuxt-link>
							</span>
						</div>

						<div>
							<CopyCode :text="project.id" />
						</div>

						<div>
							{{
								formatProjectType(
									getProjectTypeForUrl(project.project_types[0] ?? 'project', project.loaders),
								)
							}}
						</div>

						<div>
							<ProjectStatusBadge v-if="project.status" :status="project.status" />
						</div>

						<div class="flex !flex-row items-center !justify-end gap-2">
							<ButtonStyled circular>
								<nuxt-link
									v-tooltip="formatMessage(commonMessages.settingsLabel)"
									:to="`/${getProjectTypeForUrl(project.project_types[0] ?? 'project', project.loaders)}/${
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
import OrganizationProjectTransferModal from '~/components/ui/OrganizationProjectTransferModal.vue'
import { getProjectTypeForUrl } from '~/helpers/projects.js'
import { injectOrganizationContext } from '~/providers/organization-context.ts'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const { organization, projects, refresh } = injectOrganizationContext()

const auth = await useAuth()

const { data: userProjects, refresh: refreshUserProjects } = await useAsyncData(
	`user/${auth.value.user.id}/projects`,
	() => useBaseFetch(`user/${auth.value.user.id}/projects`),
	{
		watch: [auth],
	},
)

const usersOwnedProjects = ref([])

watch(
	() => userProjects.value,
	async () => {
		if (!userProjects.value) return
		if (!userProjects.value.length) return

		const projects = userProjects.value.filter((project) => project.organization === null)

		const teamIds = projects.map((project) => project?.team).filter((x) => x)
		// Shape of teams is member[][]
		const teams = await useBaseFetch(`teams?ids=${JSON.stringify(teamIds)}`, {
			apiVersion: 3,
		})
		// for each team id, figure out if the user is a member, and is_owner. Then filter the projects to only include those that are owned by the user
		const ownedTeamIds = teamIds.filter((_tid, i) => {
			const team = teams?.[i]
			if (!team) return false
			const member = team.find((member) => member.user.id === auth.value.user.id)
			return member && member.is_owner
		})
		const ownedProjects = projects.filter((project) => ownedTeamIds.includes(project.team))
		usersOwnedProjects.value = ownedProjects
	}, // watch options
	{ immediate: true, deep: true },
)

const onProjectTransferSubmit = async (projects) => {
	try {
		for (const project of projects) {
			await useBaseFetch(`organization/${organization.value.id}/projects`, {
				method: 'POST',
				body: JSON.stringify({
					project_id: project.id,
				}),
				apiVersion: 3,
			})
		}

		await refresh()
		await refreshUserProjects()

		addNotification({
			title: 'Success',
			text: 'Transferred selected projects to organization.',
			type: 'success',
		})
	} catch (err) {
		addNotification({
			title: 'An error occurred',
			text: err?.data?.description || err?.message || err || 'Unknown error',
			type: 'error',
		})
		console.error(err)
	}
}

const EDIT_DETAILS = 1 << 2

const updateSort = (inputProjects, sort, descending) => {
	let sortedArray = inputProjects
	switch (sort) {
		case 'Name':
			sortedArray = inputProjects.slice().sort((a, b) => {
				return a.name.localeCompare(b.name)
			})
			break
		case 'Status':
			sortedArray = inputProjects.slice().sort((a, b) => {
				if (a.status < b.status) {
					return -1
				}
				if (a.status > b.status) {
					return 1
				}
				return 0
			})
			break
		case 'Type':
			sortedArray = inputProjects.slice().sort((a, b) => {
				const aType = a.project_types?.[0] ?? 'project'
				const bType = b.project_types?.[0] ?? 'project'
				if (aType < bType) {
					return -1
				}
				if (aType > bType) {
					return 1
				}
				return 0
			})
			break
		default:
			break
	}

	if (descending) {
		sortedArray = sortedArray.reverse()
	}

	return sortedArray
}

const sortedProjects = ref(updateSort(projects.value, 'Name'))
const selectedProjects = ref([])
const sortBy = ref('Name')
const descending = ref(false)
const editLinksModal = ref(null)

watch(
	() => projects.value,
	(newVal) => {
		sortedProjects.value = updateSort(newVal, sortBy.value, descending.value)
	},
)

const editLinks = reactive({
	showAffected: false,
	source: { val: '', clear: false },
	discord: { val: '', clear: false },
	wiki: { val: '', clear: false },
	issues: { val: '', clear: false },
})

const updateDescending = () => {
	descending.value = !descending.value
	sortedProjects.value = updateSort(sortedProjects.value, sortBy.value, descending.value)
}

const onBulkEditLinks = async () => {
	try {
		const baseData = {
			issues_url: editLinks.value.issues.clear ? null : editLinks.value.issues.val.trim(),
			source_url: editLinks.value.source.clear ? null : editLinks.value.source.val.trim(),
			wiki_url: editLinks.value.wiki.clear ? null : editLinks.value.wiki.val.trim(),
			discord_url: editLinks.value.discord.clear ? null : editLinks.value.discord.val.trim(),
		}
		const filteredData = Object.fromEntries(Object.entries(baseData).filter(([, v]) => v !== ''))

		await useBaseFetch(`projects?ids=${JSON.stringify(selectedProjects.value.map((x) => x.id))}`, {
			method: 'PATCH',
			body: JSON.stringify(filteredData),
		})

		editLinksModal.value?.hide()
		addNotification({
			title: 'Success',
			text: "Bulk edited selected project's links.",
			type: 'success',
		})
		selectedProjects.value = []

		editLinks.value.issues.val = ''
		editLinks.value.source.val = ''
		editLinks.value.wiki.val = ''
		editLinks.value.discord.val = ''
		editLinks.value.issues.clear = false
		editLinks.value.source.clear = false
		editLinks.value.wiki.clear = false
		editLinks.value.discord.clear = false
	} catch (e) {
		addNotification({
			title: 'An error occurred',
			text: e?.data?.description || e?.message || e || 'Unknown error',
			type: 'error',
		})
		console.error(e)
	}
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

			&:first-child {
				padding-left: var(--spacing-card-bg);
			}

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
