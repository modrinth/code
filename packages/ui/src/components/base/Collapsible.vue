<template>
	<div
		class="accordion-content"
		:class="[
			baseClass ?? '',
			{
				open: isOpen,
				'no-transition': !shouldAnimate,
				'overflow-visible': overflowVisible && isFullyOpen,
				fade,
			},
		]"
		:style="isHidden ? { display: 'none' } : {}"
		@transitionend="onTransitionEnd"
	>
		<div v-bind="$attrs" :inert="collapsed">
			<slot />
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'

const props = withDefaults(
	defineProps<{
		baseClass?: string
		collapsed: boolean
		overflowVisible?: boolean
		fade?: boolean
		durationMs?: number
	}>(),
	{ baseClass: undefined, durationMs: 300 },
)

const transitionDuration = computed(() => `${props.durationMs}ms`)

defineOptions({
	inheritAttrs: false,
})

const shouldAnimate = ref(false)
const isHidden = ref(props.collapsed)
const isOpen = ref(!props.collapsed)
const isFullyOpen = ref(!props.collapsed)

onMounted(() => {
	requestAnimationFrame(() => {
		shouldAnimate.value = true
	})
})

watch(
	() => props.collapsed,
	async (collapsed) => {
		if (!collapsed) {
			// Opening
			isHidden.value = false
			isFullyOpen.value = false

			if (!shouldAnimate.value) {
				isOpen.value = true
				isFullyOpen.value = true
				return
			}

			// Wait for display: none removal to take effect, then animate open
			await nextTick()
			requestAnimationFrame(() => {
				isOpen.value = true
			})
		} else {
			// Closing
			// Remove overflow-visible so content is clipped during animation
			isFullyOpen.value = false

			if (!shouldAnimate.value) {
				isOpen.value = false
				isHidden.value = true
				return
			}

			// Wait a frame for overflow: hidden to apply, THEN start closing
			await nextTick()
			requestAnimationFrame(() => {
				isOpen.value = false
			})
		}
	},
)

function onTransitionEnd(e: TransitionEvent) {
	if (e.target !== e.currentTarget) return
	if (props.collapsed) {
		isHidden.value = true
	} else {
		isFullyOpen.value = true
	}
}
</script>

<style scoped>
.accordion-content {
	display: grid;
	grid-template-rows: 0fr;
	transition: grid-template-rows v-bind(transitionDuration) ease-in-out;
}

.accordion-content.fade > div {
	transition: opacity v-bind(transitionDuration) ease-in-out;
}

.accordion-content.fade:not(.open) > div {
	opacity: 0;
}

.accordion-content.no-transition,
.accordion-content.no-transition.fade > div {
	transition: none !important;
}

@media (prefers-reduced-motion) {
	.accordion-content,
	.accordion-content.fade > div {
		transition: none !important;
	}
}

.accordion-content.open {
	grid-template-rows: 1fr;
}

.accordion-content > div {
	overflow: hidden;
}

.accordion-content.overflow-visible > div {
	overflow: visible;
}
</style>
