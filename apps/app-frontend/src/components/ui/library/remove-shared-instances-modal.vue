<template>
	<NewModal
		ref="modal"
		:header="formatMessage(copy.header, values)"
		fade="warning"
		max-width="500px"
	>
		<Admonition type="warning" :header="formatMessage(copy.admonitionHeader, values)">{{
			formatMessage(copy.body, values)
		}}</Admonition>
		<template #actions
			><div class="flex justify-end gap-2">
				<Button type="outlined" class="!border" @click="hide">
					<XIcon />{{ formatMessage(commonMessages.cancelButton) }} </Button
				><Button type="colored" color="orange" @click="confirm">
					<LogOutIcon v-if="mode === 'leave'" /><UnlinkIcon v-else />{{
						formatMessage(sharedInstanceRemovalButtonMessages[mode])
					}}
				</Button>
			</div></template
		>
	</NewModal>
</template>

<script setup lang="ts">
import { LogOutIcon, UnlinkIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	Button,
	commonMessages,
	defineMessages,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import {
	getSharedInstanceRemovalMode,
	sharedInstanceRemovalButtonMessages,
} from '@/components/ui/library/use-library'
import type { UninstalledSharedInstance } from '@/components/ui/library/use-uninstalled-shared-instances'

const { formatMessage } = useVIntl()
const modal = ref<InstanceType<typeof NewModal>>()
const onConfirm = ref<() => void | Promise<void>>(() => {})
const sharedInstances = ref<UninstalledSharedInstance[]>([])

const mode = computed(() => getSharedInstanceRemovalMode(sharedInstances.value))
const values = computed(() => {
	const unpublishCount = sharedInstances.value.filter(
		(instance) => instance.role === 'owner',
	).length
	return {
		count: sharedInstances.value.length,
		unpublishCount,
		leaveCount: sharedInstances.value.length - unpublishCount,
	}
})

function show(
	sharedInstancesValue: UninstalledSharedInstance[],
	confirmValue: () => void | Promise<void>,
	event?: MouseEvent,
) {
	sharedInstances.value = sharedInstancesValue
	onConfirm.value = confirmValue
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

async function confirm() {
	hide()
	await onConfirm.value()
}

const messages = defineMessages({
	unpublishHeader: {
		id: 'app.library.unpublish-shared-instance.modal.header',
		defaultMessage:
			'{count, plural, one {Unpublish shared instance} other {Unpublish # shared instances}}',
	},
	unpublishAdmonitionHeader: {
		id: 'app.library.unpublish-shared-instance.modal.admonition-header',
		defaultMessage:
			'{count, plural, one {Unpublishing shared instance} other {Unpublishing # shared instances}}',
	},
	unpublishBody: {
		id: 'app.library.unpublish-shared-instance.modal.admonition-body',
		defaultMessage:
			"{count, plural, one {This deletes the shared instance from Modrinth's servers. People using it in the Modrinth App will stop receiving updates, and it will be removed from your library.} other {This deletes these shared instances from Modrinth's servers. People using them in the Modrinth App will stop receiving updates, and they will be removed from your library.}}",
	},
	leaveHeader: {
		id: 'app.library.leave-shared-instance.modal.header',
		defaultMessage: '{count, plural, one {Leave shared instance} other {Leave # shared instances}}',
	},
	leaveAdmonitionHeader: {
		id: 'app.library.leave-shared-instance.modal.admonition-header',
		defaultMessage:
			'{count, plural, one {Leaving shared instance} other {Leaving # shared instances}}',
	},
	leaveBody: {
		id: 'app.library.leave-shared-instance.modal.admonition-body',
		defaultMessage:
			"{count, plural, one {You'll lose access to this shared instance and it will be removed from your library. You'll need a new invite to join it again.} other {You'll lose access to these shared instances and they will be removed from your library. You'll need a new invite to join them again.}}",
	},
	unpublishAndLeaveHeader: {
		id: 'app.library.unpublish-and-leave-shared-instances.modal.header',
		defaultMessage: 'Unpublish and leave {count} shared instances',
	},
	unpublishAndLeaveAdmonitionHeader: {
		id: 'app.library.unpublish-and-leave-shared-instances.modal.admonition-header',
		defaultMessage: 'Unpublishing and leaving shared instances',
	},
	unpublishAndLeaveBody: {
		id: 'app.library.unpublish-and-leave-shared-instances.modal.admonition-body',
		defaultMessage:
			"{unpublishCount, plural, one {The # shared instance you own will be deleted from Modrinth's servers, and people using it} other {The # shared instances you own will be deleted from Modrinth's servers, and people using them}} will stop receiving updates. You'll lose access to the other {leaveCount} and need a new invite to rejoin. All of them will be removed from your library.",
	},
})

const copy = computed(
	() =>
		({
			unpublish: {
				header: messages.unpublishHeader,
				admonitionHeader: messages.unpublishAdmonitionHeader,
				body: messages.unpublishBody,
			},
			leave: {
				header: messages.leaveHeader,
				admonitionHeader: messages.leaveAdmonitionHeader,
				body: messages.leaveBody,
			},
			unpublish_and_leave: {
				header: messages.unpublishAndLeaveHeader,
				admonitionHeader: messages.unpublishAndLeaveAdmonitionHeader,
				body: messages.unpublishAndLeaveBody,
			},
		})[mode.value],
)

defineExpose({ show, hide })
</script>
