<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'

const props = withDefaults(defineProps<{ gap?: string }>(), { gap: '2px' })

const cornerEl = ref<HTMLElement | null>(null)
const w = ref('0px')
const h = ref('0px')
let ro: ResizeObserver | undefined

onMounted(() => {
	ro = new ResizeObserver(([entry]) => {
		const { inlineSize, blockSize } = entry.borderBoxSize[0]
		console.log(inlineSize, blockSize)
		w.value = `${inlineSize}px`
		h.value = `${blockSize}px`
	})
	if (cornerEl.value) ro.observe(cornerEl.value)
})
onBeforeUnmount(() => ro?.disconnect())
</script>

<template>
	<div class="corner-wrap">
		<div ref="cornerEl" class="corner"><slot name="corner" /></div>
		<div class="spacer" aria-hidden="true" />
		<slot />
	</div>
</template>

<style scoped>
.corner-wrap {
	position: relative;
}
.corner {
	position: absolute;
	top: 0;
	right: 0;
}
.spacer {
	float: right;
	width: v-bind(w);
	height: v-bind(h);
	margin-left: v-bind('props.gap');
}
</style>
