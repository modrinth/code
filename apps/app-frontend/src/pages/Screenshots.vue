<script setup lang="ts">
import { ImagesIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { onActivated } from 'vue'

import ScreenshotsPage from '@/components/ui/screenshots-page/index.vue'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'

defineOptions({ name: 'ScreenshotsPage' })

const { formatMessage } = useVIntl()
const messages = defineMessages({
	screenshots: { id: 'app.screenshots.heading', defaultMessage: 'Screenshots' },
})
const breadcrumb = useRootBreadcrumb({
	slot: 'root',
	id: 'screenshots',
	label: formatMessage(messages.screenshots),
	to: '/screenshots',
	visual: { type: 'icon', component: ImagesIcon },
})
onActivated(breadcrumb.reset)
</script>

<template>
	<div class="box-border h-full p-6">
		<ScreenshotsPage show-heading />
	</div>
</template>
