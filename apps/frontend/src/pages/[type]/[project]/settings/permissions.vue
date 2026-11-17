<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import PermissionsSettings from '~/components/project-settings/permissions.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.permissions)
</script>

<template>
	<PermissionsSettings />
</template>
