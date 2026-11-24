<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import DescriptionSettings from '~/components/project-settings/description/index.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.description)
</script>

<template>
	<DescriptionSettings />
</template>
