<template>
	<div class="create-account-card">
		<h1 class="create-account-title">{{ formatMessage(messages.title) }}</h1>

		<section class="create-account-section">
			<label class="create-account-label" for="create-account-dob">
				{{ formatMessage(messages.dateOfBirthLabel) }}
			</label>
			<div class="date-input-wrap">
				<input
					id="create-account-dob"
					v-model="dateOfBirthModel"
					class="date-input"
					type="date"
					:max="maxBirthDate"
				/>
				<CalendarIcon class="date-input-icon" />
			</div>
			<p class="helper-text">{{ formatMessage(messages.over13HelperText) }}</p>
		</section>

		<section class="info-panel">
			<div class="info-panel-icon">
				<InfoIcon />
			</div>
			<div class="info-panel-content">
				<p>{{ formatMessage(messages.infoPanelText) }}</p>
				<a class="text-link" :href="sourceCodeUrl" target="_blank" rel="noopener noreferrer">
					{{ formatMessage(messages.relevantSourceCodeText) }}
				</a>
			</div>
		</section>

		<section class="create-account-section">
			<label class="create-account-label" for="create-account-username">
				{{ formatMessage(messages.usernameOptionalLabel) }}
			</label>
			<StyledInput
				id="create-account-username"
				v-model="usernameModel"
				type="text"
				:placeholder="formatMessage(messages.usernamePlaceholder)"
				wrapper-class="w-full"
			/>
		</section>

		<section class="create-account-section">
			<label class="create-account-label">{{ formatMessage(messages.securityCheckLabel) }}</label>
			<div class="captcha-wrap">
				<HCaptcha v-if="globals?.captcha_enabled" :ref="onSetCaptchaRef" v-model="tokenModel" />
			</div>
		</section>

		<Checkbox
			v-model="subscribeModel"
			class="subscribe-checkbox"
			:label="formatMessage(messages.subscribeLabel)"
			:description="formatMessage(messages.subscribeLabel)"
		/>

		<ButtonStyled color="brand">
			<button
				class="!w-full complete-sign-up-btn"
				:disabled="globals?.captcha_enabled ? !tokenModel : false"
				@click="onCompleteSignUp()"
			>
				{{ formatMessage(messages.completeSignUpButton) }}
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup>
import { CalendarIcon, InfoIcon } from '@modrinth/assets'
import { ButtonStyled, Checkbox, defineMessages, StyledInput, useVIntl } from '@modrinth/ui'
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
	sourceCodeUrl: {
		type: String,
		default: 'https://github.com/modrinth/labrinth/blob/main/apps/labrinth/src/routes/internal/flows.rs',
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
		defaultMessage: 'We do not store your date of birth, it is only used to confirm your age at sign up.',
	},
	relevantSourceCodeText: {
		id: 'auth.create-account.info-panel.source-code-link',
		defaultMessage: 'Relevant source code',
	},
	usernameOptionalLabel: {
		id: 'auth.create-account.username.optional-label',
		defaultMessage: 'Username (Optional)',
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

<style scoped lang="scss">
.create-account-card {
	background: var(--color-raised-bg);
	border: 1px solid var(--color-button-bg);
	border-radius: var(--size-rounded-xl);
	box-shadow: var(--shadow-card);
	display: flex;
	flex-direction: column;
	gap: var(--gap-md);
	margin-inline: auto;
	max-width: 30rem;
	padding: var(--gap-xl);
}

.create-account-title {
	font-size: var(--text-4xl);
	font-weight: var(--weight-bold);
	line-height: 1.2;
	margin: 0;
	text-align: center;
}

.create-account-section {
	display: flex;
	flex-direction: column;
	gap: var(--gap-sm);
}

.create-account-label {
	color: var(--color-contrast);
	font-size: var(--text-xl);
	font-weight: var(--weight-bold);
}

.date-input-wrap {
	align-items: center;
	background: var(--color-button-bg);
	border-radius: var(--size-rounded-lg);
	display: flex;
	gap: var(--gap-sm);
	padding: 0.875rem 1rem;
}

.date-input {
	background: transparent;
	border: 0;
	color: var(--color-contrast);
	font-size: var(--text-lg);
	outline: none;
	width: 100%;
}

.date-input-icon {
	color: var(--color-secondary);
	flex-shrink: 0;
	height: 1.2rem;
	width: 1.2rem;
}

.helper-text {
	color: var(--color-secondary);
	font-size: var(--text-lg);
	margin: 0;
}

.info-panel {
	background: color-mix(in oklab, var(--color-brand) 20%, var(--color-bg));
	border: 1px solid color-mix(in oklab, var(--color-brand) 80%, transparent);
	border-radius: var(--size-rounded-xl);
	display: flex;
	gap: var(--gap-sm);
	padding: var(--gap-md);
}

.info-panel-icon {
	color: var(--color-brand);
	display: flex;
	flex-shrink: 0;
}

.info-panel-content {
	display: flex;
	flex-direction: column;
	gap: var(--gap-xs);

	p {
		color: var(--color-contrast);
		font-size: var(--text-lg);
		margin: 0;
	}
}

.captcha-wrap {
	background: var(--color-button-bg);
	border-radius: var(--size-rounded-lg);
	min-height: 4.25rem;
	padding: var(--gap-md);
}

.subscribe-checkbox {
	border: 1px solid var(--color-button-bg);
	border-radius: var(--size-rounded-xl);
	padding: 0.75rem 1rem;
}

.complete-sign-up-btn {
	font-weight: var(--weight-bold);
}
</style>
