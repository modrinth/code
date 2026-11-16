<script setup lang="ts">
import { ConfirmLeaveModal } from '@modrinth/ui'
import { ref } from 'vue'

defineProps<{
	username: string
	subject: 'owner' | 'user'
}>()

const modal = ref<InstanceType<typeof ConfirmLeaveModal> | null>(null)

defineExpose({ prompt: () => modal.value?.prompt() })
</script>

<template>
	<ConfirmLeaveModal
		ref="modal"
		title="Ban from shared instances?"
		:header="`Ban ${username} from shared instances`"
		:body="`This prevents the ${subject} from creating or managing shared instances. All of their shared instances will be quarantined, and members won't be able to launch them.`"
		stay-label="Cancel"
		:leave-label="`Ban ${subject}`"
		admonition-type="critical"
	/>
</template>
