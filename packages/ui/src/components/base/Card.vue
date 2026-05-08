<script setup lang="ts">
import { DropdownIcon } from '@modrinth/assets'
import { reactive } from 'vue'

import ButtonStyled from './ButtonStyled.vue'

const props = defineProps({
	collapsible: {
		type: Boolean,
		default: false,
	},
	defaultCollapsed: {
		type: Boolean,
		default: false,
	},
	noAutoBody: {
		type: Boolean,
		default: false,
	},
})

const state = reactive({
	collapsed: props.defaultCollapsed,
})

function toggleCollapsed() {
	state.collapsed = !state.collapsed
}
</script>

<template>
	<div class="card">
		<div v-if="!!$slots.header || collapsible" class="header">
			<slot name="header"></slot>
			<div v-if="collapsible" class="btn-group">
				<ButtonStyled circular>
					<button @click="toggleCollapsed">
						<DropdownIcon :style="{ transform: `rotate(${state.collapsed ? 0 : 180}deg)` }" />
					</button>
				</ButtonStyled>
			</div>
		</div>
		<slot v-if="!state.collapsed" />
	</div>
</template>

<style lang="scss" scoped>
.header {
	display: flex;

	:deep(h1, h2, h3, h4) {
		margin-block: 0;
	}

	&:not(:last-child) {
		margin-bottom: var(--gap-lg);
	}
}

.btn-group {
	margin-left: auto;
	margin-right: 0;
}
</style>
