<script setup lang="ts">
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

const { saved, current, reset, save } = useSavable(
	() => ({
		environment:
			projectV3.value.environment?.length === 1 ? projectV3.value.environment[0] : undefined,
	}),
	({ environment }) => {
		if (environment) {
			saving.value = true
			api.projects
				.editV3(projectV2.value.id, { environment })
				.then(() => refreshProject().then(reset))
				.catch(handleError)
				.finally(() => (saving.value = false))
		}
	},
)
</script>
<template>
	<div>
		<UnsavedChangesPopup
			:original="saved"
			:modified="current"
			:saving="saving"
			@reset="reset"
			@save="save"
		/>
		<div class="card experimental-styles-within">
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
				v-else-if="projectV3.side_types_migration_review_status === 'pending'"
				type="warning"
				header="Please review the options below"
				class="mb-3"
			>
				We've just overhauled the Environments system on Modrinth and new options are now available.
				Please ensure the correct option is selected below and then click 'Verify' when you're done!
			</Admonition>
			<h2 class="m-0 mb-2 block text-lg font-extrabold text-contrast">Environment</h2>
			<ProjectSettingsEnvSelector v-model="current.environment" />
		</div>
	</div>
</template>
