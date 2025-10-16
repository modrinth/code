<template>
	<div class="scrollable-pane-wrapper">
		<div
			class="wrapper-wrapper"
			:class="{
				'top-fade': !scrollableAtTop && !disableScrolling,
				'bottom-fade': !scrollableAtBottom && !disableScrolling,
			}"
		>
			<div
				ref="scrollablePane"
				:class="{
					'max-h-[19rem]': !disableScrolling,
				}"
				class="scrollable-pane"
				@scroll="onScroll"
			>
				<slot />
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'

withDefaults(
	defineProps<{
		disableScrolling?: boolean
	}>(),
	{
		disableScrolling: false,
	},
)

const scrollableAtTop = ref(true)
const scrollableAtBottom = ref(false)
const scrollablePane = ref(null)
let resizeObserver
onMounted(() => {
	resizeObserver = new ResizeObserver(function () {
		if (scrollablePane.value) {
			updateFade(
				scrollablePane.value.scrollTop,
				scrollablePane.value.offsetHeight,
				scrollablePane.value.scrollHeight,
			)
		}
	})
	resizeObserver.observe(scrollablePane.value)
})
onUnmounted(() => {
	if (resizeObserver) {
		resizeObserver.disconnect()
	}
})
function updateFade(scrollTop, offsetHeight, scrollHeight) {
	console.log(scrollTop, offsetHeight, scrollHeight)
	scrollableAtBottom.value = Math.ceil(scrollTop + offsetHeight) >= scrollHeight
	scrollableAtTop.value = scrollTop <= 0
}
function onScroll({ target: { scrollTop, offsetHeight, scrollHeight } }) {
	updateFade(scrollTop, offsetHeight, scrollHeight)
}
</script>

<style lang="scss" scoped>
@property --_top-fade-height {
	syntax: '<length-percentage>';
	inherits: false;
	initial-value: 0%;
}

@property --_bottom-fade-height {
	syntax: '<length-percentage>';
	inherits: false;
	initial-value: 0%;
}

.scrollable-pane-wrapper {
	display: flex;
	flex-direction: column;
	position: relative;
}

.wrapper-wrapper {
	flex-grow: 1;
	display: flex;
	overflow: hidden;
	position: relative;
	transition:
		--_top-fade-height 0.05s linear,
		--_bottom-fade-height 0.05s linear;

	--_fade-height: 3rem;

	mask-image: linear-gradient(
		transparent,
		rgb(0 0 0 / 100%) var(--_top-fade-height, 0%),
		rgb(0 0 0 / 100%) calc(100% - var(--_bottom-fade-height, 0%)),
		transparent 100%
	);

	&.top-fade {
		--_top-fade-height: var(--_fade-height);
	}

	&.bottom-fade {
		--_bottom-fade-height: var(--_fade-height);
	}
}
.scrollable-pane {
	display: flex;
	flex-direction: column;
	gap: 0.25rem;
	height: 100%;
	width: 100%;
	overflow-y: auto;
	overflow-x: hidden;
	position: relative;
}
</style>
