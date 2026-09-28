<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'
import {
	ButtonLink,
	commonProjectSettingsMessages,
	defineMessage,
	injectProjectPageContext,
	useVIntl,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { computed } from 'vue'

import ProjectPermissions from '~/components/ui/project-settings/modpack-permissions/ProjectPermissions.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useProjectNagMessages } from '~/composables/project-nag-validation'

const auth = await useAuth()
const flags = useFeatureFlags()
const isModerator = computed(
	() => isStaff(auth.value?.user) && !flags.value.showModeratorProjectMemberUi,
)
const { projectV2: project, allMembers, refreshProjectValidation } = injectProjectPageContext()
const permissionsValidation = useProjectNagMessages('permissions')
const { formatMessage } = useVIntl()
const openImageInNewTab = defineMessage({
	id: 'project.settings.permissions.open-image-in-new-tab',
	defaultMessage: 'Open image in new tab',
})

useProjectSettingsHeadTitle(commonProjectSettingsMessages.permissions)
</script>

<template>
	<ValidationMessage :check="permissionsValidation" class="mb-4" />
	<ProjectPermissions
		:project="project"
		:members="allMembers"
		:is-moderator="isModerator"
		:refresh-project-validation="refreshProjectValidation"
	>
		<template #image-viewer-actions="{ item }">
			<ButtonLink
				v-tooltip="formatMessage(openImageInNewTab)"
				type="quiet"
				class="!w-9 !rounded-full !p-0"
				:aria-label="formatMessage(openImageInNewTab)"
				:href="item.src"
				target="_blank"
			>
				<ExternalIcon aria-hidden="true" />
			</ButtonLink>
		</template>
	</ProjectPermissions>
</template>
