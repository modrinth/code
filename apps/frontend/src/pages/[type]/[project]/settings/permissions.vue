<script setup lang="ts">
import { commonProjectSettingsMessages, injectProjectPageContext } from '@modrinth/ui'
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

useProjectSettingsHeadTitle(commonProjectSettingsMessages.permissions)
</script>

<template>
	<ValidationMessage :check="permissionsValidation" class="mb-4" />
	<ProjectPermissions
		:project="project"
		:members="allMembers"
		:is-moderator="isModerator"
		:refresh-project-validation="refreshProjectValidation"
	/>
</template>
