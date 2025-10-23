<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.entityQuestion) }}
					<span class="text-red">*</span>
				</span>
			</label>
			<Chips
				v-model="entityType"
				:items="['individual', 'business']"
				:format-label="
					(item: string) =>
						item === 'individual'
							? formatMessage(messages.privateIndividual)
							: formatMessage(messages.businessEntity)
				"
				:never-empty="false"
				:capitalize="false"
			/>
			<span class="leading-tight text-primary">
				{{ formatMessage(messages.entityDescription) }}
			</span>
		</div>

		<div v-if="entityType" class="flex flex-col gap-4">
			<div v-if="entityType === 'business'" class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(formFieldLabels.businessName) }}
						<span class="text-red">*</span>
					</span>
				</label>
				<input
					v-model="formData.businessName"
					type="text"
					:placeholder="formatMessage(formFieldPlaceholders.businessNamePlaceholder)"
					autocomplete="organization"
					class="w-full rounded-[14px] bg-surface-4 px-4 py-2.5 text-contrast placeholder:text-secondary"
				/>
			</div>

			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(formFieldLabels.email) }}
						<span class="text-red">*</span>
					</span>
				</label>
				<input
					v-model="formData.email"
					type="email"
					:placeholder="formatMessage(formFieldPlaceholders.emailPlaceholder)"
					autocomplete="email"
					class="w-full rounded-[14px] bg-surface-4 px-4 py-2.5 text-contrast placeholder:text-secondary"
				/>
			</div>

			<div v-if="entityType === 'individual'" class="flex flex-col gap-6">
				<div class="flex gap-4">
					<div class="flex flex-1 flex-col gap-2.5">
						<label>
							<span class="text-md font-semibold text-contrast">
								{{ formatMessage(formFieldLabels.firstName) }}
								<span class="text-red">*</span>
							</span>
						</label>
						<input
							v-model="formData.firstName"
							type="text"
							:placeholder="formatMessage(formFieldPlaceholders.firstNamePlaceholder)"
							autocomplete="given-name"
							class="w-full rounded-[14px] bg-surface-4 px-4 py-2.5 text-contrast placeholder:text-secondary"
						/>
					</div>
					<div class="flex flex-1 flex-col gap-2.5">
						<label>
							<span class="text-md font-semibold text-contrast">
								{{ formatMessage(formFieldLabels.lastName) }}
								<span class="text-red">*</span>
							</span>
						</label>
						<input
							v-model="formData.lastName"
							type="text"
							:placeholder="formatMessage(formFieldPlaceholders.lastNamePlaceholder)"
							autocomplete="family-name"
							class="w-full rounded-[14px] bg-surface-4 px-4 py-2.5 text-contrast placeholder:text-secondary"
						/>
					</div>
				</div>

				<div class="flex flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(formFieldLabels.dateOfBirth) }}
							<span class="text-red">*</span>
						</span>
					</label>
					<input
						v-model="formData.dateOfBirth"
						type="date"
						:max="maxDate"
						autocomplete="bday"
						class="w-full rounded-[14px] bg-surface-4 px-4 py-2.5 text-contrast placeholder:text-secondary"
					/>
				</div>
			</div>

			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(formFieldLabels.addressLine) }}
						<span class="text-red">*</span>
					</span>
				</label>
				<input
					v-model="formData.physicalAddress.address1"
					type="text"
					:placeholder="formatMessage(formFieldPlaceholders.addressPlaceholder)"
					autocomplete="address-line1"
					class="w-full rounded-[14px] bg-surface-4 px-4 py-2.5 text-contrast placeholder:text-secondary"
				/>
			</div>

			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(formFieldLabels.addressLine2) }}
					</span>
				</label>
				<input
					v-model="formData.physicalAddress.address2"
					type="text"
					:placeholder="formatMessage(formFieldPlaceholders.address2Placeholder)"
					autocomplete="address-line2"
					class="w-full rounded-[14px] bg-surface-4 px-4 py-2.5 text-contrast placeholder:text-secondary"
				/>
			</div>

			<div class="flex gap-4">
				<div class="flex flex-1 flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(formFieldLabels.city) }}
							<span class="text-red">*</span>
						</span>
					</label>
					<input
						v-model="formData.physicalAddress.city"
						type="text"
						:placeholder="formatMessage(formFieldPlaceholders.cityPlaceholder)"
						autocomplete="address-level2"
						class="w-full rounded-[14px] bg-surface-4 px-4 py-2.5 text-contrast placeholder:text-secondary"
					/>
				</div>
				<div class="flex flex-1 flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(formFieldLabels.stateProvince) }}
							<span class="text-red">*</span>
						</span>
					</label>
					<input
						v-model="formData.physicalAddress.state"
						type="text"
						:placeholder="formatMessage(formFieldPlaceholders.statePlaceholder)"
						autocomplete="address-level1"
						class="w-full rounded-[14px] bg-surface-4 px-4 py-2.5 text-contrast placeholder:text-secondary"
					/>
				</div>
			</div>

			<div class="flex gap-4">
				<div class="flex flex-1 flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(formFieldLabels.postalCode) }}
							<span class="text-red">*</span>
						</span>
					</label>
					<input
						v-model="formData.physicalAddress.zip"
						type="text"
						:placeholder="formatMessage(formFieldPlaceholders.postalCodePlaceholder)"
						autocomplete="postal-code"
						class="w-full rounded-[14px] bg-surface-4 px-4 py-2.5 text-contrast placeholder:text-secondary"
					/>
				</div>
				<div class="flex flex-1 flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(formFieldLabels.country) }}
							<span class="text-red">*</span>
						</span>
					</label>
					<Combobox
						v-model="formData.physicalAddress.country"
						:options="countryOptions"
						:placeholder="formatMessage(formFieldPlaceholders.countryPlaceholder)"
						searchable
						search-placeholder="Search countries..."
					/>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Chips, Combobox, formFieldLabels, formFieldPlaceholders } from '@modrinth/ui'
import { useVIntl } from '@vintl/vintl'
import { all } from 'iso-3166-1'

import { useWithdrawContext } from '@/providers/creator-withdraw.ts'

const { withdrawData } = useWithdrawContext()
const { formatMessage } = useVIntl()

const providerData = withdrawData.value.providerData
const existingKycData = providerData.type === 'muralpay' ? providerData.kycData : null

const entityType = ref<'individual' | 'business' | null>(existingKycData?.type ?? null)

interface PayoutRecipientInfoMerged {
	email: string
	firstName?: string
	lastName?: string
	dateOfBirth?: string
	businessName?: string
	physicalAddress: {
		address1: string
		address2: string | null
		country: string
		state: string
		city: string
		zip: string
	}
}

const auth = await useAuth()

const formData = ref<PayoutRecipientInfoMerged>({
	email: existingKycData?.email ?? `${(auth.value.user as any)?.email}`,
	firstName: existingKycData?.type === 'individual' ? existingKycData.firstName : '',
	lastName: existingKycData?.type === 'individual' ? existingKycData.lastName : '',
	dateOfBirth: existingKycData?.type === 'individual' ? existingKycData.dateOfBirth : '',
	businessName: existingKycData?.type === 'business' ? existingKycData.name : '',
	physicalAddress: {
		address1: existingKycData?.physicalAddress?.address1 ?? '',
		address2: existingKycData?.physicalAddress?.address2 ?? null,
		country:
			existingKycData?.physicalAddress?.country ?? withdrawData.value.selection.country?.id ?? '',
		state: existingKycData?.physicalAddress?.state ?? '',
		city: existingKycData?.physicalAddress?.city ?? '',
		zip: existingKycData?.physicalAddress?.zip ?? '',
	},
})

const maxDate = computed(() => {
	const today = new Date()
	const year = today.getFullYear() - 18
	const month = String(today.getMonth() + 1).padStart(2, '0')
	const day = String(today.getDate()).padStart(2, '0')
	return `${year}-${month}-${day}`
})

// Map for countries with excessively long names to prevent overflow
const shortCountryNames: Record<string, string> = {
	GB: 'United Kingdom',
	US: 'United States',
	SH: 'Saint Helena',
	GS: 'South Georgia',
	KP: "Dem. People's Rep. of Korea",
	UM: 'US Minor Outlying Islands',
	VI: 'US Virgin Islands',
	VE: 'Venezuela',
	HM: 'Heard & McDonald Islands',
	BQ: 'Bonaire, Sint Eustatius & Saba',
	LA: "Lao People's Dem. Rep.",
	VC: 'St. Vincent & Grenadines',
}

const countryOptions = computed(() =>
	all().map((x) => {
		let label = x.country

		// Apply custom short name if available
		if (shortCountryNames[x.alpha2]) {
			label = `${shortCountryNames[x.alpha2]} (${x.alpha2})`
		}
		// Special case for Taiwan
		else if (x.alpha2 === 'TW') {
			label = 'Taiwan'
		}
		// For other long names (>30 chars), append country code
		else if (x.country.length > 30) {
			label = `${x.country} (${x.alpha2})`
		}

		return {
			value: x.alpha2,
			label,
		}
	}),
)

watch(
	[entityType, formData],
	() => {
		if (!entityType.value) {
			if (withdrawData.value.providerData.type === 'muralpay') {
				withdrawData.value.providerData.kycData = null as any
			}
			return
		}

		if (withdrawData.value.providerData.type !== 'muralpay') return

		if (entityType.value === 'individual') {
			if (formData.value.dateOfBirth) {
				withdrawData.value.providerData.kycData = {
					type: 'individual',
					firstName: formData.value.firstName || '',
					lastName: formData.value.lastName || '',
					email: formData.value.email,
					dateOfBirth: formData.value.dateOfBirth,
					physicalAddress: {
						address1: formData.value.physicalAddress.address1,
						address2: formData.value.physicalAddress.address2 || undefined,
						country: formData.value.physicalAddress.country,
						state: formData.value.physicalAddress.state,
						city: formData.value.physicalAddress.city,
						zip: formData.value.physicalAddress.zip,
					},
				}
			}
		} else {
			withdrawData.value.providerData.kycData = {
				type: 'business',
				name: formData.value.businessName || '',
				email: formData.value.email,
				physicalAddress: {
					address1: formData.value.physicalAddress.address1,
					address2: formData.value.physicalAddress.address2 || undefined,
					country: formData.value.physicalAddress.country,
					state: formData.value.physicalAddress.state,
					city: formData.value.physicalAddress.city,
					zip: formData.value.physicalAddress.zip,
				},
			}
		}
	},
	{ deep: true },
)

const messages = defineMessages({
	entityQuestion: {
		id: 'dashboard.creator-withdraw-modal.kyc.entity-question',
		defaultMessage: 'Are you a withdrawing as an individual or business?',
	},
	entityDescription: {
		id: 'dashboard.creator-withdraw-modal.kyc.entity-description',
		defaultMessage:
			'A business entity refers to a registered organization such as a corporation, partnership, or LLC.',
	},
	privateIndividual: {
		id: 'dashboard.creator-withdraw-modal.kyc.private-individual',
		defaultMessage: 'Private individual',
	},
	businessEntity: {
		id: 'dashboard.creator-withdraw-modal.kyc.business-entity',
		defaultMessage: 'Business entity',
	},
})
</script>
