<template>
	<NewModal ref="modal" fade="warning" width="550px">
		<template #title>
			<div class="flex items-center gap-2">
				<span class="text-lg font-extrabold text-contrast">Transfer</span>
				<Avatar :src="organization.icon_url" :alt="organization.name" size="xs" />
				<span class="text-lg font-extrabold text-contrast">{{ organization.name }}</span>
			</div>
		</template>
		<div class="flex flex-col gap-4">
			<Admonition type="warning" header="Selling organizations is against TOS">
				Selling organizations violates Modrinth’s Terms of Use and may lead to moderation actions,
				including organization removal or account suspension.
			</Admonition>
			<div
				class="grid grid-cols-[1fr_auto_1fr] items-center justify-center gap-6 rounded-2xl bg-surface-2 p-4"
			>
				<div class="flex items-center gap-2">
					<Avatar :src="currentOwner.avatar_url" :alt="currentOwner.username" size="xs" circle />
					<div class="flex flex-col items-start justify-start gap-1">
						<span class="font-medium text-contrast">{{ currentOwner.username }}</span>
						<span class="text-sm text-secondary">{{ currentOwner.role }}</span>
					</div>
				</div>
				<RightArrowIcon class="h-6 w-6 text-secondary" />
				<div class="flex items-center gap-2">
					<Avatar :src="transferTo.avatar_url" :alt="transferTo.username" size="xs" circle />
					<div class="flex flex-col items-start justify-start gap-1">
						<span class="font-medium text-contrast">{{ transferTo.username }} </span>
						<span class="text-sm text-secondary">{{ transferTo.role }} </span>
					</div>
				</div>
			</div>
			<ul class="m-0 flex flex-col gap-1 pl-6 text-secondary">
				<li>You will immediately lose owner access to this organization</li>
				<li>The new owner can modify or delete the organization and all its projects</li>
				<li>This action cannot be undone</li>
			</ul>
			<div>
				<p class="m-0 mb-2">
					To confirm this transfer, type
					<span class="font-bold text-contrast">{{ organization.name }}</span> below
				</p>
				<StyledInput
					v-model="confirmationText"
					:placeholder="`Enter ${organization.name}`"
					wrapper-class="w-full"
				/>
			</div>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled>
					<button @click="hide">
						<XIcon />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button :disabled="!isConfirmEnabled" @click="onConfirmClick">
						<TransferIcon />
						Transfer ownership
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { RightArrowIcon, TransferIcon, XIcon } from '@modrinth/assets'
import { Admonition, Avatar, ButtonStyled, NewModal, StyledInput } from '@modrinth/ui'
import { computed, ref } from 'vue'

const props = defineProps<{
	organization: { name: string; icon_url: string | null }
	currentOwner: { avatar_url: string | null; username: string; role: string }
	transferTo: { avatar_url: string | null; username: string; role: string }
	onConfirm: () => void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const confirmationText = ref('')

const isConfirmEnabled = computed(
	() =>
		!!props.organization.name &&
		confirmationText.value.toLowerCase().trim() === props.organization.name.toLowerCase().trim(),
)

function show(e?: MouseEvent) {
	confirmationText.value = ''
	modal.value?.show(e)
}

function hide() {
	modal.value?.hide()
}

function onConfirmClick() {
	hide()
	props.onConfirm()
}

defineExpose({ show, hide })
</script>
