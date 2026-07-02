<template>
	<StackedAdmonitions :items="stackItems" class="w-full">
		<template #item="{ item }">
			<Admonition
				v-if="item.kind === 'shared-instance-stale'"
				type="warning"
				inline-actions
				:header="formatMessage(messages.sharedInstanceChangesHeader)"
			>
				{{ formatMessage(messages.sharedInstanceChangesBody) }}
				<template #actions>
					<ButtonStyled color="orange">
						<button class="!h-10" :disabled="isPublishing" @click="publishSharedInstanceChanges">
							<UploadIcon aria-hidden="true" />
							{{
								isPublishing
									? formatMessage(messages.sharedInstancePublishingButton)
									: formatMessage(messages.sharedInstancePublishButton)
							}}
						</button>
					</ButtonStyled>
				</template>
			</Admonition>
		</template>
	</StackedAdmonitions>
</template>

<script setup lang="ts">
import { UploadIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	defineMessages,
	injectNotificationManager,
	type StackedAdmonitionItem,
	StackedAdmonitions,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { publish_shared_instance } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

type InstanceAdmonitionItem = StackedAdmonitionItem & {
	kind: 'shared-instance-stale'
}

const props = defineProps<{
	instance: GameInstance
}>()

const emit = defineEmits<{
	published: []
}>()

const messages = defineMessages({
	sharedInstanceChangesHeader: {
		id: 'app.instance.admonitions.shared-instance.changes-header',
		defaultMessage: "Your changes haven't been shared yet",
	},
	sharedInstanceChangesBody: {
		id: 'app.instance.admonitions.shared-instance.changes-body',
		defaultMessage: "Your local instance is ahead of the users you've shared it with.",
	},
	sharedInstancePublishButton: {
		id: 'app.instance.admonitions.shared-instance.publish-button',
		defaultMessage: 'Push update',
	},
	sharedInstancePublishingButton: {
		id: 'app.instance.admonitions.shared-instance.publishing-button',
		defaultMessage: 'Pushing...',
	},
})

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const isPublishing = ref(false)

const showSharedInstancePublishAdmonition = computed(
	() =>
		props.instance.shared_instance?.role === 'owner' &&
		props.instance.shared_instance.status === 'stale',
)

const stackItems = computed<InstanceAdmonitionItem[]>(() => {
	if (!showSharedInstancePublishAdmonition.value) return []

	return [
		{
			id: 'shared-instance-stale',
			type: 'warning',
			dismissible: false,
			kind: 'shared-instance-stale',
		},
	]
})

async function publishSharedInstanceChanges() {
	if (isPublishing.value) return

	isPublishing.value = true
	try {
		await publish_shared_instance(props.instance.id)
		emit('published')
	} catch (err) {
		handleError(err as Error)
	} finally {
		isPublishing.value = false
	}
}
</script>
