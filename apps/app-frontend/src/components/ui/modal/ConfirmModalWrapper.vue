<!-- @deprecated Use ConfirmModal from @modrinth/ui directly. Ads/noblur now handled by injectModalBehavior. -->
<script setup lang="ts">
import { ConfirmModal } from '@modrinth/ui'
import { ref } from 'vue'

defineProps({
	confirmationText: {
		type: String,
		default: '',
	},
	hasToType: {
		type: Boolean,
		default: false,
	},
	title: {
		type: String,
		default: 'No title defined',
		required: true,
	},
	description: {
		type: String,
		default: 'No description defined',
		required: true,
	},
	proceedIcon: {
		type: Object,
		default: undefined,
	},
	proceedLabel: {
		type: String,
		default: 'Proceed',
	},
	danger: {
		type: Boolean,
		default: true,
	},
	/** @deprecated No longer used — ads are handled by provideModalBehavior */
	showAdOnClose: {
		type: Boolean,
		default: true,
	},
	markdown: {
		type: Boolean,
		default: true,
	},
})

const emit = defineEmits(['proceed'])
const modal = ref(null)

defineExpose({
	show: () => {
		modal.value.show()
	},
	hide: () => {
		modal.value.hide()
	},
})

function proceed() {
	emit('proceed')
}
</script>

<template>
	<ConfirmModal
		ref="modal"
		:confirmation-text="confirmationText"
		:has-to-type="hasToType"
		:title="title"
		:description="description"
		:proceed-icon="proceedIcon"
		:proceed-label="proceedLabel"
		:danger="danger"
		:markdown="markdown"
		@proceed="proceed"
	/>
</template>
