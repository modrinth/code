<template>
	<span
		v-if="flags.length"
		class="pointer-events-none flex shrink-0 items-center gap-0.5 text-xs font-medium leading-none text-purple"
		:aria-label="`${flags.length} flags`"
	>
		<TagCategoryFlagIcon class="size-3.5 shrink-0 text-purple" aria-hidden="true" />
		<span aria-hidden="true">{{ flags.length }}</span>
	</span>
</template>

<script setup lang="ts">
import { TagCategoryFlagIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { injectReviewFlags } from '~/providers/project-review/review-flags'

const props = defineProps<{ panelKey?: string }>()
const reviewFlags = injectReviewFlags(null)
const flags = computed(() => (props.panelKey ? (reviewFlags?.forPanel(props.panelKey) ?? []) : []))
</script>
