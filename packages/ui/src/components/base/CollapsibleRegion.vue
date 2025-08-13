<template>
	<div
		class="relative overflow-hidden rounded-xl border-[2px] border-solid border-divider shadow-lg"
		:class="{ 'max-h-32': isCollapsed }"
	>
		<div
			class="px-4 pt-4"
			:class="{
				'content-disabled pb-16': isCollapsed,
				'pb-4': !isCollapsed,
			}"
		>
			<slot />
		</div>

		<div
			v-if="isCollapsed"
			class="pointer-events-none absolute inset-0 bg-gradient-to-b from-transparent to-button-bg"
		></div>

		<div class="absolute bottom-4 left-1/2 z-20 -translate-x-1/2">
			<ButtonStyled circular type="transparent">
				<button class="flex items-center gap-1 text-xs" @click="toggleCollapsed">
					<ExpandIcon v-if="isCollapsed" />
					<CollapseIcon v-else />
					{{ isCollapsed ? expandText : collapseText }}
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CollapseIcon, ExpandIcon } from '@modrinth/assets'
import { ref } from 'vue'

import ButtonStyled from './ButtonStyled.vue'

const props = withDefaults(
	defineProps<{
		initiallyCollapsed?: boolean
		expandText?: string
		collapseText?: string
	}>(),
	{
		initiallyCollapsed: true,
		expandText: 'Expand',
		collapseText: 'Collapse',
	},
)

const isCollapsed = ref(props.initiallyCollapsed)

function toggleCollapsed() {
	isCollapsed.value = !isCollapsed.value
}

function setCollapsed(value: boolean) {
	isCollapsed.value = value
}

defineExpose({
	isCollapsed,
	setCollapsed,
	toggleCollapsed,
})
</script>

<style lang="scss" scoped>
.content-disabled {
	pointer-events: none;
	user-select: none;
	-webkit-user-select: none;
	-moz-user-select: none;
	-ms-user-select: none;

	:deep(*) {
		pointer-events: none !important;
		user-select: none !important;
		-webkit-user-select: none !important;
		-moz-user-select: none !important;
		-ms-user-select: none !important;
	}

	:deep(button),
	:deep(input),
	:deep(textarea),
	:deep(select),
	:deep(a),
	:deep([tabindex]) {
		tabindex: -1 !important;
	}

	:deep(*:focus) {
		outline: none !important;
	}
}
</style>
