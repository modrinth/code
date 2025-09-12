<template>
	<NewModal ref="taxFormModal" header="Submitting tax form" :closable="false">
		<div class="max-w-[40rem]">
			<Admonition
				type="info"
				header="Modrinth uses third-party provider Track1099 to securely collect and store your tax forms."
			>
				<a
					href="https://www.track1099.com/info/security"
					class="flex w-fit flex-row gap-1 align-middle text-link"
					target="_blank"
					rel="noopener noreferrer"
				>
					Learn more about their security practices here. <ExternalIcon class="my-auto" />
				</a>
			</Admonition>
			<div class="mt-4 flex flex-col gap-2">
				<label>
					<span class="text-lg font-semibold text-contrast">
						Are you a US citizen?
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<Chips
					v-model="isUSCitizen"
					:items="['yes', 'no']"
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
							Are you a private individual or part of a foreign entity?
							<span class="text-brand-red">*</span>
						</span>
					</label>
					<span class="text-md leading-tight">
						A foreign entity means a business entity organized outside the United States (such as a
						non-US corporation, partnership, or LLC).
					</span>
					<Chips
						v-model="entityType"
						:items="['private-individual', 'foreign-entity']"
						:format-label="
							(item) => (item === 'private-individual' ? 'Private individual' : 'Foreign entity')
						"
						:never-empty="false"
						:capitalize="false"
						class="mt-2"
					/>
				</div>
			</Transition>
			<div class="mt-4 flex w-full flex-row justify-between gap-2">
				<ButtonStyled @click="handleCancel">
					<button><XIcon /> Cancel</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!canContinue || loading" @click="continueForm">
						<template v-if="!loading">Continue <RightArrowIcon /></template>
						<template v-else><SpinnerIcon class="animate-spin" /> Loading…</template>
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { ExternalIcon, RightArrowIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, Chips, injectNotificationManager, NewModal } from '@modrinth/ui'

import { type FormRequestResponse, useAvalara1099 } from '~/composables/avalara1099'

const { addNotification } = injectNotificationManager()

const taxFormModal = ref<InstanceType<typeof NewModal> | null>(null)

async function startTaxForm(e: MouseEvent) {
	taxFormModal.value?.show(e)
}

defineExpose({
	startTaxForm,
})

const auth = await useAuth()

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
		addNotification({
			title: 'Error occurred while continuing tax form',
			text: error instanceof Error ? error.message : String(error),
			type: 'error',
		})
	} finally {
		// Clear manual spinner when control returns to the app
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
