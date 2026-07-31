<script setup lang="ts">
import { TagItem } from '@modrinth/ui'
import { onBeforeUnmount, onMounted, ref } from 'vue'

import InstanceCardView from '@/components/ui/library/instance-group/instance-card-view.vue'
import { gatherDuration } from '@/components/ui/library/instance-group/use-instance-drag-gather'
import type { GameInstance } from '@/helpers/types'

const props = withDefaults(
	defineProps<{
		instance: GameInstance
		count?: number
	}>(),
	{
		count: 1,
	},
)

const showGatheredCount = ref(false)
let countTimer: number | undefined

onMounted(() => {
	if (props.count <= 1 || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
		showGatheredCount.value = true
		return
	}

	countTimer = window.setTimeout(() => {
		showGatheredCount.value = true
		countTimer = undefined
	}, gatherDuration - 150)
})

onBeforeUnmount(() => {
	if (countTimer !== undefined) {
		clearTimeout(countTimer)
	}
})
</script>

<template>
	<div aria-hidden="true" class="relative w-full select-none">
		<div
			v-if="count > 1"
			class="absolute inset-x-3 -bottom-2 top-2 rounded-[20px] border border-solid border-surface-5 bg-surface-2 shadow-md motion-safe:transition-opacity motion-safe:duration-200 motion-safe:delay-100 motion-safe:ease-out"
			:class="showGatheredCount ? 'opacity-60' : 'opacity-0'"
		/>
		<div
			v-if="count > 1"
			class="absolute inset-x-1.5 -bottom-1 top-1 rounded-[20px] border border-solid border-surface-4 bg-surface-3 shadow-md motion-safe:transition-opacity motion-safe:duration-150 motion-safe:ease-out"
			:class="showGatheredCount ? 'opacity-80' : 'opacity-0'"
		/>
		<InstanceCardView :instance="instance" class="opacity-90 shadow-lg">
			<template #overlay>
				<TagItem
					v-if="count > 1"
					class="!absolute right-3 top-3 z-[2] border-surface-5 bg-surface-2 font-semibold tabular-nums text-contrast motion-safe:transition-opacity motion-safe:duration-150 motion-safe:ease-out"
				>
					{{ count }}
				</TagItem>
			</template>
		</InstanceCardView>
	</div>
</template>
