<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="warning" max-width="560px">
		<div class="flex flex-col gap-6">
			<Admonition type="warning" :header="warningHeader">
				<template #icon="{ iconClass }">
					<CircleAlertIcon :class="iconClass" />
				</template>
				<IntlFormatted :message-id="mode === 'owner' ? messages.ownerBody : messages.userBody">
					<template #username>
						<span class="font-semibold text-contrast">{{ username }}</span>
					</template>
				</IntlFormatted>
			</Admonition>

			<p class="m-0 text-secondary">
				{{ formatMessage(messages.continueDescription) }}
			</p>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button @click="goBack">
						{{ formatMessage(messages.goBackButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button @click="continueAnyway">
						{{ formatMessage(messages.continueButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CircleAlertIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	defineMessages,
	injectAuth,
	IntlFormatted,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

type SharedInstanceWrongAccountMode = 'owner' | 'user'

const emit = defineEmits<{
	(e: 'continue'): void
}>()

const { formatMessage } = useVIntl()
const auth = injectAuth()
const router = useRouter()
const modal = ref<InstanceType<typeof NewModal>>()
const mode = ref<SharedInstanceWrongAccountMode>('user')
const username = ref('')
const warningHeader = computed(() =>
	formatMessage(
		!auth.session_token.value ? messages.signedOutWarningHeader : messages.warningHeader,
	),
)

const messages = defineMessages({
	header: {
		id: 'app.instance.shared-instance-wrong-account.header',
		defaultMessage: 'Shared instance warning',
	},
	warningHeader: {
		id: 'app.instance.shared-instance-wrong-account.warning-header',
		defaultMessage: 'You are using the wrong Modrinth account',
	},
	signedOutWarningHeader: {
		id: 'app.instance.shared-instance-wrong-account.signed-out-warning-header',
		defaultMessage: 'Sign in to the correct Modrinth account first',
	},
	userBody: {
		id: 'app.instance.shared-instance-wrong-account.user-body',
		defaultMessage:
			'You need to sign in as <username></username> to receive updates for this shared instance.',
	},
	ownerBody: {
		id: 'app.instance.shared-instance-wrong-account.owner-body',
		defaultMessage:
			'You need to sign in as <username></username> to manage sharing and publish updates for this shared instance.',
	},
	continueDescription: {
		id: 'app.instance.shared-instance-wrong-account.continue-description',
		defaultMessage:
			'Continuing will disable shared instances functionality until you use the correct Modrinth account.',
	},
	goBackButton: {
		id: 'app.instance.shared-instance-wrong-account.go-back-button',
		defaultMessage: 'Go back',
	},
	continueButton: {
		id: 'app.instance.shared-instance-wrong-account.continue-button',
		defaultMessage: 'Continue anyway',
	},
})

function show(nextMode: SharedInstanceWrongAccountMode, nextUsername: string, event?: MouseEvent) {
	mode.value = nextMode
	username.value = nextUsername
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

function continueAnyway() {
	hide()
	emit('continue')
}

async function goBack() {
	hide()
	if (!auth.session_token.value) {
		await router.push({ path: '/' })
		return
	}

	router.back()
}

defineExpose({
	show,
	hide,
})
</script>
