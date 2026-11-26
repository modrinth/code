<script setup lang="ts">
import { onBeforeUnmount, ref, useId, watch } from 'vue'

const props = defineProps<{
	openByDefault?: boolean
	buttonClass: string
}>()

const open = ref(props.openByDefault ?? false)
const panelId = useId()
const panel = ref<HTMLElement>()
const content = ref<HTMLElement>()
let heightAnimation: Animation | undefined

watch(open, (expanded) => {
	const el = panel.value
	if (!el || !content.value) return

	const from = el.getBoundingClientRect().height
	heightAnimation?.cancel()
	heightAnimation = undefined
	const to = expanded ? content.value.getBoundingClientRect().height : 0
	if (
		from === to ||
		!el.getClientRects().length ||
		el.ownerDocument.defaultView?.matchMedia('(prefers-reduced-motion: reduce)').matches
	) {
		return
	}

	const animation = el.animate([{ height: `${from}px` }, { height: `${to}px` }], {
		duration: 300,
		easing: 'ease-in-out',
		fill: 'both',
	})
	heightAnimation = animation
	animation.onfinish = () => {
		if (heightAnimation !== animation) return
		animation.cancel()
		heightAnimation = undefined
	}
})

onBeforeUnmount(() => heightAnimation?.cancel())
</script>

<template>
	<div>
		<button
			type="button"
			:class="buttonClass"
			:aria-expanded="open"
			:aria-controls="panelId"
			@click="open = !open"
		>
			<slot name="button" :open="open" />
		</button>
		<div :id="panelId" ref="panel" class="overflow-clip" :class="{ 'h-0': !open }">
			<div ref="content" class="flow-root" :inert="!open">
				<slot />
			</div>
		</div>
	</div>
</template>
