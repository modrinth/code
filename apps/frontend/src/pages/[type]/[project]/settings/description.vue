<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import DescriptionSettings from '~/components/project-settings/description.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.description)
</script>

<template>
	<DescriptionSettings />
</template>
