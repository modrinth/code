<template>
	<div
		class="grid min-h-20 grid-cols-[auto_minmax(0,3fr)_minmax(0,4fr)_auto] items-center gap-2 rounded-[20px] border border-solid border-surface-4 bg-bg-raised p-3"
	>
		<Avatar size="48px" no-shadow class="!rounded-[14px]" :tint-by="server.server_id" />
		<div class="flex h-full flex-col justify-center gap-0.5">
			<div class="flex items-center gap-1.5">
				<div class="truncate text-base font-semibold text-contrast">{{ server.name }}</div>
				<TagItem
					class="border !border-solid border-blue bg-highlight-blue text-xs"
					:style="`--_color: var(--color-blue)`"
				>
					<Link2Icon aria-hidden="true" />
					{{ formatMessage(messages.linked) }}
				</TagItem>
			</div>
			<div class="flex items-center gap-1 text-sm font-semibold" :class="stateDisplay.class">
				<component
					:is="stateDisplay.icon"
					aria-hidden="true"
					class="size-4 shrink-0"
					stroke-width="3px"
				/>
				{{ formatMessage(stateDisplay.message) }}
			</div>
		</div>
		<div class="truncate text-center text-sm text-secondary">
			<template v-if="state === 'failed' && server.linked_instance.error">
				{{ server.linked_instance.error }}
			</template>
			<template v-else-if="state === 'pending'">
				{{ formatMessage(messages.pendingDetail) }}
			</template>
			<button
				v-else
				v-tooltip="formatMessage(messages.copyAddress)"
				class="cursor-pointer border-none bg-transparent p-0 font-minecraft text-secondary hover:underline"
				@click="copyToClipboard(address)"
			>
				{{ address }}
			</button>
		</div>
		<div class="flex justify-end gap-1">
			<Button
				v-if="state === 'running' || state === 'pending'"
				type="colored"
				color="brand"
				:disabled="playDisabled"
				@click="emit('play')"
			>
				<PlayIcon aria-hidden="true" />
				{{ formatMessage(commonMessages.playButton) }}
			</Button>
			<ButtonLink type="outlined" class="!border" :to="`/hosting/manage/${server.server_id}`">
				<SettingsIcon aria-hidden="true" />
				{{ formatMessage(messages.manage) }}
			</ButtonLink>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	ClockIcon,
	IssuesIcon,
	Link2Icon,
	NoSignalIcon,
	PlayIcon,
	SettingsIcon,
	SignalIcon,
	UpdatedIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	ButtonLink,
	commonMessages,
	defineMessages,
	type MessageDescriptor,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { type Component, computed } from 'vue'

import {
	type LinkedServer,
	linkedServerState,
	type LinkedServerState,
	serverAddress,
} from '@/helpers/linked-servers'
import { copyToClipboard } from '@/helpers/utils'

/** A server on the user's Core that follows this instance, as a row in Instance > Worlds. */
const props = defineProps<{
	server: LinkedServer
	playDisabled?: boolean
}>()

const emit = defineEmits<{
	play: []
}>()

const { formatMessage } = useVIntl()

const state = computed(() => linkedServerState(props.server))
const address = computed(() => serverAddress(props.server))

const messages = defineMessages({
	linked: {
		id: 'instance.linked-servers.item.linked',
		defaultMessage: 'Linked',
	},
	manage: {
		id: 'instance.linked-servers.item.manage',
		defaultMessage: 'Manage',
	},
	copyAddress: {
		id: 'instance.linked-servers.item.copy-address',
		defaultMessage: 'Copy address',
	},
	running: {
		id: 'instance.linked-servers.item.state.running',
		defaultMessage: 'Running',
	},
	offline: {
		id: 'instance.linked-servers.item.state.offline',
		defaultMessage: 'Offline',
	},
	updating: {
		id: 'instance.linked-servers.item.state.updating',
		defaultMessage: 'Updating',
	},
	pending: {
		id: 'instance.linked-servers.item.state.pending',
		defaultMessage: 'Running, update waiting',
	},
	failed: {
		id: 'instance.linked-servers.item.state.failed',
		defaultMessage: 'Update failed',
	},
	pendingDetail: {
		id: 'instance.linked-servers.item.pending-detail',
		defaultMessage: 'The update applies when the server stops',
	},
})

const stateDisplays: Record<
	LinkedServerState,
	{ icon: Component; message: MessageDescriptor; class: string }
> = {
	running: { icon: SignalIcon, message: messages.running, class: 'text-green' },
	offline: { icon: NoSignalIcon, message: messages.offline, class: 'text-secondary' },
	updating: { icon: UpdatedIcon, message: messages.updating, class: 'text-blue' },
	pending: { icon: ClockIcon, message: messages.pending, class: 'text-orange' },
	failed: { icon: IssuesIcon, message: messages.failed, class: 'text-red' },
}
const stateDisplay = computed(() => stateDisplays[state.value])
</script>
