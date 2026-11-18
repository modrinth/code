<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import LinksSettings from '~/components/project-settings/links/index.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.links)
</script>

<template>
	<LinksSettings />
</template>
