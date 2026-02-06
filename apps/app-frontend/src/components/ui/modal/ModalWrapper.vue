<!-- @deprecated Use NewModal from @modrinth/ui directly. Ads/noblur now handled by injectModalBehavior. -->
<script setup lang="ts">
import { NewModal as Modal } from '@modrinth/ui'
import { useTemplateRef } from 'vue'

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
	/** @deprecated No longer used — ads are handled by provideModalBehavior */
	showAdOnClose: {
		type: Boolean,
		default: true,
	},
})
const modal = useTemplateRef('modal')

defineExpose({
	show: (e: MouseEvent) => {
		modal.value?.show(e)
	},
	hide: () => {
		modal.value?.hide()
	},
})
</script>

<template>
	<Modal
		ref="modal"
		:header="header"
		:closable="closable"
		:hide-header="hideHeader"
		:on-hide="() => props.onHide?.()"
	>
		<template #title>
			<slot name="title" />
		</template>
		<slot />
	</Modal>
</template>
