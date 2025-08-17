<template>
	<NewModal ref="taxFormModal" header="Submit Tax Form" :closable="false">
		<div
			v-if="!shrinkModalBehindAvalara"
			class="card flex flex-col rounded-2xl border-[1px] border-solid !bg-button-bg p-2 text-contrast"
		>
			<span>
				Modrinth has a legal obligation to share these tax forms with the United States Internal
				Revenue Service (IRS) after you meet the annual withdrawal threshold of $600 USD.
			</span>
		</div>
		<div class="mt-2 flex flex-col gap-4" v-if="!shrinkModalBehindAvalara">
			<div class="flex flex-col gap-2">
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
				<div v-if="isUSCitizen === 'no'" class="flex flex-col gap-2">
					<label>
						<span class="text-lg font-semibold text-contrast">
							Are you a private individual or part of a foreign entity?
							<span class="text-brand-red">*</span>
						</span>
					</label>
					<span class="text-md -mt-2"
						>A foreign entity means a business entity organized outside the United States (such as a
						non-US corporation, partnership, or LLC).</span
					>
					<Chips
						v-model="entityType"
						:items="['private-individual', 'foreign-entity']"
						:format-label="
							(item) => (item === 'private-individual' ? 'Private Individual' : 'Foreign Entity')
						"
						:never-empty="false"
						:capitalize="false"
					/>
				</div>
			</Transition>
		</div>
		<div class="mt-4 flex w-full flex-row justify-between gap-2">
			<ButtonStyled @click="$emit('close')"
				><button @click="hideModal"><XIcon /> Cancel</button></ButtonStyled
			>
			<ButtonStyled color="brand">
				<button :disabled="!canContinue || loading" @click="continueForm">
					<span v-if="!loading">Continue</span>
					<span v-else>Loading…</span>
				</button>
			</ButtonStyled>
		</div>
	</NewModal>
	<div
		class="mb-4 flex w-full flex-col gap-2 rounded-2xl border-[1px] border-solid border-orange bg-bg-orange p-4 text-sm text-contrast"
	>
		<span class="flex gap-2 align-middle text-xl font-bold text-orange"
			><TriangleAlertIcon class="my-auto" /> Tax Form Required</span
		>
		<span class="text-md text-orange"
			>You have reached the annual withdrawal threshold of $600 USD and must complete a tax form
			before you can withdraw additional revenue.</span
		>
		<div>
			<ButtonStyled color="orange">
				<button @click="startTaxForm"><FileTextIcon /> Start Tax Form</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { FileTextIcon, TriangleAlertIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, Chips, NewModal } from '@modrinth/ui'
import { useAvalara1099, type FormRequestResponse } from '~/composables/avalara1099'

const taxFormModal = ref<InstanceType<typeof NewModal> | null>(null)
const auth = await useAuth()
console.log(auth.value.user)

const isUSCitizen = ref<'yes' | 'no' | null>(null)
const entityType = ref<'private-individual' | 'foreign-entity' | null>(null)

function hideModal() {
	taxFormModal.value?.hide()
}

const determinedFormType = computed(() => {
	if (isUSCitizen.value === 'yes') {
		return 'W-9'
	} else if (isUSCitizen.value === 'no' && entityType.value === 'private-individual') {
		return 'W-8 BEN'
	} else if (isUSCitizen.value === 'no' && entityType.value === 'foreign-entity') {
		return 'W-8 BEN-E'
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

async function startTaxForm(e: MouseEvent) {
	taxFormModal.value?.show(e)
}

const avalaraState = ref<ReturnType<typeof useAvalara1099> | null>(null)
const loading = computed(() =>
	avalaraState.value ? ((avalaraState.value as any).loading?.value ?? false) : false,
)

const shrinkModalBehindAvalara = ref(false)

async function continueForm() {
	if (!import.meta.client) return
	if (!determinedFormType.value) return

	shrinkModalBehindAvalara.value = true

	// TODO: Replace with actual response
	const response: FormRequestResponse = {}

	if (!avalaraState.value) {
		console.log('epic')
		avalaraState.value = useAvalara1099(response, {
			prefill: {
				name: 'Firstname Middlename Surname',
				email: (auth.value.user as any)?.email ?? '',
				account_number: (auth.value.user as any)?.id ?? '',
			},
		})
	}

	try {
		if (avalaraState.value) {
			await avalaraState.value.start()
			console.log(avalaraState.value.status)
		}
	} catch (e) {}

	shrinkModalBehindAvalara.value = false
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
