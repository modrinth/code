<template>
	<NewModal ref="modal" header="Install from invite link" max-width="32rem">
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">Invite link</span>
			<StyledInput
				v-model="inviteLink"
				placeholder="https://modrinth.com/share/..."
				:disabled="processing"
				@keydown.enter="process"
			/>
			<span v-if="inviteLink && !inviteId" class="text-sm text-red">
				Enter a valid Modrinth shared-instance invite link.
			</span>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled>
					<button :disabled="processing" @click="modal?.hide()"><XIcon /> Cancel</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!inviteId || processing" @click="process">
						<LinkIcon />
						Process
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { LinkIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, NewModal, StyledInput } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { parseSharedInstanceInviteLink } from './shared-instance-invite-parser'

const props = defineProps<{ processInvite: (inviteId: string) => Promise<void> }>()
const modal = ref<InstanceType<typeof NewModal>>()
const inviteLink = ref('')
const processing = ref(false)
const inviteId = computed(() => parseSharedInstanceInviteLink(inviteLink.value))

function show(event?: MouseEvent) {
	inviteLink.value = ''
	modal.value?.show(event)
}

async function process() {
	if (!inviteId.value) return

	processing.value = true
	modal.value?.hide()
	try {
		await props.processInvite(inviteId.value)
	} finally {
		processing.value = false
	}
}

defineExpose({ show })
</script>
