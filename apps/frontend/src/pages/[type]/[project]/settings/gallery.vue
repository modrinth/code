<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import GallerySettings from '~/components/project-settings/gallery/index.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.gallery)
</script>

<template>
	<GallerySettings />
</template>
