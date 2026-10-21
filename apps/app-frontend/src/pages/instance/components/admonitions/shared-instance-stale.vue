<template>
	<Admonition
		type="info"
		inline-actions
		:header="formatMessage(messages.sharedInstanceChangesHeader)"
		:progress="isPublishing ? publishProgress : undefined"
		:waiting="isPublishing && !publishTask?.totalBytes"
	>
		{{ formatMessage(messages.sharedInstanceChangesBody) }}
		<template #actions>
			<Button
				type="colored"
				color="blue"
				size="lg"
				:disabled="isPublishButtonDisabled"
				@click="reviewChanges"
			>
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
import { SpinnerIcon, UploadIcon } from '@modrinth/assets'
import { Admonition, Button, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { findRunningSharedInstancePublish } from '@/components/ui/download-manager/shared-instance-publishes'
import SharedInstancePublishModal from '@/components/ui/shared-instances/SharedInstancePublishModal.vue'
import type { GameInstance } from '@/helpers/types'

import { instanceAdmonitionsMessages as messages } from './messages'

const props = defineProps<{
	instance: GameInstance
}>()

const emit = defineEmits<{
	published: []
}>()

const { formatMessage } = useVIntl()
const publishModal = ref<InstanceType<typeof SharedInstancePublishModal>>()
const publishState = ref<'idle' | 'reviewing' | 'publishing'>('idle')
const publishTask = computed(() => findRunningSharedInstancePublish(props.instance.id))
const isPublishing = computed(() => publishState.value === 'publishing' || !!publishTask.value)
const publishProgress = computed(() =>
	publishTask.value?.totalBytes
		? publishTask.value.uploadedBytes / publishTask.value.totalBytes
		: 0,
)
const isReviewingPublish = computed(() => publishState.value === 'reviewing')

const isPublishButtonDisabled = computed(() => isPublishing.value || isReviewingPublish.value)

function reviewChanges(e?: MouseEvent) {
	publishModal.value?.show(e)
}
</script>
