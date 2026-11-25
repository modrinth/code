<template>
	<NewModal ref="modal" fade="warning" width="550px">
		<template #title>
			<div class="flex items-center gap-2">
				<IntlFormatted :message-id="messages.title" :values="{ name: project.name }">
					<template #action="{ children }">
						<span class="text-lg font-extrabold text-contrast"
							><component :is="() => children"
						/></span>
					</template>
					<template #project="{ children }">
						<Avatar
							:src="project.icon_url"
							:raw-src="project.raw_icon_url"
							:alt="project.name"
							size="xs"
							no-shadow
						/>
						<span class="text-lg font-extrabold text-contrast"
							><component :is="() => children"
						/></span>
					</template>
				</IntlFormatted>
			</div>
		</template>
		<div class="flex flex-col gap-4 pb-2">
			<Admonition type="warning" :header="formatMessage(messages.scamWarning)">
				<IntlFormatted :message-id="messages.scamWarningDescription">
					<template #support-link="{ children }">
						<a
							href="https://support.modrinth.com"
							target="_blank"
							rel="noopener noreferrer"
							class="underline"
						>
							<component :is="() => children" />
						</a>
					</template>
				</IntlFormatted>
			</Admonition>
			<div
				class="grid grid-cols-[1fr_auto_1fr] items-center justify-center gap-6 rounded-2xl bg-surface-2 p-4"
			>
				<div class="flex items-center gap-2">
					<Avatar
						:src="currentOwner.avatar_url"
						:alt="currentOwner.username"
						:tint-by="currentOwner.organizationId ?? currentOwner.username"
						size="xs"
						:circle="!currentOwner.organizationId"
					/>
					<div class="flex flex-col items-start justify-start gap-1">
						<span class="font-medium text-contrast">{{ currentOwner.username }}</span>
						<span class="text-sm text-secondary">{{ currentOwner.role }}</span>
					</div>
				</div>
				<RightArrowIcon class="h-6 w-6 text-secondary" />
				<div class="flex items-center gap-2">
					<Avatar
						:src="transferTo.avatar_url"
						:alt="transferTo.username || transferTo.name"
						:tint-by="transferTo.organizationId ?? transferTo.username"
						size="xs"
						:circle="!transferTo.organizationId"
					/>
					<div class="flex flex-col items-start justify-start gap-1">
						<span class="font-medium text-contrast">
							{{ transferTo.username || transferTo.name }}
						</span>
						<span class="text-sm text-secondary">{{
							transferTo.role || formatMessage(messages.member)
						}}</span>
					</div>
				</div>
			</div>
			<ul class="m-0 flex flex-col gap-1 pl-6 text-secondary">
				<li>{{ formatMessage(messages.loseAccess) }}</li>
				<li>{{ formatMessage(messages.newOwnerAccess) }}</li>
				<li>{{ formatMessage(messages.irreversible) }}</li>
			</ul>
			<div>
				<p class="m-0 mb-2">
					<IntlFormatted :message-id="messages.confirmation" :values="{ name: project.name }">
						<template #project="{ children }">
							<span class="font-bold text-contrast"><component :is="() => children" /></span>
						</template>
					</IntlFormatted>
				</p>
				<Input
					v-model="confirmationText"
					:placeholder="formatMessage(messages.confirmationPlaceholder, { name: project.name })"
					wrapper-class="w-full"
				/>
			</div>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button @click="hide">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="red" :disabled="!isConfirmEnabled" @click="onConfirmClick">
					<TransferIcon />
					{{ confirmLabel }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { RightArrowIcon, TransferIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	commonMessages,
	defineMessages,
	Input,
	IntlFormatted,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	title: {
		id: 'project.settings.ownership.confirmation.title',
		defaultMessage: '<action>Transfer</action> <project>{name}</project>',
	},
	scamWarning: {
		id: 'project.settings.ownership.confirmation.scam-warning',
		defaultMessage: 'Beware of scams',
	},
	scamWarningDescription: {
		id: 'project.settings.ownership.confirmation.scam-warning-description',
		defaultMessage:
			'Do not transfer projects to buyers. This is a common scam and against our TOS. If you encounter a buyer, please <support-link>contact support</support-link>.',
	},
	member: {
		id: 'project.settings.ownership.confirmation.member',
		defaultMessage: 'Member',
	},
	loseAccess: {
		id: 'project.settings.ownership.confirmation.lose-access',
		defaultMessage: 'You will immediately lose owner access to this project',
	},
	newOwnerAccess: {
		id: 'project.settings.ownership.confirmation.new-owner-access',
		defaultMessage: 'The new owner can modify or delete the project at any time',
	},
	irreversible: {
		id: 'project.settings.ownership.confirmation.irreversible',
		defaultMessage: 'This action cannot be undone',
	},
	confirmation: {
		id: 'project.settings.ownership.confirmation.confirmation',
		defaultMessage: 'To confirm this transfer, type <project>{name}</project> below',
	},
	confirmationPlaceholder: {
		id: 'project.settings.ownership.confirmation.confirmation-placeholder',
		defaultMessage: 'Enter {name}',
	},
})

const props = defineProps<{
	project: { name: string; icon_url: string | null; raw_icon_url?: string | null }
	currentOwner: {
		avatar_url: string | null
		username: string
		role: string
		organizationId?: string
	}
	transferTo: {
		avatar_url?: string | null
		organizationId?: string
		username?: string
		name?: string
		role?: string
	}
	confirmLabel: string
	onConfirm: () => void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const confirmationText = ref('')

const isConfirmEnabled = computed(
	() =>
		!!props.project.name &&
		confirmationText.value.toLowerCase().trim() === props.project.name.toLowerCase().trim(),
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
