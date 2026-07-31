<template>
	<Admonition
		type="info"
		inline-actions
		:header="formatMessage(messages.sharedInstanceChangesHeader)"
	>
		{{ formatMessage(messages.sharedInstanceChangesBody) }}
		<template #actions>
			<Button type="colored" color="blue" class="!h-10" :disabled="isPublishButtonDisabled" @click="reviewChanges">
				<SpinnerIcon
					v-if="isReviewingPublish || isPublishing"
					class="animate-spin"
					aria-hidden="true"
				/>
				<UploadIcon v-else aria-hidden="true" />
				{{
					isPublishing
						? formatMessage(messages.sharedInstancePublishingButton)
						: isReviewingPublish
							? formatMessage(messages.sharedInstanceReviewingButton)
							: formatMessage(messages.sharedInstancePublishButton)
				}}
			</Button>
		</template>
	</Admonition>

	<SharedInstancePublishModal
		ref="publishModal"
		:instance="instance"
		@published="emit('published')"
		@state-change="publishState = $event"
	/>
</template>

<script setup lang="ts">
import { Button } from '@modrinth/ui'
import { SpinnerIcon, UploadIcon } from '@modrinth/assets'
import { Admonition, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import SharedInstancePublishModal from '@/components/ui/shared-instances/SharedInstancePublishModal.vue'
import type { GameInstance } from '@/helpers/types'

import { instanceAdmonitionsMessages as messages } from './instance-admonitions-messages'

defineProps<{
	instance: GameInstance
}>()

const emit = defineEmits<{
	published: []
}>()

const { formatMessage } = useVIntl()
const publishModal = ref<InstanceType<typeof SharedInstancePublishModal>>()
const publishState = ref<'idle' | 'reviewing' | 'publishing'>('idle')
const isPublishing = computed(() => publishState.value === 'publishing')
const isReviewingPublish = computed(() => publishState.value === 'reviewing')

const isPublishButtonDisabled = computed(() => isPublishing.value || isReviewingPublish.value)

function reviewChanges(e?: MouseEvent) {
	publishModal.value?.show(e)
}
</script>
