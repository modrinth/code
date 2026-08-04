<template>
	<StackedAdmonitions v-bind="$attrs" :items="stackItems" class="w-full">
		<template #item="{ item, dismissible }">
			<InstanceAdmonitionsSharedInstanceStale
				v-if="item.kind === 'shared-instance-stale'"
				:instance="instance"
				@published="emit('published')"
			/>
			<InstanceAdmonitionsSharedInstanceUpdateAvailable
				v-else-if="item.kind === 'shared-instance-update-available'"
				:instance-name="instance.name"
				@review="emit('review-update', $event)"
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
import { StackedAdmonitions } from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import type { SharedInstanceUnavailableReason } from '@/helpers/install'
import type { GameInstance } from '@/helpers/types'

import InstanceAdmonitionsSharedInstanceStale from './instance-admonitions-shared-instance-stale.vue'
import InstanceAdmonitionsSharedInstanceUnavailable from './instance-admonitions-shared-instance-unavailable.vue'
import InstanceAdmonitionsSharedInstanceUpdateAvailable from './instance-admonitions-shared-instance-update-available.vue'
import InstanceAdmonitionsSharedInstanceWrongAccount from './instance-admonitions-shared-instance-wrong-account.vue'
import type { InstanceAdmonitionItem, SharedInstanceRole } from './types'

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
const showSharedInstanceUpdateAdmonition = computed(
	() =>
		!sharedInstanceWrongAccount.value &&
		!displayedSharedInstanceUnavailableReason.value &&
		props.instance.install_stage === 'installed' &&
		props.sharedInstanceRole === 'member' &&
		props.sharedInstanceUpdateAvailable === true,
)

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

	if (showSharedInstancePublishAdmonition.value) {
		items.push({
			id: 'shared-instance-stale',
			type: 'warning',
			dismissible: false,
			kind: 'shared-instance-stale',
		})
	}

	if (showSharedInstanceUpdateAdmonition.value) {
		items.push({
			id: 'shared-instance-update-available',
			type: 'info',
			dismissible: false,
			kind: 'shared-instance-update-available',
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
</script>
