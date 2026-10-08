<template>
	<NewModal
		ref="modal"
		:header="header"
		width="min(34rem, calc(100vw - 2rem))"
		max-width="34rem"
		no-padding
		noblur
	>
		<InvitePlayersContent
			ref="content"
			v-bind="props"
			@invite="(payload) => emit('invite', payload)"
			@cancel="(user) => emit('cancel', user)"
			@copy-link="(link) => emit('copy-link', link)"
		/>
	</NewModal>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import NewModal from '../../modal/NewModal.vue'
import InvitePlayersContent from './invite-players-content.vue'
import type {
	InvitePlayersContentProps,
	InvitePlayersInvitePayload,
	InvitePlayersUser,
} from './types'

const props = withDefaults(defineProps<InvitePlayersContentProps & { header?: string }>(), {
	header: 'Share instance',
})

const emit = defineEmits<{
	invite: [payload: InvitePlayersInvitePayload]
	cancel: [user: InvitePlayersUser]
	'copy-link': [link: string]
}>()

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const content = ref<InstanceType<typeof InvitePlayersContent> | null>(null)

function show(event?: MouseEvent) {
	content.value?.resetSearch()
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
