<template>
	<NewModal ref="taxFormModal" :header="formatMessage(messages.taxFormHeader)">
		<div class="w-full sm:w-[540px]">
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
						<RightArrowIcon v-if="!loading" /> <SpinnerIcon v-else class="animate-spin" />
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { ExternalIcon, RightArrowIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, Chips, injectNotificationManager, NewModal } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'

import { type FormRequestResponse, useAvalara1099 } from '@/composables/avalara1099'
import { normalizeChildren } from '@/utils/vue-children.ts'

const { addNotification } = injectNotificationManager()

const taxFormModal = ref<InstanceType<typeof NewModal> | null>(null)

async function startTaxForm(e: MouseEvent) {
	taxFormModal.value?.show(e)
}

defineExpose({
	startTaxForm,
})

const auth = await useAuth()

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
			await avalaraState.value.start()
			if (avalaraState.value.status === 'signed') {
				addNotification({
					title: 'Tax form submitted',
					text: 'You can now withdraw your full balance.',
					type: 'success',
				})
				emit('success')
				hideModal()
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
	} finally {
		manualLoading.value = false
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
