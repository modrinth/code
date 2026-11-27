<script setup lang="ts">
import { ShieldOffIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	Checkbox,
	commonMessages,
	defineMessages,
	injectPageContext,
	IntlFormatted,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { SANDBOX_HELP_URL, SUPPORT_URL } from './links'

const emit = defineEmits<{ confirm: [] }>()

const { formatMessage } = useVIntl()
const { openExternalUrl } = injectPageContext()

const messages = defineMessages({
	header: {
		id: 'app.settings.sandbox.disable.header',
		defaultMessage: 'Disable the sandbox?',
	},
	description: {
		id: 'app.settings.sandbox.disable.description',
		defaultMessage:
			'If someone online or a mod asks you to disable the sandbox, they may be trying to gain access to your personal data. Before disabling it, confirm that you understand the risks below.',
	},
	risksTitle: {
		id: 'app.settings.sandbox.disable.risks-title',
		defaultMessage: 'A malicious mod could:',
	},
	filesTitle: {
		id: 'app.settings.sandbox.disable.risk.files.title',
		defaultMessage: 'Read, change, or delete your files',
	},
	filesDescription: {
		id: 'app.settings.sandbox.disable.risk.files.description',
		defaultMessage: 'Your documents, photos, and other files could be stolen or erased.',
	},
	passwordsTitle: {
		id: 'app.settings.sandbox.disable.risk.passwords.title',
		defaultMessage: 'Steal your passwords and logins',
	},
	passwordsDescription: {
		id: 'app.settings.sandbox.disable.risk.passwords.description',
		defaultMessage: 'Your Discord, browser passwords, and banking credentials could be stolen.',
	},
	accountTitle: {
		id: 'app.settings.sandbox.disable.risk.account.title',
		defaultMessage: 'Take over your Minecraft account',
	},
	accountDescription: {
		id: 'app.settings.sandbox.disable.risk.account.description',
		defaultMessage: 'Someone could gain access to your account, impersonate you, or lock you out.',
	},
	help: {
		id: 'app.settings.sandbox.disable.help',
		defaultMessage:
			'If you have any questions, <support>contact support</support> or read our <article>help article</article> before continuing.',
	},
	confirm: {
		id: 'app.settings.sandbox.disable.confirm',
		defaultMessage: 'Disable sandbox',
	},
})

const risks = [
	{ title: messages.filesTitle, description: messages.filesDescription },
	{ title: messages.passwordsTitle, description: messages.passwordsDescription },
	{ title: messages.accountTitle, description: messages.accountDescription },
]

const modal = ref<InstanceType<typeof NewModal>>()
const acknowledged = ref<boolean[]>(risks.map(() => false))
const allAcknowledged = computed(() => acknowledged.value.every(Boolean))

function show(event?: MouseEvent) {
	acknowledged.value = risks.map(() => false)
	modal.value?.show(event)
}

function confirm() {
	if (!allAcknowledged.value) return
	modal.value?.hide()
	emit('confirm')
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="danger" max-width="544px">
		<div class="flex flex-col gap-6">
			<p class="m-0 leading-6">{{ formatMessage(messages.description) }}</p>
			<div class="flex flex-col gap-2.5">
				<span class="font-semibold leading-6 text-contrast">
					{{ formatMessage(messages.risksTitle) }}
				</span>
				<Checkbox
					v-for="(risk, index) in risks"
					:key="index"
					v-model="acknowledged[index]"
					:description="formatMessage(risk.title)"
					color="white"
					class="w-full !items-start rounded-[18px] border !border-solid border-surface-5 !bg-surface-4 !p-3"
				>
					<span class="flex min-w-0 flex-1 flex-col gap-1">
						<span class="font-semibold leading-6 text-contrast">
							{{ formatMessage(risk.title) }}
						</span>
						<span class="text-sm leading-5 text-primary">
							{{ formatMessage(risk.description) }}
						</span>
					</span>
				</Checkbox>
			</div>
			<p class="m-0 leading-6">
				<IntlFormatted :message-id="messages.help">
					<template #support="{ children }">
						<button
							type="button"
							class="inline cursor-pointer border-0 bg-transparent p-0 text-[length:inherit] font-medium leading-[inherit] text-blue hover:underline"
							@click="openExternalUrl(SUPPORT_URL)"
						>
							<component :is="() => children" />
						</button>
					</template>
					<template #article="{ children }">
						<button
							type="button"
							class="inline cursor-pointer border-0 bg-transparent p-0 text-[length:inherit] font-medium leading-[inherit] text-blue hover:underline"
							@click="openExternalUrl(SANDBOX_HELP_URL)"
						>
							<component :is="() => children" />
						</button>
					</template>
				</IntlFormatted>
			</p>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="red" :disabled="!allAcknowledged" @click="confirm">
					<ShieldOffIcon aria-hidden="true" />
					{{ formatMessage(messages.confirm) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
