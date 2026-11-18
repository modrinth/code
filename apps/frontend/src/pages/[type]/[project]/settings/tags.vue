<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import TagsSettings from '~/components/project-settings/tags/index.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.tags)
</script>

<template>
	<TagsSettings />
</template>
