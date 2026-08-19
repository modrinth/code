<template>
	<NewModal ref="modal" :header="modalHeader" fade="danger" max-width="500px">
		<div class="flex flex-col gap-6">
			<Admonition type="critical" :header="formatMessage(messages.admonitionHeader)">
				{{ admonitionBody }}
			</Admonition>

			<div v-if="instances.length > 0" class="flex min-w-0 flex-col gap-2">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.instancesLabel, { count: instances.length }) }}
				</span>
				<div class="relative">
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-2"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-2"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showTopFade"
							class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-2 bg-gradient-to-b from-bg-raised to-transparent"
						/>
					</Transition>
					<div
						ref="instanceListRef"
						class="flex max-h-[240px] flex-col gap-2 overflow-y-auto"
						@scroll="checkScrollState"
					>
						<div
							v-for="instance in instances"
							:key="instance.id"
							class="flex min-w-0 items-center gap-2.5 rounded-[20px] border border-solid border-transparent bg-surface-2 p-3"
						>
							<Avatar
								:src="getInstanceIconUrl(instance.icon_path)"
								:tint-by="instance.id"
								size="40px"
								class="!rounded-xl"
								no-shadow
							/>
							<div class="flex min-w-0 flex-col gap-1.5">
								<span class="min-w-0 truncate font-semibold text-contrast">
									{{ instance.name }}
								</span>
								<span class="truncate text-sm font-medium text-secondary">
									{{ formatLoader(formatMessage, instance.loader) }} {{ instance.game_version }}
								</span>
							</div>
						</div>
					</div>
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-2"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-2"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showBottomFade"
							class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-2 bg-gradient-to-t from-bg-raised to-transparent"
						/>
					</Transition>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="red" @click="confirm">
					<TrashIcon />
					{{ deleteButtonLabel }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { TrashIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	commonMessages,
	defineMessages,
	formatLoader,
	NewModal,
	useScrollIndicator,
	useVIntl,
} from '@modrinth/ui'
import { computed, nextTick, ref } from 'vue'

import { getInstanceIconUrl } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'app.instance.confirm-delete.header',
		defaultMessage: 'Delete instance',
	},
	bulkHeader: {
		id: 'app.instance.confirm-delete.bulk-header',
		defaultMessage: 'Delete {count} instances',
	},
	admonitionHeader: {
		id: 'app.instance.confirm-delete.admonition-header',
		defaultMessage: 'This action cannot be undone',
	},
	admonitionBody: {
		id: 'app.instance.confirm-delete.admonition-body',
		defaultMessage:
			'All data for your instance will be permanently deleted, including your worlds, configs, and all installed content.',
	},
	bulkAdmonitionBody: {
		id: 'app.instance.confirm-delete.bulk-admonition-body',
		defaultMessage:
			'All data for these {count} instances will be permanently deleted, including their worlds, configs, and all installed content.',
	},
	instancesLabel: {
		id: 'app.instance.confirm-delete.instances-label',
		defaultMessage: '{count, plural, one {Instance} other {Instances ({count})}}',
	},
	deleteButton: {
		id: 'app.instance.confirm-delete.delete-button',
		defaultMessage: 'Delete instance',
	},
	bulkDeleteButton: {
		id: 'app.instance.confirm-delete.bulk-delete-button',
		defaultMessage: 'Delete {count} instances',
	},
})

const props = withDefaults(defineProps<{ instances?: GameInstance[] }>(), {
	instances: () => [],
})
const instances = computed(() => props.instances)
const count = computed(() => instances.value.length || 1)

const emit = defineEmits<{
	(e: 'delete'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const instanceListRef = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade, checkScrollState, forceCheck } =
	useScrollIndicator(instanceListRef)
const modalHeader = computed(() =>
	count.value === 1
		? formatMessage(messages.header)
		: formatMessage(messages.bulkHeader, { count: count.value }),
)
const admonitionBody = computed(() =>
	count.value === 1
		? formatMessage(messages.admonitionBody)
		: formatMessage(messages.bulkAdmonitionBody, { count: count.value }),
)
const deleteButtonLabel = computed(() =>
	count.value === 1
		? formatMessage(messages.deleteButton)
		: formatMessage(messages.bulkDeleteButton, { count: count.value }),
)

function show() {
	modal.value?.show()
	nextTick(() => forceCheck())
}

function confirm() {
	modal.value?.hide()
	emit('delete')
}

defineExpose({
	show,
})
</script>
