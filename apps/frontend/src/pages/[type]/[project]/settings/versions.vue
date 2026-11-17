<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import VersionsSettings from '~/components/project-settings/versions/index.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.versions)

const emit = defineEmits<{ onDownload: [] }>()
</script>

<template>
	<VersionsSettings @on-download="emit('onDownload')" />
</template>
