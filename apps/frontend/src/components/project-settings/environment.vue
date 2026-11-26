<template>
	<LoadingIndicator v-if="!projectV3" class="py-6" />
	<SettingsSection v-else-if="showEnvironmentMigration" title="Project environment">
		<div class="settings-content-card"><EnvironmentMigration /></div>
	</SettingsSection>
	<div v-else class="grid place-content-center py-32">
		<div class="flex flex-col items-center gap-5 text-center">
			<div class="flex flex-col gap-2">
				<div class="text-xl font-semibold text-contrast">
					{{ formatMessage(messages.perVersion) }}
				</div>
				<div>{{ formatMessage(messages.manageVersions) }}</div>
			</div>
			<ButtonLink
				type="colored"
				color="green"
				:to="`/${projectV2.project_type}/${projectV2.id}/settings/versions`"
				class="items flex"
			>
				<SettingsIcon /> {{ formatMessage(messages.editVersions) }}
			</ButtonLink>
		</div>
	</div>
</template>

<script setup lang="ts">
import { SettingsIcon } from '@modrinth/assets'
import {
	ButtonLink,
	defineMessages,
	EnvironmentMigration,
	injectProjectPageContext,
	LoadingIndicator,
	useVIntl,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'

import SettingsSection from './settings-section.vue'

const { currentMember, projectV2, projectV3 } = injectProjectPageContext()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	perVersion: {
		id: 'project.settings.environment.per-version',
		defaultMessage: 'Environments are now managed per version.',
	},
	manageVersions: {
		id: 'project.settings.environment.manage-versions',
		defaultMessage: 'Visit Project Settings to manage environments for each version.',
	},
	editVersions: {
		id: 'project.settings.environment.edit-versions',
		defaultMessage: 'Edit versions',
	},
})

const showEnvironmentMigration = computed(() => {
	return isStaff(currentMember.value?.user)
})
</script>
