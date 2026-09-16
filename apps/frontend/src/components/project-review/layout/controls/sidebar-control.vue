<template>
	<div v-if="params.group.id === cornerGroupId" class="flex h-full items-center px-1">
		<IconButton
			type="quiet"
			size="sm"
			:label="formatMessage(label)"
			:aria-expanded="visible"
			@click="toggleSidebar(side)"
		>
			<PanelLeftOpenIcon v-if="side === 'left'" aria-hidden="true" />
			<PanelRightOpenIcon v-else aria-hidden="true" />
		</IconButton>
	</div>
</template>

<script setup lang="ts">
import { PanelLeftOpenIcon, PanelRightOpenIcon } from '@modrinth/assets'
import { IconButton, useVIntl } from '@modrinth/ui'
import type { IDockviewHeaderActionsProps } from 'dockview-vue'
import { computed } from 'vue'

import { projectReviewMessages as messages } from '../../messages'
import { injectProjectReviewContext } from '../context'

defineOptions({ inheritAttrs: false })
const props = defineProps<{ side: 'left' | 'right'; params: IDockviewHeaderActionsProps }>()
const { formatMessage } = useVIntl()
const { leftVisible, rightVisible, topLeftGroupId, topRightGroupId, toggleSidebar } =
	injectProjectReviewContext()
const visible = computed(() => (props.side === 'left' ? leftVisible.value : rightVisible.value))
const cornerGroupId = computed(() =>
	props.side === 'left' ? topLeftGroupId.value : topRightGroupId.value,
)
const label = computed(() => {
	if (props.side === 'left') return visible.value ? messages.hideLeft : messages.showLeft
	return visible.value ? messages.hideRight : messages.showRight
})
</script>
