<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import {
	Admonition,
	injectNotificationManager,
	injectProjectPageContext,
	ProjectSettingsEnvSelector,
	UnsavedChangesPopup,
	useSavable,
} from '@modrinth/ui'
import { injectApi } from '@modrinth/ui/src/providers/api.ts'

const { projectV2, projectV3, refreshProject } = injectProjectPageContext()
const { handleError } = injectNotificationManager()
const api = injectApi()

const saving = ref(false)

const supportsEnvironment = computed(() =>
	projectV3.value.project_types.some((type) => ['mod', 'modpack'].includes(type)),
)

const needsToVerify = computed(
	() =>
		projectV3.value.side_types_migration_review_status === 'pending' &&
		projectV3.value.environment?.length > 0 &&
		projectV3.value.environment?.[0] !== 'unknown' &&
		supportsEnvironment.value,
)

const originalEnv =
	projectV3.value.environment?.length === 1 ? projectV3.value.environment[0] : undefined

const { saved, current, reset, save } = useSavable(
	() => ({
		environment: originalEnv,
		side_types_migration_review_status: projectV3.value.side_types_migration_review_status,
	}),
	({ environment, side_types_migration_review_status }) => {
		saving.value = true
		side_types_migration_review_status = 'reviewed'
		api.projects
			.editV3(projectV2.value.id, { environment, side_types_migration_review_status })
			.then(() => refreshProject().then(reset))
			.catch(handleError)
			.finally(() => (saving.value = false))
	},
)
// Set current to reviewed, which will trigger unsaved changes popup.
// It should not be possible to save without reviewing it.
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
})
</script>
<template>
	<div>
		<UnsavedChangesPopup
			v-if="supportsEnvironment"
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
		<div class="card experimental-styles-within">
			<h2 class="m-0 mb-2 block text-lg font-extrabold text-contrast">Environment</h2>
			<Admonition
				v-if="!supportsEnvironment"
				type="critical"
				header="This project type does not support environment metadata"
				class="mb-3"
			>
				Only mod or modpack projects can have environment metadata.
			</Admonition>
			<template v-else>
				<Admonition
					v-if="
						!projectV3.environment ||
						projectV3.environment.length === 0 ||
						projectV3.environment[0] === 'unknown'
					"
					type="critical"
					header="Please select an environment for your project"
					class="mb-3"
				>
					Your project is missing environment metadata, please select the appropriate option below.
				</Admonition>
				<Admonition
					v-else-if="projectV3.environment.length > 1"
					type="info"
					header="Your project has multiple environments."
					class="mb-3"
				>
					Different versions of your project have different environments selected, so you can't edit
					them globally at this time.
				</Admonition>
				<Admonition
					v-else-if="needsToVerify"
					type="warning"
					header="Please review the options below"
					class="mb-3"
				>
					We've just overhauled the Environments system on Modrinth and new options are now
					available. Please ensure the correct option is selected below and then click 'Verify' when
					you're done!
				</Admonition>
				<ProjectSettingsEnvSelector v-model="current.environment" />
			</template>
		</div>
	</div>
</template>
