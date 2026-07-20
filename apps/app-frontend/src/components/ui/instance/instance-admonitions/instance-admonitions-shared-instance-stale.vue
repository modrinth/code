<template>
	<Admonition
		type="warning"
		inline-actions
		:header="formatMessage(messages.sharedInstanceChangesHeader)"
	>
		{{ formatMessage(messages.sharedInstanceChangesBody) }}
		<template #actions>
			<ButtonStyled color="orange">
				<button class="!h-10" :disabled="isPublishButtonDisabled" @click="reviewChanges">
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
				</button>
			</ButtonStyled>
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
import { SpinnerIcon, UploadIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import SharedInstancePublishModal from '@/components/ui/shared-instances/SharedInstancePublishModal.vue'
import type { GameInstance } from '@/helpers/types'

import { instanceAdmonitionsMessages as messages } from './instance-admonitions-messages'

const props = defineProps<{
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
