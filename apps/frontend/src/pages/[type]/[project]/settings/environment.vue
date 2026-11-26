<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import EnvironmentSettings from '~/components/project-settings/environment.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.environment)
</script>

<template>
	<EnvironmentSettings />
</template>
