<template>
	<section
		class="h-full overflow-auto bg-surface-2 p-4"
		:aria-label="formatMessage(messages[slotName])"
	>
		<component :is="content" v-if="content" />
		<template v-else>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages[slotName]) }}
			</h2>
			<p class="text-secondary">{{ formatMessage(messages.empty) }}</p>
		</template>
	</section>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import type { IDockviewPanelProps } from 'dockview-vue'
import { computed } from 'vue'

import { projectReviewMessages as messages } from '../messages'
import { injectProjectReviewContext } from '../context'
import type { ProjectReviewSlot } from '../types'

defineOptions({ inheritAttrs: false })
type PanelParams = { slot: ProjectReviewSlot }

const props = defineProps<{ params: PanelParams | IDockviewPanelProps<PanelParams> }>()
const { slots } = injectProjectReviewContext()
const { formatMessage } = useVIntl()
const slotName = computed(() =>
	'slot' in props.params ? props.params.slot : props.params.params.slot,
)
const content = computed(() => slots[slotName.value])
</script>
