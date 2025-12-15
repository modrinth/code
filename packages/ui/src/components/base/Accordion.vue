<template>
	<div v-bind="$attrs">
		<button
			v-if="!!slots.title"
			:class="buttonClass ?? 'flex flex-col gap-2 bg-transparent m-0 p-0 border-none'"
			@click="() => (forceOpen ? undefined : toggledOpen ? close() : open())"
		>
			<slot name="button" :open="isOpen">
				<div class="flex items-center gap-1 w-full">
					<slot name="title" />
					<DropdownIcon
						v-if="!forceOpen"
						class="ml-auto size-5 transition-transform duration-300 shrink-0"
						:class="{ 'rotate-180': isOpen }"
					/>
				</div>
			</slot>
			<slot name="summary" />
		</button>
		<div class="accordion-content" :class="{ open: isOpen }">
			<div>
				<div :class="contentClass ? contentClass : ''" :inert="!isOpen">
					<slot />
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { DropdownIcon } from '@modrinth/assets'
import { computed, ref, useSlots, watch } from 'vue'

const props = withDefaults(
	defineProps<{
		openByDefault?: boolean
		type?: 'standard' | 'outlined' | 'transparent'
		buttonClass?: string
		contentClass?: string
		titleWrapperClass?: string
		forceOpen?: boolean
	}>(),
	{
		type: 'standard',
		openByDefault: false,
		buttonClass: null,
		contentClass: null,
		titleWrapperClass: null,
		forceOpen: false,
	},
)

const toggledOpen = ref(props.openByDefault)
const isOpen = computed(() => toggledOpen.value || props.forceOpen)
const emit = defineEmits(['onOpen', 'onClose'])

const slots = useSlots()

watch(
	() => props.openByDefault,
	(newValue) => {
		if (newValue !== toggledOpen.value) {
			toggledOpen.value = newValue
		}
	},
	{ immediate: true },
)

function open() {
	toggledOpen.value = true
	emit('onOpen')
}
function close() {
	toggledOpen.value = false
	emit('onClose')
}

defineExpose({
	open,
	close,
	isOpen: toggledOpen,
})

defineOptions({
	inheritAttrs: false,
})
</script>
<style scoped>
.accordion-content {
	display: grid;
	grid-template-rows: 0fr;
	transition: grid-template-rows 0.3s ease-in-out;
}

@media (prefers-reduced-motion) {
	.accordion-content {
		transition: none !important;
	}
}

.accordion-content.open {
	grid-template-rows: 1fr;
}

.accordion-content > div {
	overflow: hidden;
}
</style>
