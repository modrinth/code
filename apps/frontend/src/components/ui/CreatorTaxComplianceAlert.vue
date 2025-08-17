<template>
	<NewModal ref="taxFormModal" header="Submit Tax Form">
		<div
			class="card flex flex-col rounded-2xl border-[1px] border-solid !bg-button-bg p-2 text-contrast"
		>
			<span>
				We need to determine which type of form you need to fill out. There are three types of tax
				forms:
			</span>
			<ul>
				<li>W-9: Used for U.S. citizens to certify their taxpayer status</li>
				<li>W-8BEN: Used for foreign individuals to certify their foreign status</li>
				<li>W-8BEN-E: Used for foreign entities* to certify their foreign status</li>
			</ul>
			<span>
				Modrinth has a legal obligation to share these forms with the IRS after you meet the annual
				threshold of $600 USD.
			</span>
			<span class="mt-2 text-xs italic"
				>* A foreign entity means a business entity organized outside the United States (such as a
				non-US corporation, partnership, or LLC).</span
			>
		</div>
		<div class="mt-2 flex flex-col gap-4">
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
		<div class="mt-4 flex w-full flex-row justify-between">
			<ButtonStyled @click="$emit('close')"
				><button><XIcon /> Cancel</button></ButtonStyled
			>
			<ButtonStyled color="brand" :disabled="!canContinue">
				<button>Continue</button>
			</ButtonStyled>
		</div>
	</NewModal>
	<div
		v-if="shouldShow"
		class="mb-4 flex w-full flex-col gap-2 rounded-2xl border-[1px] border-solid border-orange bg-bg-orange p-4 text-sm text-contrast"
	>
		<span class="flex gap-2 align-middle text-xl font-bold text-orange"
			><TriangleAlertIcon class="my-auto" /> Tax Form Required</span
		>
		<span class="text-md text-orange"
			>You have reached the annual withdrawal threshold of $600 USD and must complete a tax form
			before you can withdraw additional funds.</span
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

const taxFormModal = ref<InstanceType<typeof NewModal> | null>(null)

const isUSCitizen = ref<'yes' | 'no' | null>(null)
const entityType = ref<'private-individual' | 'foreign-entity' | null>(null)

// const props = defineProps<{
// 	totalAnnualWithdrawal: number
// 	taxComplianceFilled: 'w9' | 'w8-ben' | 'w8-ben-e' | null
// }>()

const shouldShow = computed(() => {
	return true
	// return props.totalAnnualWithdrawal >= 600 && !props.taxComplianceFilled
})

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

watch(isUSCitizen, (newValue) => {
	if (newValue === 'yes') {
		entityType.value = null
	}
})
</script>
