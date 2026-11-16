<template>
	<CollapsibleAdmonition
		v-if="reason === 'quarantined'"
		v-model="expanded"
		type="warning"
		:header="title"
	>
		<div class="flex flex-col gap-2 bg-bg-orange pb-4 pl-12 pr-4 leading-tight text-contrast/85">
			<span>{{ formatSharedInstanceUnavailable(reason, manager, role) }}</span>
			<!-- eslint-disable-next-line vue/no-v-html -->
			<div
				v-if="showLockReason"
				class="markdown-body break-words rounded-xl bg-black/20 px-4 py-3 [&>:first-child]:mt-0 [&>:last-child]:mb-0"
				v-html="renderedLockReason"
			/>
			<div v-if="showDeleteAction" class="mt-2">
				<Button type="colored" color="orange" size="lg" @click="emit('delete')">
					<TrashIcon aria-hidden="true" />
					{{ formatMessage(messages.deleteInstance) }}
				</Button>
			</div>
		</div>
	</CollapsibleAdmonition>
	<Admonition
		v-else
		type="warning"
		:header="title"
		:dismissible="dismissible"
		@dismiss="emit('dismiss')"
	>
		{{ formatSharedInstanceUnavailable(reason ?? null, manager, role) }}
	</Admonition>
</template>

<script setup lang="ts">
import { TrashIcon } from '@modrinth/assets'
import { Admonition, Button, CollapsibleAdmonition, defineMessages, useVIntl } from '@modrinth/ui'
import { renderString } from '@modrinth/utils/parse'
import { computed, ref } from 'vue'

import type { SharedInstanceUnavailableReason } from '@/helpers/install'
import {
	sharedInstanceUnavailableTitleMessage,
	useSharedInstanceErrors,
} from '@/helpers/shared-instance-errors'

import type { SharedInstanceRole } from './types'

const props = defineProps<{
	reason?: SharedInstanceUnavailableReason | null
	manager?: string | null
	role?: SharedInstanceRole | null
	lockReason?: string | null
	dismissible?: boolean
}>()

const emit = defineEmits<{
	dismiss: []
	delete: []
}>()

const { formatMessage } = useVIntl()
const { formatSharedInstanceUnavailable } = useSharedInstanceErrors()

const expanded = ref(true)
const title = computed(() =>
	formatMessage(sharedInstanceUnavailableTitleMessage(props.reason ?? null, props.role)),
)
const isOwnerLock = computed(() => props.reason === 'quarantined' && props.role === 'owner')
const showDeleteAction = computed(() => props.reason === 'quarantined' && !isOwnerLock.value)
const showLockReason = computed(() => isOwnerLock.value && !!props.lockReason)
const renderedLockReason = computed(() => renderString(props.lockReason ?? ''))

const messages = defineMessages({
	deleteInstance: {
		id: 'instance.locked.delete-button',
		defaultMessage: 'Delete instance',
	},
})
</script>
