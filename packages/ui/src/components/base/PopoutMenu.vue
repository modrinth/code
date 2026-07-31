<template>
	<Dropdown
		ref="dropdown"
		no-auto-focus
		:aria-id="dropdownId || null"
		:placement="placement"
		:class="dropdownClass"
		@apply-hide="focusTrigger"
	>
		<Button ref="trigger" v-bind="$attrs" v-tooltip="tooltip">
			<slot></slot>
		</Button>
		<template #popper="{ hide: hideFunction }">
			<button class="dummy-button" @focusin="hideAndFocusTrigger(hideFunction)"></button>
			<div ref="menu" class="contents">
				<slot name="menu"> </slot>
			</div>
			<button class="dummy-button" @focusin="hideAndFocusTrigger(hideFunction)"></button>
		</template>
	</Dropdown>
</template>

<script setup lang="ts">
import { Dropdown } from 'floating-vue'
import { ref } from 'vue'

import Button from './buttons/Button.vue'
import type { ButtonElementHandle } from './buttons/types'

const trigger = ref<ButtonElementHandle | null>(null)
const menu = ref()
const dropdown = ref()

defineProps({
	dropdownId: {
		type: String,
		default: null,
		required: false,
	},
	dropdownClass: {
		type: String,
		default: null,
		required: false,
	},
	tooltip: {
		type: String,
		default: null,
		required: false,
	},
	placement: {
		type: String,
		default: 'bottom-end',
		required: false,
	},
})

function hideAndFocusTrigger(hide) {
	hide()
	focusTrigger()
}

function focusTrigger() {
	trigger.value?.element?.focus()
}

defineOptions({
	inheritAttrs: false,
})

function hide() {
	dropdown.value.hide()
}

function show() {
	dropdown.value.show()
}

defineExpose({
	show,
	hide,
})
</script>
<style scoped>
.dummy-button {
	position: absolute;
	width: 0;
	height: 0;
	margin: 0;
	padding: 0;
	border: none;
	overflow: hidden;
	clip: rect(0 0 0 0);
	white-space: nowrap;
	outline: none;
}
</style>
