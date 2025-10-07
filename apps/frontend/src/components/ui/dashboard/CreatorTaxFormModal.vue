<template>
	<NewModal
		ref="taxFormModal"
		:header="formatMessage(messages.taxFormHeader)"
		:hide-header="currentStage === 'download-confirmation'"
		:close-on-click-outside="currentStage !== 'download-confirmation'"
		:close-on-esc="currentStage !== 'download-confirmation'"
	>
		<div
			class="w-full"
			:class="[currentStage === 'form-selection' ? 'sm:w-[540px]' : 'sm:w-[400px]']"
		>
			<div v-if="currentStage === 'form-selection'">
				<Admonition type="info" :header="formatMessage(messages.securityHeader)">
					<IntlFormatted :message-id="messages.securityDescription">
						<template #security-link="{ children }">
							<a
								href="https://www.track1099.com/info/security"
								class="flex w-fit flex-row gap-1 align-middle text-link"
								target="_blank"
								rel="noopener noreferrer"
							>
								<component :is="() => normalizeChildren(children)" />
								<ExternalIcon class="my-auto" />
							</a>
						</template>
					</IntlFormatted>
				</Admonition>
				<div class="mt-4 flex flex-col gap-2">
					<label>
						<span class="text-lg font-semibold text-contrast">
							{{ formatMessage(messages.usCitizenQuestion) }}
							<span class="text-brand-red">*</span>
						</span>
					</label>
					<Chips
						v-model="isUSCitizen"
						:items="['yes', 'no']"
						:format-label="
							(item) => (item === 'yes' ? formatMessage(messages.yes) : formatMessage(messages.no))
						"
						:never-empty="false"
						:capitalize="true"
					/>
				</div>

				<Transition
					enter-active-class="transition-all duration-300 ease-in-out"
					enter-from-class="h-0 overflow-hidden opacity-0"
					enter-to-class="h-auto overflow-visible opacity-100"
					leave-active-class="transition-all duration-300 ease-in-out"
					leave-from-class="h-auto overflow-visible opacity-100"
					leave-to-class="h-0 overflow-hidden opacity-0"
				>
					<div v-if="isUSCitizen === 'no'" class="flex flex-col gap-1">
						<label class="mt-4">
							<span class="text-lg font-semibold text-contrast">
								{{ formatMessage(messages.entityQuestion) }}
								<span class="text-brand-red">*</span>
							</span>
						</label>
						<Chips
							v-model="entityType"
							:items="['private-individual', 'foreign-entity']"
							:format-label="
								(item) =>
									item === 'private-individual'
										? formatMessage(messages.privateIndividual)
										: formatMessage(messages.foreignEntity)
							"
							:never-empty="false"
							:capitalize="false"
							class="mt-2"
						/>
						<span class="text-md mt-2 leading-tight">
							{{ formatMessage(messages.entityDescription) }}
						</span>
					</div>
				</Transition>
				<div class="mt-4 flex justify-end gap-3">
					<ButtonStyled @click="handleCancel">
						<button><XIcon /> {{ formatMessage(messages.cancel) }}</button>
					</ButtonStyled>
					<ButtonStyled>
						<button :disabled="!canContinue || loading" @click="continueForm">
							{{ formatMessage(messages.continue) }}
							<RightArrowIcon v-if="!loading" />
							<SpinnerIcon v-else class="animate-spin" />
						</button>
					</ButtonStyled>
				</div>
			</div>

			<div v-else-if="currentStage === 'download-confirmation'" class="flex flex-col gap-6">
				<div class="relative block h-[180px] w-[400px] overflow-hidden rounded-xl rounded-b-none">
					<div
						class="absolute inset-0 rounded-xl rounded-b-none bg-gradient-to-r from-brand-green to-brand-blue"
					></div>
					<div
						class="absolute inset-0 rounded-xl rounded-b-none"
						style="
							background: linear-gradient(
								180deg,
								rgba(39, 41, 46, 0.15) 0%,
								var(--color-raised-bg) 100%
							);
						"
					></div>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						width="220"
						height="132"
						viewBox="0 0 220 132"
						fill="none"
						class="absolute left-[90px] top-[48px] h-[140px] w-[220px]"
					>
						<mask id="path-1-inside-1_687_15946" fill="white">
							<path
								d="M204 0C212.837 0 220 7.16344 220 16V140H0V16C0 7.16344 7.16344 0 16 0H204ZM15 12C13.3431 12 12 13.3431 12 15C12 16.6569 13.3431 18 15 18C16.6569 18 18 16.6569 18 15C18 13.3431 16.6569 12 15 12ZM25 12C23.3431 12 22 13.3431 22 15C22 16.6569 23.3431 18 25 18C26.6569 18 28 16.6569 28 15C28 13.3431 26.6569 12 25 12ZM35 12C33.3431 12 32 13.3431 32 15C32 16.6569 33.3431 18 35 18C36.6569 18 38 16.6569 38 15C38 13.3431 36.6569 12 35 12Z"
							/>
						</mask>
						<path
							d="M204 0C212.837 0 220 7.16344 220 16V140H0V16C0 7.16344 7.16344 0 16 0H204ZM15 12C13.3431 12 12 13.3431 12 15C12 16.6569 13.3431 18 15 18C16.6569 18 18 16.6569 18 15C18 13.3431 16.6569 12 15 12ZM25 12C23.3431 12 22 13.3431 22 15C22 16.6569 23.3431 18 25 18C26.6569 18 28 16.6569 28 15C28 13.3431 26.6569 12 25 12ZM35 12C33.3431 12 32 13.3431 32 15C32 16.6569 33.3431 18 35 18C36.6569 18 38 16.6569 38 15C38 13.3431 36.6569 12 35 12Z"
							fill="white"
							fill-opacity="0.12"
						/>
						<path
							d="M220 16H221H220ZM220 140V141H221V140H220ZM0 140H-1V141H0V140ZM204 0V1C212.284 1 219 7.71573 219 16H220H221C221 6.61116 213.389 -1 204 -1V0ZM220 16H219V140H220H221V16H220ZM220 140V139H0V140V141H220V140ZM0 140H1V16H0H-1V140H0ZM0 16H1C1 7.71573 7.71573 1 16 1V0V-1C6.61116 -1 -1 6.61116 -1 16H0ZM16 0V1H204V0V-1H16V0ZM15 12V11C12.7909 11 11 12.7909 11 15H12H13C13 13.8954 13.8954 13 15 13V12ZM12 15H11C11 17.2091 12.7909 19 15 19V18V17C13.8954 17 13 16.1046 13 15H12ZM15 18V19C17.2091 19 19 17.2091 19 15H18H17C17 16.1046 16.1046 17 15 17V18ZM18 15H19C19 12.7909 17.2091 11 15 11V12V13C16.1046 13 17 13.8954 17 15H18ZM25 12V11C22.7909 11 21 12.7909 21 15H22H23C23 13.8954 23.8954 13 25 13V12ZM22 15H21C21 17.2091 22.7909 19 25 19V18V17C23.8954 17 23 16.1046 23 15H22ZM25 18V19C27.2091 19 29 17.2091 29 15H28H27C27 16.1046 26.1046 17 25 17V18ZM28 15H29C29 12.7909 27.2091 11 25 11V12V13C26.1046 13 27 13.8954 27 15H28ZM35 12V11C32.7909 11 31 12.7909 31 15H32H33C33 13.8954 33.8954 13 35 13V12ZM32 15H31C31 17.2091 32.7909 19 35 19V18V17C33.8954 17 33 16.1046 33 15H32ZM35 18V19C37.2091 19 39 17.2091 39 15H38H37C37 16.1046 36.1046 17 35 17V18ZM38 15H39C39 12.7909 37.2091 11 35 11V12V13C36.1046 13 37 13.8954 37 15H38Z"
							fill="white"
							fill-opacity="0.12"
							mask="url(#path-1-inside-1_687_15946)"
						/>
						<path
							d="M220 16H221H220ZM220 140V141H221V140H220ZM0 140H-1V141H0V140ZM204 0V1C212.284 1 219 7.71573 219 16H220H221C221 6.61116 213.389 -1 204 -1V0ZM220 16H219V140H220H221V16H220ZM220 140V139H0V140V141H220V140ZM0 140H1V16H0H-1V140H0ZM0 16H1C1 7.71573 7.71573 1 16 1V0V-1C6.61116 -1 -1 6.61116 -1 16H0ZM16 0V1H204V0V-1H16V0ZM15 12V11C12.7909 11 11 12.7909 11 15H12H13C13 13.8954 13.8954 13 15 13V12ZM12 15H11C11 17.2091 12.7909 19 15 19V18V17C13.8954 17 13 16.1046 13 15H12ZM15 18V19C17.2091 19 19 17.2091 19 15H18H17C17 16.1046 16.1046 17 15 17V18ZM18 15H19C19 12.7909 17.2091 11 15 11V12V13C16.1046 13 17 13.8954 17 15H18ZM25 12V11C22.7909 11 21 12.7909 21 15H22H23C23 13.8954 23.8954 13 25 13V12ZM22 15H21C21 17.2091 22.7909 19 25 19V18V17C23.8954 17 23 16.1046 23 15H22ZM25 18V19C27.2091 19 29 17.2091 29 15H28H27C27 16.1046 26.1046 17 25 17V18ZM28 15H29C29 12.7909 27.2091 11 25 11V12V13C26.1046 13 27 13.8954 27 15H28ZM35 12V11C32.7909 11 31 12.7909 31 15H32H33C33 13.8954 33.8954 13 35 13V12ZM32 15H31C31 17.2091 32.7909 19 35 19V18V17C33.8954 17 33 16.1046 33 15H32ZM35 18V19C37.2091 19 39 17.2091 39 15H38H37C37 16.1046 36.1046 17 35 17V18ZM38 15H39C39 12.7909 37.2091 11 35 11V12V13C36.1046 13 37 13.8954 37 15H38Z"
							fill="url(#paint0_radial_687_15946)"
							fill-opacity="0.25"
							mask="url(#path-1-inside-1_687_15946)"
						/>
						<path
							d="M110 42C129.882 42 146 58.1177 146 78C146 97.8823 129.882 114 110 114C90.1177 114 74 97.8823 74 78C74 58.1177 90.1177 42 110 42ZM127.828 63.9219C126.266 62.36 123.734 62.3598 122.172 63.9219L104.375 81.7188L97.8281 75.1719C96.266 73.61 93.7339 73.6098 92.1719 75.1719C90.61 76.7339 90.61 79.2661 92.1719 80.8281L101.547 90.2031C103.109 91.7652 105.641 91.765 107.203 90.2031L127.828 69.5781C129.39 68.016 129.39 65.484 127.828 63.9219Z"
							fill="white"
							fill-opacity="0.75"
						/>
						<defs>
							<radialGradient
								id="paint0_radial_687_15946"
								cx="0"
								cy="0"
								r="1"
								gradientUnits="userSpaceOnUse"
								gradientTransform="rotate(34.0355) scale(184.025 170.739)"
							>
								<stop stop-color="white" />
								<stop offset="0.68" stop-color="white" stop-opacity="0" />
							</radialGradient>
						</defs>
					</svg>
				</div>
				<div class="flex flex-col gap-2">
					<span class="text-2xl font-semibold text-contrast">{{
						formatMessage(messages.confirmationTitle)
					}}</span>
					<span>{{
						formatMessage(messages.confirmationSuccess, { formType: determinedFormType })
					}}</span>
					<IntlFormatted :message-id="messages.confirmationSupportText">
						<template #support-link="{ children }">
							<nuxt-link
								to="https://support.modrinth.com"
								class="text-link"
								target="_blank"
								rel="noopener noreferrer"
							>
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
					</IntlFormatted>
				</div>
				<div class="flex w-full flex-row justify-stretch gap-2">
					<ButtonStyled>
						<button class="w-full text-contrast" @click="handleClose">{{ closeButtonText }}</button>
					</ButtonStyled>
					<ButtonStyled color="green">
						<button class="w-full text-contrast" @click="downloadTaxForm">
							<DownloadIcon />{{
								formatMessage(messages.downloadButton, { formType: determinedFormType })
							}}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { DownloadIcon, ExternalIcon, RightArrowIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, Chips, injectNotificationManager, NewModal } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'

import { type FormRequestResponse, useAvalara1099 } from '@/composables/avalara1099'
import { normalizeChildren } from '@/utils/vue-children.ts'

const props = withDefaults(
	defineProps<{
		closeButtonText?: string
		emitSuccessOnClose?: boolean
	}>(),
	{
		closeButtonText: 'Close',
		emitSuccessOnClose: true,
	},
)

const { addNotification } = injectNotificationManager()

const taxFormModal = ref<InstanceType<typeof NewModal> | null>(null)

type ModalStage = 'form-selection' | 'download-confirmation'
const currentStage = ref<ModalStage>('form-selection')

async function startTaxForm(e: MouseEvent) {
	currentStage.value = 'form-selection'
	taxFormModal.value?.show(e)
}

async function showDownloadConfirmation(e: MouseEvent) {
	currentStage.value = 'download-confirmation'
	taxFormModal.value?.show(e)
}

defineExpose({
	startTaxForm,
	showDownloadConfirmation,
})

const auth = await useAuth()
const flags = useFeatureFlags()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	taxFormHeader: {
		id: 'dashboard.creator-tax-form-modal.header',
		defaultMessage: 'Tax form',
	},
	securityHeader: {
		id: 'dashboard.creator-tax-form-modal.security.header',
		defaultMessage: 'Security practices',
	},
	securityDescription: {
		id: 'dashboard.creator-tax-form-modal.security.description',
		defaultMessage:
			'Modrinth uses third-party provider Track1099 to securely collect and store your tax forms. <security-link>Learn more here.</security-link>',
	},
	usCitizenQuestion: {
		id: 'dashboard.creator-tax-form-modal.us-citizen.question',
		defaultMessage: 'Are you a US citizen?',
	},
	yes: { id: 'common.yes', defaultMessage: 'Yes' },
	no: { id: 'common.no', defaultMessage: 'No' },
	entityQuestion: {
		id: 'dashboard.creator-tax-form-modal.entity.question',
		defaultMessage: 'Are you a private individual or part of a foreign entity?',
	},
	entityDescription: {
		id: 'dashboard.creator-tax-form-modal.entity.description',
		defaultMessage:
			'A foreign entity means a business entity organized outside the United States (such as a non-US corporation, partnership, or LLC).',
	},
	privateIndividual: {
		id: 'dashboard.creator-tax-form-modal.entity.private-individual',
		defaultMessage: 'Private individual',
	},
	foreignEntity: {
		id: 'dashboard.creator-tax-form-modal.entity.foreign-entity',
		defaultMessage: 'Foreign entity',
	},
	cancel: { id: 'action.cancel', defaultMessage: 'Cancel' },
	continue: { id: 'action.continue', defaultMessage: 'Continue' },
	confirmationTitle: {
		id: 'dashboard.creator-tax-form-modal.confirmation.title',
		defaultMessage: "You're all set! 🎉",
	},
	confirmationSuccess: {
		id: 'dashboard.creator-tax-form-modal.confirmation.success',
		defaultMessage: 'Your {formType} tax form has been submitted successfully!',
	},
	confirmationSupportText: {
		id: 'dashboard.creator-tax-form-modal.confirmation.support-text',
		defaultMessage:
			'You can freely withdraw now. If you have questions or need to update your details <support-link>contact support</support-link>.',
	},
	downloadButton: {
		id: 'dashboard.creator-tax-form-modal.confirmation.download-button',
		defaultMessage: 'Download {formType}',
	},
})

const isUSCitizen = ref<'yes' | 'no' | null>(null)
const entityType = ref<'private-individual' | 'foreign-entity' | null>(null)

function hideModal() {
	manualLoading.value = false
	taxFormModal.value?.hide()
}

function handleCancel() {
	emit('cancelled')
	hideModal()
	setTimeout(() => {
		currentStage.value = 'form-selection'
	}, 300)
}

function handleClose() {
	if (currentStage.value === 'download-confirmation' && props.emitSuccessOnClose) {
		emit('success')
	}
	hideModal()
	setTimeout(() => {
		currentStage.value = 'form-selection'
	}, 300)
}

const determinedFormType = computed(() => {
	if (isUSCitizen.value === 'yes') {
		return 'W-9'
	} else if (isUSCitizen.value === 'no' && entityType.value === 'private-individual') {
		return 'W-8BEN'
	} else if (isUSCitizen.value === 'no' && entityType.value === 'foreign-entity') {
		return 'W-8BEN-E'
	}
	return null
})

const canContinue = computed(() => {
	if (isUSCitizen.value === 'yes') {
		return true
	} else if (isUSCitizen.value === 'no' && entityType.value) {
		return true
	}
	return false
})

const emit = defineEmits<{
	(event: 'success' | 'cancelled'): void
}>()

const avalaraState = ref<ReturnType<typeof useAvalara1099> | null>(null)
const formResponse = ref<any>(null)
const manualLoading = ref(false)
const loading = computed(
	() =>
		manualLoading.value ||
		(avalaraState.value ? ((avalaraState.value as any).loading?.value ?? false) : false),
)

async function continueForm() {
	if (!import.meta.client) return
	if (!determinedFormType.value) return

	manualLoading.value = true

	// Skip Avalara if testTaxForm flag is enabled
	if (flags.value.testTaxForm) {
		currentStage.value = 'download-confirmation'
		manualLoading.value = false
		return
	}

	const response = (await useBaseFetch('payout/compliance', {
		apiVersion: 3,
		method: 'POST',
		body: {
			form_type: determinedFormType.value,
		},
	})) as FormRequestResponse

	if (!avalaraState.value) {
		avalaraState.value = useAvalara1099(response, {
			prefill: {
				email: (auth.value.user as any)?.email ?? '',
				account_number: (auth.value.user as any)?.id ?? '',
				reference_number: (auth.value.user as any)?.id ?? '',
			},
		})
	}

	try {
		if (avalaraState.value) {
			const response = await avalaraState.value.start()
			formResponse.value = response
			if (avalaraState.value.status === 'signed') {
				currentStage.value = 'download-confirmation'
				manualLoading.value = false
				return
			}

			addNotification({
				title: 'Tax form incomplete',
				text: 'You have not completed the tax form. Please try again.',
				type: 'warning',
			})
		}
	} catch (error) {
		console.error('Error occurred while continuing tax form:', error)
		handleCancel()
	} finally {
		manualLoading.value = false
	}
}

function downloadTaxForm() {
	if (!formResponse.value) return

	const signedPdfUrl = formResponse.value.links?.signed_pdf
	if (signedPdfUrl) {
		window.open(signedPdfUrl, '_blank')
	}
}

watch(isUSCitizen, (newValue) => {
	if (newValue === 'yes') {
		entityType.value = null
	}
})
</script>

<style>
dialog[open]:has(> iframe[src*='form_embed']) {
	width: min(960px, calc(100vw - 2rem)) !important;
	max-width: 100% !important;
	height: min(95vh, max(640px, 75vh)) !important;
	background: var(--color-raised-bg) !important;
	border: 1px solid var(--color-button-border) !important;
	border-radius: var(--radius-lg) !important;
	box-shadow: var(--shadow-floating) !important;
	padding: 0 !important;
}

dialog[open] > iframe[src*='form_embed'] {
	position: absolute !important;
	inset: 0 !important;
	width: 100% !important;
	height: 100% !important;
	display: block !important;
	border: none !important;
	border-radius: var(--radius-lg) !important;
}

@media (max-width: 640px) {
	dialog[open]:has(> iframe[src*='form_embed']) {
		width: calc(100vw - 1rem) !important;
		height: 95vh !important;
		border-radius: var(--radius-md) !important;
	}

	dialog[open] > iframe[src*='form_embed'] {
		border-radius: var(--radius-md) !important;
	}
}
</style>
