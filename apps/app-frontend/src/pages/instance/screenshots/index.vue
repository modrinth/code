<script setup lang="ts">
import ScreenshotsPage from '@/components/ui/screenshots-page/index.vue'

import { injectInstancePage } from '../instance-context'

const instancePage = injectInstancePage()
</script>

<template>
	<ScreenshotsPage :instance-id="instancePage.instanceId.value" />
</template>
