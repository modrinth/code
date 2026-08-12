<script setup lang="ts">
import { SmartClickable, Toggle } from '@modrinth/ui'
import { computed, useId } from 'vue'

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
	<SmartClickable
		class="flex w-full flex-col overflow-clip rounded-2xl border border-solid border-surface-4 bg-surface-3"
		:disabled="isToggleLocked"
	>
		<template #clickable>
			<button
				aria-hidden="true"
				tabindex="-1"
				:disabled="isToggleLocked"
				class="flex h-full w-full"
				:class="isToggleLocked ? 'cursor-not-allowed' : 'cursor-pointer'"
				@click="toggle"
			/>
		</template>
		<div class="grid w-full grid-cols-[1fr_auto] items-center gap-6 p-4">
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
		<div
			v-if="$slots.footer"
			class="smart-clickable:allow-pointer-events border-0 border-t bg-surface-3 border-solid border-surface-4 px-4 py-3"
		>
			<slot name="footer" />
		</div>
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
