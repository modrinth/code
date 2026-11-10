<script setup lang="ts">
import { LockIcon, SaveIcon, XIcon } from '@modrinth/assets'
import { sharedInstanceQuarantineReasons } from '@modrinth/moderation'
import {
	Avatar,
	Button,
	Combobox,
	type ComboboxOption,
	MarkdownEditor,
	NewModal,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { useSharedInstanceModeration } from '~/composables/shared-instance-moderation'

const CUSTOM_REASON = -1
const QUARANTINE_REASON_MAX_LENGTH = 2000

const props = defineProps<{
	instance: { id: string; name: string; icon_url?: string | null }
}>()

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const editing = ref(false)
const selectedReason = ref(CUSTOM_REASON)
const message = ref('')
const { pending, quarantine } = useSharedInstanceModeration()

const reasonOptions: ComboboxOption<number>[] = [
	...sharedInstanceQuarantineReasons.map((reason, index) => ({
		value: index,
		label: reason.label,
	})),
	{ value: CUSTOM_REASON, label: 'Custom message' },
]

const trimmedMessage = computed(() => message.value.trim())
const messageTooLong = computed(() => trimmedMessage.value.length > QUARANTINE_REASON_MAX_LENGTH)

async function selectReason(index: number) {
	selectedReason.value = index
	const reason = sharedInstanceQuarantineReasons[index]
	if (!reason) return
	message.value =
		typeof reason.message === 'function' ? await reason.message(undefined) : reason.message
}

function show(options: { reason?: string | null } = {}) {
	editing.value = options.reason !== undefined
	selectedReason.value = CUSTOM_REASON
	message.value = options.reason ?? ''
	modal.value?.show()
}

async function submit() {
	if (messageTooLong.value) return
	const success = await quarantine(props.instance, trimmedMessage.value || null, editing.value)
	if (!success) return
	modal.value?.hide()
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" max-width="600px" width="calc(100vw - 2rem)">
		<template #title>
			<div class="flex min-w-0 items-center gap-3">
				<Avatar
					:src="instance.icon_url"
					:alt="instance.name"
					:tint-by="instance.id"
					size="3rem"
					no-shadow
					pad-transparent-corners
				/>
				<span class="truncate text-2xl font-semibold text-contrast">
					{{ editing ? 'Edit quarantine message' : 'Quarantine instance' }}
				</span>
			</div>
		</template>

		<div class="flex flex-col gap-5">
			<div class="flex flex-col gap-2">
				<span id="quarantine-reason-label" class="font-semibold text-contrast">Reason</span>
				<Combobox
					:model-value="selectedReason"
					:options="reasonOptions"
					aria-labelledby="quarantine-reason-label"
					@update:model-value="(value) => selectReason(value as number)"
				/>
			</div>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					Message to the owner
					<span class="font-medium text-secondary">(optional)</span>
				</span>
				<MarkdownEditor
					v-model="message"
					:max-length="QUARANTINE_REASON_MAX_LENGTH"
					placeholder="Explain why the instance was quarantined"
				/>
				<span class="text-sm text-secondary">
					{{
						trimmedMessage
							? 'The owner will be notified.'
							: "The owner will not be notified as there's no message."
					}}
				</span>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button @click="modal?.hide()">
					<XIcon />
					Cancel
				</Button>
				<Button
					type="colored"
					color="orange"
					:loading="pending"
					:disabled="messageTooLong"
					@click="submit"
				>
					<SaveIcon v-if="editing" />
					<LockIcon v-else />
					{{ editing ? 'Save message' : 'Quarantine' }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
