<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import GeneralSettings from '~/components/project-settings/general/index.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.general)
</script>

<template>
	<GeneralSettings />
</template>
