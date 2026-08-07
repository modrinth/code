<script setup lang="ts">
import { SmartClickable, Toggle } from '@modrinth/ui'
import { computed, useId } from 'vue'

const props = defineProps<{
	disabled?: boolean
}>()

const value = defineModel<boolean>({ required: true })

const baseId = useId()
const toggleId = computed(() => `toggle-card-toggle-${baseId}`)

function toggle() {
	if (props.disabled) return
	value.value = !value.value
}
</script>

<template>
	<SmartClickable
		class="flex w-full flex-col overflow-clip rounded-2xl border border-solid border-surface-4 bg-surface-3"
		:disabled="disabled"
	>
		<template #clickable>
			<button
				aria-hidden="true"
				tabindex="-1"
				:disabled="disabled"
				class="flex h-full w-full"
				:class="disabled ? 'cursor-not-allowed' : 'cursor-pointer'"
				@click="toggle"
			/>
		</template>
		<div class="grid w-full grid-cols-[1fr_auto] items-center gap-6 p-4">
			<div>
				<slot :toggle-id="toggleId" />
			</div>
			<div>
				<slot name="toggle" :disabled="disabled">
					<Toggle
						:id="toggleId"
						v-model="value"
						:disabled="disabled"
						class="smart-clickable:allow-pointer-events"
					/>
				</slot>
			</div>
		</div>
		<Transition name="toggle-card-content">
			<div v-if="value && $slots.expanded" class="smart-clickable:allow-pointer-events">
				<div>
					<div class="border-0 border-t border-solid border-surface-4 bg-surface-2 p-4">
						<slot name="expanded" />
					</div>
				</div>
			</div>
		</Transition>
	</SmartClickable>
</template>
<style scoped>
.toggle-card-content-enter-active,
.toggle-card-content-leave-active {
	display: grid;
	grid-template-rows: 1fr;
	transition: grid-template-rows 0.25s var(--ease-out-expo);

	& > div {
		grid-row: 1 / span 2;
	}
}

.toggle-card-content-enter-from,
.toggle-card-content-leave-to {
	grid-template-rows: 0fr;
}
</style>
