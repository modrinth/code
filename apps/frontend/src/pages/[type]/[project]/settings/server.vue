<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import ServerSettings from '~/components/project-settings/server/index.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.server)
</script>

<template>
	<ServerSettings />
</template>
