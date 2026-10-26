<template>
	<StackedAdmonitions v-bind="$attrs" :items="stackItems" class="w-full">
		<template #item="{ item, dismissible }">
			<InstanceAdmonitionsSharedInstanceStale
				v-if="item.kind === 'shared-instance-stale'"
				:instance="instance"
				@published="emit('published')"
			/>
			<InstanceAdmonitionsSharedInstanceOwnerUpdate
				v-else-if="item.kind === 'shared-instance-owner-update'"
				@review-update="emit('review-update', $event)"
			/>
			<InstanceAdmonitionsSharedInstanceRestored
				v-else-if="item.kind === 'shared-instance-restored'"
				@dismiss="dismissRestoredNotice"
			/>
			<InstanceAdmonitionsSharedInstanceWrongAccount
				v-else-if="item.kind === 'shared-instance-wrong-account'"
				:expected-user-id="sharedInstanceExpectedUserId"
				:role="sharedInstanceRole"
				:signed-out="sharedInstanceSignedOut"
			/>
			<InstanceAdmonitionsSharedInstanceUnavailable
				v-else-if="item.kind === 'shared-instance-unavailable'"
				:reason="displayedSharedInstanceUnavailableReason"
				:manager="sharedInstanceUnavailableManager"
				:dismissible="dismissible"
				@dismiss="sharedInstanceUnavailableDismissed = true"
				@delete="emit('delete')"
			/>
		</template>
	</StackedAdmonitions>
</template>

<script setup lang="ts">
import { injectNotificationManager, StackedAdmonitions } from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import { toError } from '@/helpers/errors'
import type { SharedInstanceUnavailableReason } from '@/helpers/install'
import { dismiss_shared_instance_restored_notice } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

import InstanceAdmonitionsSharedInstanceOwnerUpdate from './shared-instance-owner-update.vue'
import InstanceAdmonitionsSharedInstanceRestored from './shared-instance-restored.vue'
import InstanceAdmonitionsSharedInstanceStale from './shared-instance-stale.vue'
import InstanceAdmonitionsSharedInstanceUnavailable from './shared-instance-unavailable.vue'
import InstanceAdmonitionsSharedInstanceWrongAccount from './shared-instance-wrong-account.vue'
import type { InstanceAdmonitionItem, SharedInstanceRole } from './types.ts'

defineOptions({
	inheritAttrs: false,
})

const props = defineProps<{
	instance: GameInstance
	sharedInstanceUnavailableReason?: SharedInstanceUnavailableReason | null
	sharedInstanceUnavailableManager?: string | null
	sharedInstanceWrongAccount?: boolean
	sharedInstanceExpectedUserId?: string | null
	sharedInstanceRole?: SharedInstanceRole | null
	sharedInstanceSignedOut?: boolean
	sharedInstanceUpdateAvailable?: boolean
}>()

const emit = defineEmits<{
	published: []
	delete: []
	'review-update': [event: MouseEvent]
}>()

const { handleError } = injectNotificationManager()

const sharedInstanceWrongAccount = computed(() => props.sharedInstanceWrongAccount ?? false)
const displayedSharedInstanceUnavailableReason = computed<SharedInstanceUnavailableReason | null>(
	() =>
		props.instance.quarantined ? 'quarantined' : (props.sharedInstanceUnavailableReason ?? null),
)
const sharedInstanceUnavailableDismissed = ref(false)
const showSharedInstancePublishAdmonition = computed(
	() =>
		!sharedInstanceWrongAccount.value &&
		props.instance.install_stage === 'installed' &&
		props.instance.shared_instance?.role === 'owner' &&
		props.instance.shared_instance.status === 'stale',
)
const showSharedInstanceOwnerUpdateAdmonition = computed(
	() =>
		!sharedInstanceWrongAccount.value &&
		props.instance.install_stage === 'installed' &&
		props.instance.shared_instance?.role === 'owner' &&
		(props.instance.shared_instance.status === 'update_available' ||
			props.sharedInstanceUpdateAvailable),
)
const restoredNoticeDismissed = ref(false)
const showSharedInstanceRestoredAdmonition = computed(
	() =>
		!restoredNoticeDismissed.value &&
		props.instance.install_stage === 'installed' &&
		props.instance.shared_instance?.role === 'owner' &&
		!!props.instance.shared_instance.restored_from_remote,
)

async function dismissRestoredNotice() {
	restoredNoticeDismissed.value = true
	await dismiss_shared_instance_restored_notice(props.instance.id).catch((error) => {
		restoredNoticeDismissed.value = false
		handleError(toError(error))
	})
}
const stackItems = computed<InstanceAdmonitionItem[]>(() => {
	const items: InstanceAdmonitionItem[] = []

	if (sharedInstanceWrongAccount.value) {
		items.push({
			id: 'shared-instance-wrong-account',
			type: 'warning',
			dismissible: false,
			kind: 'shared-instance-wrong-account',
		})
	}

	const unavailableReason = displayedSharedInstanceUnavailableReason.value
	const sharedInstanceQuarantined = unavailableReason === 'quarantined'
	if (
		unavailableReason &&
		(sharedInstanceQuarantined || !sharedInstanceUnavailableDismissed.value)
	) {
		items.push({
			id: 'shared-instance-unavailable',
			type: 'warning',
			dismissible: !sharedInstanceQuarantined,
			kind: 'shared-instance-unavailable',
		})
	}

	if (showSharedInstanceOwnerUpdateAdmonition.value) {
		items.push({
			id: 'shared-instance-owner-update',
			type: 'info',
			dismissible: false,
			kind: 'shared-instance-owner-update',
		})
	}

	if (showSharedInstancePublishAdmonition.value) {
		items.push({
			id: 'shared-instance-stale',
			type: 'warning',
			dismissible: false,
			kind: 'shared-instance-stale',
		})
	}

	if (showSharedInstanceRestoredAdmonition.value) {
		items.push({
			id: 'shared-instance-restored',
			type: 'info',
			dismissible: false,
			kind: 'shared-instance-restored',
		})
	}

	return items
})

watch(
	() => [props.instance.id, displayedSharedInstanceUnavailableReason.value],
	() => {
		sharedInstanceUnavailableDismissed.value = false
	},
)

watch(
	() => props.instance.id,
	() => {
		restoredNoticeDismissed.value = false
	},
)
</script>
