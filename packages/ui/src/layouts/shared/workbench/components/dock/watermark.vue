<template>
	<div v-if="inPopout" class="flex h-full items-center justify-center p-3 text-secondary">
		<span role="status">{{ formatMessage(messages.emptyPopout) }}</span>
	</div>
	<component
		:is="workbench.documents.watermark"
		v-else-if="workbench.documents?.watermark && inCenter"
		:params="params"
	/>
</template>

<script setup lang="ts">
import type { WatermarkRendererInitParameters } from 'dockview-vue'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectWorkbench, injectWorkbenchDock } from '../../providers/workbench'

defineOptions({ inheritAttrs: false })

/** Shown in an empty group: a note in an emptied popout window, or the documents' own in the center. */
const props = defineProps<{ params: WatermarkRendererInitParameters }>()

const { formatMessage } = useVIntl()
const workbench = injectWorkbench()
const host = injectWorkbenchDock()

const messages = defineMessages({
	emptyPopout: {
		id: 'workbench.popout.empty',
		defaultMessage: 'No tabs open in this panel',
	},
})

const inPopout = computed(() => props.params.group?.api.location.type === 'popout')
const inCenter = host === 'center'
</script>
