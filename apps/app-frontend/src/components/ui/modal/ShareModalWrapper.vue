<!-- @deprecated Use ShareModal from @modrinth/ui directly. Ads/noblur now handled by injectModalBehavior. -->
<script setup lang="ts">
import { ShareModal } from '@modrinth/ui'
import { ref } from 'vue'

defineProps({
	header: {
		type: String,
		default: 'Share',
	},
	shareTitle: {
		type: String,
		default: 'Modrinth',
	},
	shareText: {
		type: String,
		default: null,
	},
	link: {
		type: Boolean,
		default: false,
	},
	openInNewTab: {
		type: Boolean,
		default: true,
	},
})

const modal = ref(null)

defineExpose({
	show: (passedContent) => {
		modal.value.show(passedContent)
	},
	hide: () => {
		modal.value.hide()
	},
})
</script>

<template>
	<ShareModal
		ref="modal"
		:header="header"
		:share-title="shareTitle"
		:share-text="shareText"
		:link="link"
		:open-in-new-tab="openInNewTab"
	/>
</template>
