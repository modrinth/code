<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import {
	Admonition,
	commonProjectSettingsMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	ProjectSettingsEnvSelector,
	UnsavedChangesPopup,
	useSavable,
} from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'

const { formatMessage } = useVIntl()

const { currentMember, projectV2, projectV3, refreshProject } = injectProjectPageContext()
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()

const saving = ref(false)

const supportsEnvironment = computed(() =>
	projectV3.value.project_types.some((type) => ['mod', 'modpack'].includes(type)),
)

const needsToVerify = computed(
	() =>
		projectV3.value.side_types_migration_review_status === 'pending' &&
		(projectV3.value.environment?.length ?? 0) > 0 &&
		projectV3.value.environment?.[0] !== 'unknown' &&
		supportsEnvironment.value,
)

const hasPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return (currentMember.value?.permissions & EDIT_DETAILS) === EDIT_DETAILS
})

function getInitialEnv() {
	return projectV3.value.environment?.length === 1 ? projectV3.value.environment[0] : undefined
}

const { saved, current, reset, save } = useSavable(
	() => ({
		environment: getInitialEnv(),
		side_types_migration_review_status: projectV3.value.side_types_migration_review_status,
	}),
	({ environment, side_types_migration_review_status }) => {
		saving.value = true
		side_types_migration_review_status = 'reviewed'
		client.labrinth.projects_v3
			.edit(projectV2.value.id, { environment, side_types_migration_review_status })
			.then(() => refreshProject().then(reset))
			.catch(handleError)
			.finally(() => (saving.value = false))
	},
)
// Set current to reviewed, which will trigger unsaved changes popup.
// It should not be possible to save without reviewing it.
const originalEnv = getInitialEnv()
if (originalEnv && originalEnv !== 'unknown') {
	current.value.side_types_migration_review_status = 'reviewed'
}

const messages = defineMessages({
	verifyButton: {
		id: 'project.settings.environment.verification.verify-button',
		defaultMessage: 'Verify',
	},
	verifyLabel: {
		id: 'project.settings.environment.verification.verify-text',
		defaultMessage: `Verify that this project's environment is set correctly.`,
	},
	wrongProjectTypeTitle: {
		id: 'project.settings.environment.notice.wrong-project-type.title',
		defaultMessage: `This project type does not support environment metadata`,
	},
	wrongProjectTypeDescription: {
		id: 'project.settings.environment.notice.wrong-project-type.description',
		defaultMessage: `Only mod or modpack projects can have environment metadata.`,
	},
	missingEnvTitle: {
		id: 'project.settings.environment.notice.missing-env.title',
		defaultMessage: `Please select an environment for your project`,
	},
	missingEnvDescription: {
		id: 'project.settings.environment.notice.missing-env.description',
		defaultMessage: `Your project is missing environment metadata, please select the appropriate option below.`,
	},
	multipleEnvironmentsTitle: {
		id: 'project.settings.environment.notice.multiple-environments.title',
		defaultMessage: 'Your project has multiple environments',
	},
	multipleEnvironmentsDescription: {
		id: 'project.settings.environment.notice.multiple-environments.description',
		defaultMessage:
			"Different versions of your project have different environments selected, so you can't edit them globally at this time.",
	},
	reviewOptionsTitle: {
		id: 'project.settings.environment.notice.review-options.title',
		defaultMessage: 'Please review the options below',
	},
	reviewOptionsDescription: {
		id: 'project.settings.environment.notice.review-options.description',
		defaultMessage:
			"We've just overhauled the Environments system on Modrinth and new options are now available. Please ensure the correct option is selected below and then click 'Verify' when you're done!",
	},
})
</script>
<template>
	<div>
		<div class="card experimental-styles-within">
			<h2 class="m-0 mb-2 block text-lg font-extrabold text-contrast">
				{{ formatMessage(commonProjectSettingsMessages.environment) }}
			</h2>
			<Admonition
				v-if="!supportsEnvironment"
				type="critical"
				:header="formatMessage(messages.wrongProjectTypeTitle)"
				:body="formatMessage(messages.wrongProjectTypeDescription)"
				class="mb-3"
			/>
			<template v-else>
				<Admonition
					v-if="!hasPermission"
					type="critical"
					:header="formatMessage(commonProjectSettingsMessages.noPermissionTitle)"
					:body="formatMessage(commonProjectSettingsMessages.noPermissionDescription)"
					class="mb-3"
				/>
				<Admonition
					v-else-if="
						!projectV3.environment ||
						projectV3.environment.length === 0 ||
						(projectV3.environment.length === 1 && projectV3.environment[0] === 'unknown')
					"
					type="critical"
					:header="formatMessage(messages.missingEnvTitle)"
					:body="formatMessage(messages.missingEnvDescription)"
					class="mb-3"
				/>
				<Admonition
					v-else-if="projectV3.environment.length > 1"
					type="info"
					:header="formatMessage(messages.multipleEnvironmentsTitle)"
					:body="formatMessage(messages.multipleEnvironmentsDescription)"
					class="mb-3"
				/>
				<Admonition
					v-else-if="needsToVerify"
					type="warning"
					:header="formatMessage(messages.reviewOptionsTitle)"
					:body="formatMessage(messages.reviewOptionsDescription)"
					class="mb-3"
				/>
				<ProjectSettingsEnvSelector
					v-model="current.environment"
					:disabled="!hasPermission || (projectV3?.environment?.length ?? 0) > 1"
				/>
			</template>
		</div>
		<UnsavedChangesPopup
			v-if="supportsEnvironment && hasPermission && (projectV3?.environment?.length ?? 0) <= 1"
			:original="saved"
			:modified="current"
			:saving="saving"
			:can-reset="!needsToVerify"
			:text="needsToVerify ? messages.verifyLabel : undefined"
			:save-label="needsToVerify ? messages.verifyButton : undefined"
			:save-icon="needsToVerify ? CheckIcon : undefined"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>
