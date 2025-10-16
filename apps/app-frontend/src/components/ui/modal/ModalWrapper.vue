<script setup lang="ts">
import { NewModal as Modal } from '@modrinth/ui'
import { useTemplateRef } from 'vue'

import { hide_ads_window, show_ads_window } from '@/helpers/ads.js'
import { useTheming } from '@/store/theme.ts'

const themeStore = useTheming()

const props = defineProps({
	header: {
		type: String,
		default: null,
	},
	hideHeader: {
		type: Boolean,
		default: false,
	},
	closable: {
		type: Boolean,
		default: true,
	},
	onHide: {
		type: Function,
		default() {
			return () => {}
		},
	},
	showAdOnClose: {
		type: Boolean,
		default: true,
	},
})
const modal = useTemplateRef('modal')

defineExpose({
	show: (e: MouseEvent) => {
		hide_ads_window()
		modal.value?.show(e)
	},
	hide: () => {
		onModalHide()
		modal.value?.hide()
	},
})

function onModalHide() {
	if (props.showAdOnClose) {
		show_ads_window()
	}
	props.onHide?.()
}
</script>

<template>
	<Modal
		ref="modal"
		:header="header"
		:noblur="!themeStore.advancedRendering"
		:closable="closable"
		:hide-header="hideHeader"
		@hide="onModalHide"
	>
		<template #title>
			<slot name="title" />
		</template>
		<slot />
	</Modal>
</template>
