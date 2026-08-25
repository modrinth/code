<script setup lang="ts">
import type { CSSProperties } from 'vue'
import { computed, ref } from 'vue'

import type { AnchoredTeleportSide } from '../../../utils/use-anchored-teleport'
import { overflowMenuPanelClasses } from './overflow-menu'

defineOptions({ inheritAttrs: false })

const props = defineProps<{
	open: boolean
	panelId: string
	label: string
	panelStyle: CSSProperties
	side?: AnchoredTeleportSide
	anchorStyle?: CSSProperties
	origin?: string
	// covers the gap so hovering from the trigger doesn't count as leaving
	bridge?: { side: AnchoredTeleportSide; size: number }
}>()

const element = ref<HTMLElement | null>(null)

const transformOrigin = computed(() => {
	switch (props.side) {
		case 'top':
			return 'bottom center'
		case 'left':
			return 'right center'
		case 'right':
			return 'left center'
		case 'bottom':
			return 'top center'
		default:
			return props.origin
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
			:class="overflowMenuPanelClasses"
			:style="[props.panelStyle, { transformOrigin }]"
			role="menu"
			:aria-label="props.label"
		>
			<span
				v-if="props.side && props.anchorStyle"
				aria-hidden="true"
				class="overflow-menu-arrow"
				:data-side="props.side"
				:style="props.anchorStyle"
			/>
			<span
				v-if="props.bridge"
				aria-hidden="true"
				class="overflow-menu-bridge"
				:data-side="props.bridge.side"
				:style="{ '--overflow-menu-bridge-size': `${props.bridge.size}px` }"
			/>
			<div
				data-anchored-scroll-region
				class="flex min-w-48 flex-col p-2 overflow-y-auto"
				:style="{ maxHeight: props.panelStyle.maxHeight }"
			>
				<slot />
			</div>
		</div>
	</Transition>
</template>

<style scoped>
.overflow-menu-arrow {
	position: absolute;
	width: 0;
	height: 0;
	pointer-events: none;
}

.overflow-menu-arrow::before {
	position: absolute;
	width: 10px;
	height: 10px;
	content: '';
	background-color: var(--surface-3);
	transform: translate(-50%, -50%) rotate(45deg);
}

.overflow-menu-arrow[data-side='bottom'] {
	top: 0;
}

.overflow-menu-arrow[data-side='bottom']::before {
	border-top: 1px solid var(--surface-5);
	border-left: 1px solid var(--surface-5);
	border-radius: 0 0 99999px 0;
}

.overflow-menu-arrow[data-side='top'] {
	bottom: 0;
}

.overflow-menu-arrow[data-side='top']::before {
	border-right: 1px solid var(--surface-5);
	border-bottom: 1px solid var(--surface-5);
}

.overflow-menu-arrow[data-side='right'] {
	left: 0;
}

.overflow-menu-arrow[data-side='right']::before {
	border-bottom: 1px solid var(--surface-5);
	border-left: 1px solid var(--surface-5);
}

.overflow-menu-arrow[data-side='left'] {
	right: 0;
}

.overflow-menu-arrow[data-side='left']::before {
	border-top: 1px solid var(--surface-5);
	border-right: 1px solid var(--surface-5);
}

.overflow-menu-bridge {
	position: absolute;
}

.overflow-menu-bridge[data-side='right'] {
	top: 0;
	bottom: 0;
	left: calc(-1 * var(--overflow-menu-bridge-size));
	width: var(--overflow-menu-bridge-size);
}

.overflow-menu-bridge[data-side='left'] {
	top: 0;
	bottom: 0;
	right: calc(-1 * var(--overflow-menu-bridge-size));
	width: var(--overflow-menu-bridge-size);
}

.overflow-menu-bridge[data-side='bottom'] {
	left: 0;
	right: 0;
	top: calc(-1 * var(--overflow-menu-bridge-size));
	height: var(--overflow-menu-bridge-size);
}

.overflow-menu-bridge[data-side='top'] {
	left: 0;
	right: 0;
	bottom: calc(-1 * var(--overflow-menu-bridge-size));
	height: var(--overflow-menu-bridge-size);
}
</style>
