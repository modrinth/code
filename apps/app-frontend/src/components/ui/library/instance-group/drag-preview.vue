<script setup lang="ts">
import { TagItem } from '@modrinth/ui'

import InstanceCardView from '@/components/ui/library/instance-group/instance-card-view.vue'
import type { GameInstance } from '@/helpers/types'

withDefaults(
	defineProps<{
		instance: GameInstance
		count?: number
	}>(),
	{
		count: 1,
	},
)
</script>

<template>
	<div aria-hidden="true" class="relative w-full select-none">
		<div
			v-if="count > 1"
			class="absolute inset-x-3 -bottom-2 top-2 rounded-[20px] border border-solid border-surface-5 bg-surface-2 opacity-60 shadow-md"
		/>
		<div
			v-if="count > 1"
			class="absolute inset-x-1.5 -bottom-1 top-1 rounded-[20px] border border-solid border-surface-4 bg-surface-3 opacity-80 shadow-md"
		/>
		<InstanceCardView :instance="instance" class="opacity-90 shadow-lg">
			<template #overlay>
				<TagItem
					v-if="count > 1"
					class="!absolute right-3 top-3 z-[2] border-surface-5 bg-surface-2 font-semibold tabular-nums text-contrast"
				>
					{{ count }}
				</TagItem>
			</template>
		</InstanceCardView>
	</div>
</template>
