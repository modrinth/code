<template>
	<div
		class="shadow-card mx-auto flex w-full max-w-[30rem] flex-col gap-6 rounded-2xl border border-button-bg bg-surface-3 p-6"
	>
		<h1
			class="mx-auto my-0 flex w-full justify-center text-center text-2xl font-semibold text-contrast"
		>
			{{ formatMessage(messages.title) }}
		</h1>

		<section v-if="requiresDob" class="flex flex-col gap-2.5">
			<label class="text-md font-semibold text-contrast" for="create-account-dob">
				{{ formatMessage(messages.dateOfBirthLabel) }}
			</label>
			<input
				id="create-account-dob"
				v-model="dateOfBirthModel"
				class="scheme-dark w-full border-0 bg-surface-4 text-lg text-contrast outline-none [color-scheme:dark]"
				type="date"
				:max="maxBirthDate"
			/>
			<div>You must be over 13 years old to use Modrinth.</div>
			<Admonition type="info">
				<div class="flex flex-col gap-1.5 leading-normal">
					<div>
						{{ formatMessage(messages.infoPanelText) }}
					</div>
					<a
						class="w-fit text-link"
						:href="sourceCodeUrl"
						target="_blank"
						rel="noopener noreferrer"
					>
						{{ formatMessage(messages.relevantSourceCodeText) }}
					</a>
				</div>
			</Admonition>
		</section>

		<section class="flex flex-col gap-2.5">
			<label class="text-md font-semibold text-contrast" for="create-account-username">
				{{ formatMessage(messages.usernameOptionalLabel) }}
				<span class="font-normal text-primary">(optional)</span>
			</label>
			<StyledInput
				id="create-account-username"
				v-model="usernameModel"
				type="text"
				:placeholder="formatMessage(messages.usernamePlaceholder)"
				wrapper-class="w-full"
			/>
		</section>

		<section class="flex flex-col gap-2.5" v-if="globals?.captcha_enabled">
			<label class="text-md font-semibold text-contrast">{{
				formatMessage(messages.securityCheckLabel)
			}}</label>
			<HCaptcha v-if="globals?.captcha_enabled" :ref="onSetCaptchaRef" v-model="tokenModel" />
		</section>

		<div class="flex gap-2.5 rounded-2xl border border-solid border-surface-5 p-3">
			<Checkbox
				v-model="subscribeModel"
				class="text-left leading-snug text-primary transition-all hover:brightness-100"
				:label="formatMessage(messages.subscribeLabel)"
				:description="formatMessage(messages.subscribeLabel)"
			/>
		</div>

		<ButtonStyled color="brand">
			<button
				class="!w-full font-bold"
				:disabled="globals?.captcha_enabled ? !tokenModel : false"
				@click="onCompleteSignUp()"
			>
				{{ formatMessage(messages.completeSignUpButton) }}
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup>
import {
	Admonition,
	ButtonStyled,
	Checkbox,
	defineMessages,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import HCaptcha from '@/components/ui/auth/HCaptcha.vue'

const props = defineProps({
	dateOfBirth: {
		type: String,
		default: '',
	},
	username: {
		type: String,
		default: '',
	},
	token: {
		type: String,
		default: '',
	},
	subscribe: {
		type: Boolean,
		default: false,
	},
	globals: {
		type: Object,
		default: null,
	},
	requiresDob: {
		type: Boolean,
		default: true,
	},
	sourceCodeUrl: {
		type: String,
		default:
			'https://github.com/modrinth/labrinth/blob/main/apps/labrinth/src/routes/internal/flows.rs',
	},
	onCompleteSignUp: {
		type: Function,
		default: () => {},
	},
	onSetCaptchaRef: {
		type: Function,
		default: undefined,
	},
})

const emit = defineEmits([
	'update:dateOfBirth',
	'update:username',
	'update:token',
	'update:subscribe',
])

const dateOfBirthModel = computed({
	get: () => props.dateOfBirth,
	set: (value) => emit('update:dateOfBirth', value),
})

const usernameModel = computed({
	get: () => props.username,
	set: (value) => emit('update:username', value),
})

const tokenModel = computed({
	get: () => props.token,
	set: (value) => emit('update:token', value),
})

const subscribeModel = computed({
	get: () => props.subscribe,
	set: (value) => emit('update:subscribe', value),
})

const maxBirthDate = computed(() => {
	const date = new Date()
	date.setFullYear(date.getFullYear() - 13)
	return date.toISOString().slice(0, 10)
})

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'auth.create-account.title',
		defaultMessage: 'Create an Account',
	},
	dateOfBirthLabel: {
		id: 'auth.create-account.date-of-birth.label',
		defaultMessage: 'Date of birth',
	},
	over13HelperText: {
		id: 'auth.create-account.date-of-birth.over13-helper',
		defaultMessage: 'You must be over 13 years old to use Modrinth.',
	},
	infoPanelText: {
		id: 'auth.create-account.info-panel.text',
		defaultMessage:
			'We do not store your date of birth, it is only used to confirm your age at sign up.',
	},
	relevantSourceCodeText: {
		id: 'auth.create-account.info-panel.source-code-link',
		defaultMessage: 'Relevant source code',
	},
	usernameOptionalLabel: {
		id: 'auth.create-account.username.optional-label',
		defaultMessage: 'Username',
	},
	usernamePlaceholder: {
		id: 'auth.create-account.username.placeholder',
		defaultMessage: 'Enter username',
	},
	securityCheckLabel: {
		id: 'auth.create-account.security-check.label',
		defaultMessage: 'Security check',
	},
	subscribeLabel: {
		id: 'auth.create-account.subscribe.label',
		defaultMessage: 'Keep me updated on the cool things Modrinth is working on via email',
	},
	completeSignUpButton: {
		id: 'auth.create-account.complete-sign-up',
		defaultMessage: 'Complete sign up',
	},
})
</script>
