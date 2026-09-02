<script setup lang="ts">
import type { CSSProperties } from 'vue'
import { computed, ref } from 'vue'

import type { AnchoredTeleportSide } from '../../../../utils/use-anchored-teleport'
import { buttonMenuPanelClasses } from './button-menu'

defineOptions({ inheritAttrs: false })

const props = defineProps<{
	open: boolean
	panelId: string
	label: string
	panelStyle: CSSProperties
	side?: AnchoredTeleportSide
	anchorStyle?: CSSProperties
	origin?: string
	expand?: 'vertical' | 'horizontal'
	// covers the gap so hovering from the trigger doesn't count as leaving
	bridge?: { side: AnchoredTeleportSide; size: number }
}>()

const element = ref<HTMLElement | null>(null)

const expandStyle = computed(() => {
	const origin = props.origin ?? 'top center'
	return {
		transformOrigin: origin,
		'--floating-expand-origin': origin,
		...(props.expand === 'horizontal'
			? {
					'--floating-expand-x': '0.3',
					'--floating-expand-y': '0.8',
				}
			: {}),
	}
})

defineExpose({ element })
</script>

<template>
	<Transition name="floating-expand">
		<div
			v-if="props.open"
			:id="props.panelId"
			ref="element"
			v-bind="$attrs"
			:class="buttonMenuPanelClasses"
			:style="[props.panelStyle, expandStyle]"
			role="menu"
			:aria-label="props.label"
		>
			<span
				v-if="props.side && props.anchorStyle"
				aria-hidden="true"
				class="button-menu-arrow"
				:data-side="props.side"
				:style="props.anchorStyle"
			/>
			<span
				v-if="props.bridge"
				aria-hidden="true"
				class="button-menu-bridge"
				:data-side="props.bridge.side"
				:style="{ '--button-menu-bridge-size': `${props.bridge.size}px` }"
			/>
			<div
				data-anchored-scroll-region
				class="flex flex-col p-2 overflow-y-auto"
				:style="{ maxHeight: props.panelStyle.maxHeight }"
			>
				<slot />
			</div>
		</div>
	</Transition>
</template>

<style scoped>
.button-menu-arrow {
	position: absolute;
	width: 0;
	height: 0;
	pointer-events: none;
}

.button-menu-arrow::before {
	position: absolute;
	width: 10px;
	height: 10px;
	content: '';
	background-color: var(--surface-3);
	transform: translate(-50%, -50%) rotate(45deg);
}

.button-menu-arrow[data-side='bottom'] {
	top: 0;
}

.button-menu-arrow[data-side='bottom']::before {
	border-top: 1px solid var(--surface-5);
	border-left: 1px solid var(--surface-5);
	border-radius: 0 0 99999px 0;
}

.button-menu-arrow[data-side='top'] {
	bottom: 0;
}

.button-menu-arrow[data-side='top']::before {
	border-right: 1px solid var(--surface-5);
	border-bottom: 1px solid var(--surface-5);
}

.button-menu-arrow[data-side='right'] {
	left: 0;
}

.button-menu-arrow[data-side='right']::before {
	border-bottom: 1px solid var(--surface-5);
	border-left: 1px solid var(--surface-5);
}

.button-menu-arrow[data-side='left'] {
	right: 0;
}

.button-menu-arrow[data-side='left']::before {
	border-top: 1px solid var(--surface-5);
	border-right: 1px solid var(--surface-5);
}

.button-menu-bridge {
	position: absolute;
}

.button-menu-bridge[data-side='right'] {
	top: 0;
	bottom: 0;
	left: calc(-1 * var(--button-menu-bridge-size));
	width: var(--button-menu-bridge-size);
}

.button-menu-bridge[data-side='left'] {
	top: 0;
	bottom: 0;
	right: calc(-1 * var(--button-menu-bridge-size));
	width: var(--button-menu-bridge-size);
}

.button-menu-bridge[data-side='bottom'] {
	left: 0;
	right: 0;
	top: calc(-1 * var(--button-menu-bridge-size));
	height: var(--button-menu-bridge-size);
}

.button-menu-bridge[data-side='top'] {
	left: 0;
	right: 0;
	bottom: calc(-1 * var(--button-menu-bridge-size));
	height: var(--button-menu-bridge-size);
}
</style>
