<template>
	<template v-if="binding">
		<NodeRenderer
			:key="`${binding.projectId}:${binding.scope}`"
			:nodes="nodes"
			:state="state"
			:global-state="globalState"
			:write="write"
			:title-depth="1"
		/>
	</template>
	<p v-else class="m-0 text-secondary">{{ formatMessage(messages.noReviewActions) }}</p>
</template>

<script setup lang="ts">
import {
	CHECKLIST_META_KEY,
	computeAttentionMap,
	computeNodeMeta,
	resolveChildren,
	type Writer,
} from '@modrinth/moderation/src/types/node'
import NodeRenderer from '@modrinth/moderation/src/types/node/components/NodeRenderer.vue'
import { useVIntl } from '@modrinth/ui'
import { computed, provide } from 'vue'

import type { ReviewTarget } from '~/providers/project-review/review'
import { injectReviewSession } from '~/providers/project-review/review-session'
import { injectReviewStages } from '~/providers/project-review/review-stages'

import { projectReviewMessages as messages } from '../messages'

const props = defineProps<{ target: ReviewTarget }>()
const { formatMessage } = useVIntl()
const session = injectReviewSession()
const { resolve } = injectReviewStages()
const binding = computed(() => resolve(props.target))
const globalState = computed(() =>
	binding.value ? session.readProject(binding.value.projectId) : {},
)
const state = computed(() =>
	binding.value ? session.read(binding.value.projectId, binding.value.scope) : {},
)
const nodes = computed(() =>
	binding.value ? resolveChildren(binding.value.stage, state.value) : [],
)
const write: Writer = (id, value) => {
	if (!binding.value) return
	session.write(binding.value.projectId, binding.value.scope, id, value)
}

provide(
	CHECKLIST_META_KEY,
	computed(() => {
		const metaMap = computeNodeMeta(nodes.value, state.value, () => false)
		return {
			metaMap,
			attentionMap: computeAttentionMap(nodes.value, state.value, metaMap),
			tooltipHtml: new Map<object, string>(),
		}
	}),
)
</script>
