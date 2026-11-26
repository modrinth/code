<template>
	<SettingsSection :title="formatMessage(messages.dangerZone)" :title-id="dangerZoneTitleId" danger>
		<ConfirmModal
			ref="confirmModal"
			:title="formatMessage(messages.deleteConfirmationTitle)"
			:description="formatMessage(messages.deleteConfirmationDescription)"
			:has-to-type="true"
			:confirmation-text="project.name"
			:proceed-label="formatMessage(messages.deleteConfirmationProceedText)"
			@proceed="deleteProject"
		/>
		<DangerZonePanel>
			<MonetizationActions v-if="!isServerProject" />
			<OwnershipActions />
			<div class="danger-zone-row">
				<div>
					<h3 class="mb-1 mt-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.deleteProjectTitle) }}
					</h3>
					<p class="m-0 leading-normal text-secondary">
						{{ formatMessage(messages.deleteProjectDescription) }}
					</p>
				</div>
				<Button
					size="sm"
					type="colored"
					color="red"
					:disabled="!hasDeletePermission"
					@click="confirmDeleteProject"
					>{{ formatMessage(messages.deleteProjectButton) }}</Button
				>
			</div>
		</DangerZonePanel>
	</SettingsSection>
</template>

<script setup lang="ts">
import {
	Button,
	ConfirmModal,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	useVIntl,
} from '@modrinth/ui'
import { useId } from 'vue'

import { ensurePrivileged, isReauthCancelled, runPrivileged } from '~/composables/reauth.ts'

import SettingsSection from '../../settings-section.vue'
import MonetizationActions from './monetization-actions.vue'
import OwnershipActions from './ownership-actions.vue'
import DangerZonePanel from './panel.vue'

const { formatMessage } = useVIntl()
const dangerZoneTitleId = useId()
const { addNotification } = injectNotificationManager()
const { projectV3: project, currentMember } = injectProjectPageContext()
const { labrinth } = injectModrinthClient()
const router = useNativeRouter()
const tags = useGeneratedState()

const confirmModal = useTemplateRef('confirmModal')
const isServerProject = computed(() => project.value?.minecraft_server != null)

const hasDeletePermission = computed(() => {
	const DELETE_PROJECT = 1 << 7
	return ((currentMember.value?.permissions ?? 0) & DELETE_PROJECT) === DELETE_PROJECT
})

const requiresPrivileged = computed(() =>
	tags.value.approvedStatuses.includes(project.value.status),
)

async function confirmDeleteProject() {
	if (await ensurePrivileged(requiresPrivileged.value)) confirmModal.value?.show()
}

const deleteProject = async () => {
	try {
		await runPrivileged(
			() => labrinth.projects_v3.deleteProject(project.value.id),
			requiresPrivileged.value,
		)
	} catch (err) {
		if (isReauthCancelled(err)) {
			return
		}
		throw err
	}
	await initUserProjects()
	await router.push('/dashboard/projects')
	addNotification({
		title: formatMessage(messages.deletedTitle),
		text: formatMessage(messages.deletedDescription),
		type: 'success',
	})
}

const messages = defineMessages({
	deleteProjectDescription: {
		id: 'project.settings.delete-project.description',
		defaultMessage:
			'Deleting this project will permanently remove it from Modrinth. Deleted projects cannot be recovered by Modrinth staff or support. Files actively used by modpacks hosted on Modrinth may remain available after deletion.',
	},
	deletedTitle: {
		id: 'project.settings.delete-project.deleted.title',
		defaultMessage: 'Project deleted',
	},
	deletedDescription: {
		id: 'project.settings.delete-project.deleted.description',
		defaultMessage: 'Your project has been deleted.',
	},
	dangerZone: {
		id: 'project.settings.danger-zone',
		defaultMessage: 'Danger zone',
	},
	deleteProjectTitle: {
		id: 'project.settings.delete-project.title',
		defaultMessage: 'Delete project',
	},
	deleteProjectButton: {
		id: 'project.settings.delete-project.button',
		defaultMessage: 'Delete project',
	},
	deleteConfirmationTitle: {
		id: 'project.settings.delete-project.confirmation.title',
		defaultMessage: 'Are you sure you want to delete this project?',
	},
	deleteConfirmationDescription: {
		id: 'project.settings.delete-project.confirmation.description',
		defaultMessage: `If you proceed, all of this project's information and all of its versions will be immediately deleted from our database. None of it is recoverable later if you change your mind. Consider just setting your project to be Private for a less permanent option.`,
	},
	deleteConfirmationProceedText: {
		id: 'project.settings.delete-project.confirmation.proceed-text',
		defaultMessage: 'Permanently delete project',
	},
})
</script>
