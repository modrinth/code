<template>
	<div
		class="shadow-card mx-auto flex w-full max-w-[28rem] flex-col gap-6 rounded-2xl border border-button-bg bg-surface-3 p-6"
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
			<DatePicker
				id="create-account-dob"
				v-model="dateOfBirthModel"
				wrapper-class="w-full"
				min-date="1900-01-01"
				:max-date="maxInputDate"
				preserve-day
				:placeholder="formatMessage(messages.dateOfBirthPlaceholder)"
			/>
			<div>
				{{ formatMessage(messages.over13HelperText) }}
			</div>
			<Admonition :type="'info'">
				<template #header>
					<div class="-mb-3 -mt-1 flex flex-col gap-0 text-sm font-normal leading-normal">
						<div>
							{{ formatMessage(messages.infoPanelText) }}
						</div>
						<a
							class="w-fit text-link underline"
							:href="SOURCE_CODE_URL"
							target="_blank"
							rel="noopener noreferrer"
						>
							{{ formatMessage(messages.relevantSourceCodeText) }}
						</a>
					</div>
				</template>
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

		<section v-if="globals?.captcha_enabled" class="flex flex-col gap-2.5">
			<label class="text-md font-semibold text-contrast">{{
				formatMessage(messages.securityCheckLabel)
			}}</label>
			<HCaptcha v-if="globals?.captcha_enabled" :ref="onSetCaptchaRef" v-model="tokenModel" />
		</section>

		<div
			class="flex gap-2.5 rounded-2xl border border-solid border-surface-5 bg-surface-3 transition-all hover:brightness-110"
		>
			<Checkbox
				v-model="subscribeModel"
				class="p-3 text-left leading-snug text-primary transition-all"
				:label="formatMessage(messages.subscribeLabel)"
				:description="formatMessage(messages.subscribeLabel)"
			/>
		</div>

		<ButtonStyled color="brand">
			<button
				class="!w-full font-bold"
				:disabled="globals?.captcha_enabled ? !tokenModel : false"
				@click="onCompleteSignUpClick"
			>
				{{ formatMessage(messages.completeSignUpButton) }}
				<RightArrowIcon />
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { RightArrowIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	Checkbox,
	DatePicker,
	defineMessages,
	injectNotificationManager,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import HCaptcha from '@/components/ui/auth/HCaptcha.vue'

interface AuthGlobals {
	captcha_enabled?: boolean
	[key: string]: unknown
}

interface Props {
	globals?: AuthGlobals | null
	requiresDob?: boolean
	onCompleteSignUp?: () => void
	onSetCaptchaRef?: ((captchaRef: unknown) => void) | undefined
}

const {
	globals = null,
	requiresDob = true,
	onCompleteSignUp = () => {},
	onSetCaptchaRef = undefined,
} = defineProps<Props>()

const SOURCE_CODE_URL =
	'https://github.com/modrinth/code/blob/main/apps/frontend/src/components/ui/auth/CreateAccount.vue'

const dateOfBirthModel = defineModel<string | null>('dateOfBirth', { default: '' })
const usernameModel = defineModel<string>('username', { default: '' })
const tokenModel = defineModel<string>('token', { default: '' })
const subscribeModel = defineModel<boolean>('subscribe', { default: false })

const maxInputDate = computed(() => `${new Date().getFullYear()}-12-31`)

const maxBirthDate = computed(() => {
	const date = new Date()
	date.setFullYear(date.getFullYear() - 13)
	return date.toISOString().slice(0, 10)
})

const getBirthYear = (dateOfBirth: string | null): number | null => {
	if (!dateOfBirth) {
		return null
	}

	const [yearPart = ''] = dateOfBirth.split('-')
	const year = Number(yearPart)
	return Number.isInteger(year) ? year : null
}

const isDateOfBirthMissing = computed(() => requiresDob && !dateOfBirthModel.value)

const isDateOfBirthYearZero = computed(() => {
	if (!requiresDob || !dateOfBirthModel.value) {
		return false
	}

	return getBirthYear(dateOfBirthModel.value) === 0
})

const isUnder13 = computed(
	() => requiresDob && !!dateOfBirthModel.value && dateOfBirthModel.value > maxBirthDate.value,
)

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

function onCompleteSignUpClick() {
	if (isDateOfBirthMissing.value) {
		addNotification({
			title: formatMessage(messages.dateOfBirthRequiredTitle),
			text: formatMessage(messages.dateOfBirthRequiredText),
			type: 'warning',
		})
		return
	}

	if (isDateOfBirthYearZero.value) {
		addNotification({
			title: formatMessage(messages.dateOfBirthInvalidTitle),
			text: formatMessage(messages.dateOfBirthInvalidText),
			type: 'error',
		})
		return
	}

	if (isUnder13.value) {
		addNotification({
			title: formatMessage(messages.ageRequirementWarningTitle),
			text: formatMessage(messages.under13HelperText),
			type: 'error',
		})
		return
	}

	onCompleteSignUp()
}

const messages = defineMessages({
	title: {
		id: 'auth.create-account.title',
		defaultMessage: 'Create an Account',
	},
	dateOfBirthLabel: {
		id: 'auth.create-account.date-of-birth.label',
		defaultMessage: 'Date of birth',
	},
	dateOfBirthPlaceholder: {
		id: 'auth.create-account.date-of-birth.placeholder',
		defaultMessage: 'Select your date of birth',
	},
	dateOfBirthRequiredTitle: {
		id: 'auth.create-account.date-of-birth.required.title',
		defaultMessage: 'Date of birth required',
	},
	dateOfBirthRequiredText: {
		id: 'auth.create-account.date-of-birth.required.text',
		defaultMessage: 'Please enter your date of birth before continuing.',
	},
	dateOfBirthInvalidTitle: {
		id: 'auth.create-account.date-of-birth.invalid.title',
		defaultMessage: 'Invalid date of birth',
	},
	dateOfBirthInvalidText: {
		id: 'auth.create-account.date-of-birth.invalid.text',
		defaultMessage: 'Please enter a valid date of birth. Year cannot be 0000.',
	},
	over13HelperText: {
		id: 'auth.create-account.date-of-birth.over13-helper',
		defaultMessage: 'You must be over 13 years old to use Modrinth.',
	},
	under13HelperText: {
		id: 'auth.create-account.date-of-birth.under13-helper',
		defaultMessage: 'You cannot create an account at Modrinth unless you are over 13 years old.',
	},
	ageRequirementWarningTitle: {
		id: 'auth.create-account.age-requirement.warning-title',
		defaultMessage: 'Age requirement',
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
