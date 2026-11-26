<script setup lang="ts">
import { computed, useId } from 'vue'

import SmartClickable from './SmartClickable.vue'
import Toggle from './Toggle.vue'

const props = defineProps<{
	disabled?: boolean
	toggleDisabled?: boolean
}>()

const value = defineModel<boolean>({ required: true })

const baseId = useId()
const toggleId = computed(() => `toggle-card-toggle-${baseId}`)
const isToggleLocked = computed(() => !!props.disabled || (!!props.toggleDisabled && value.value))

function toggle() {
	if (isToggleLocked.value) return
	value.value = !value.value
}
</script>

<template>
	<div
		class="flex w-full flex-col overflow-clip rounded-2xl border border-solid border-surface-4 bg-surface-3"
	>
		<div
			class="relative isolate before:pointer-events-none before:absolute before:inset-0 before:-z-10 before:bg-surface-3 before:transition-[filter] before:duration-150 before:content-[''] motion-reduce:before:transition-none"
			:class="{ 'hover:before:[filter:brightness(1.15)]': !isToggleLocked }"
		>
			<SmartClickable
				class="grid w-full grid-cols-[1fr_auto] items-center gap-6 p-4"
				:disabled="isToggleLocked"
			>
				<template #clickable>
					<button
						type="button"
						aria-hidden="true"
						tabindex="-1"
						:disabled="isToggleLocked"
						class="m-0 flex h-full w-full border-0 bg-transparent p-0"
						:class="isToggleLocked ? 'cursor-not-allowed' : 'cursor-pointer'"
						@click="toggle"
					/>
				</template>
				<div>
					<slot :toggle-id="toggleId" />
				</div>
				<div>
					<slot name="toggle" :disabled="isToggleLocked">
						<Toggle
							:id="toggleId"
							v-model="value"
							:disabled="isToggleLocked"
							class="smart-clickable:allow-pointer-events"
						/>
					</slot>
				</div>
			</SmartClickable>
		</div>
		<div v-if="$slots['after-summary']" class="border-0 border-t border-solid border-surface-4">
			<slot name="after-summary" />
		</div>
		<Transition name="toggle-card-content">
			<div v-if="value && $slots.expanded">
				<div>
					<div class="border-0 border-t border-solid border-surface-4 bg-surface-2 p-4">
						<slot name="expanded" />
					</div>
				</div>
			</div>
		</Transition>
		<div
			v-if="$slots.footer"
			class="border-0 border-t bg-surface-3 border-solid border-surface-4 px-4 py-3"
		>
			<slot name="footer" />
		</div>
	</div>
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
