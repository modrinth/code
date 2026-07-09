<template>
	<StackedAdmonitions v-bind="$attrs" :items="stackItems" class="w-full">
		<template #item="{ item, dismissible }">
			<InstanceAdmonitionsSharedInstanceStale
				v-if="item.kind === 'shared-instance-stale'"
				:instance="instance"
				@published="emit('published')"
			/>
			<InstanceAdmonitionsSharedInstanceWrongAccount
				v-else-if="item.kind === 'shared-instance-wrong-account'"
				:expected-user-id="sharedInstanceExpectedUserId"
				:role="sharedInstanceRole"
				:signed-out="sharedInstanceSignedOut"
			/>
			<InstanceAdmonitionsSharedInstanceUnavailable
				v-else-if="item.kind === 'shared-instance-unavailable'"
				:reason="sharedInstanceUnavailableReason"
				:manager="sharedInstanceUnavailableManager"
				:dismissible="dismissible"
				@dismiss="sharedInstanceUnavailableDismissed = true"
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
}>()

const emit = defineEmits<{
	published: []
}>()

const sharedInstanceWrongAccount = computed(() => props.sharedInstanceWrongAccount ?? false)
const sharedInstanceUnavailableDismissed = ref(false)
const showSharedInstancePublishAdmonition = computed(
	() =>
		!sharedInstanceWrongAccount.value &&
		props.instance.shared_instance?.role === 'owner' &&
		props.instance.shared_instance.status === 'stale',
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

	if (props.sharedInstanceUnavailableReason && !sharedInstanceUnavailableDismissed.value) {
		items.push({
			id: 'shared-instance-unavailable',
			type: 'warning',
			dismissible: true,
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

	return items
})

watch(
	() => [props.instance.id, props.sharedInstanceUnavailableReason],
	() => {
		sharedInstanceUnavailableDismissed.value = false
	},
)
</script>
