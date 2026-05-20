<template>
	<div class="flex flex-col gap-3 sm:gap-4">
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
				<StyledInput
					v-model="formData.businessName"
					:placeholder="formatMessage(formFieldPlaceholders.businessNamePlaceholder)"
					autocomplete="organization"
					wrapper-class="w-full"
				/>
			</div>

			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(formFieldLabels.email) }}
						<span class="text-red">*</span>
					</span>
				</label>
				<StyledInput
					v-model="formData.email"
					type="email"
					:placeholder="formatMessage(formFieldPlaceholders.emailPlaceholder)"
					autocomplete="email"
					wrapper-class="w-full"
				/>
			</div>

			<div v-if="entityType === 'individual'" class="flex flex-col gap-6">
				<div class="flex flex-col gap-3 sm:flex-row sm:gap-4">
					<div class="flex flex-1 flex-col gap-2.5">
						<label>
							<span class="text-md font-semibold text-contrast">
								{{ formatMessage(formFieldLabels.firstName) }}
								<span class="text-red">*</span>
							</span>
						</label>
						<StyledInput
							v-model="formData.firstName"
							:placeholder="formatMessage(formFieldPlaceholders.firstNamePlaceholder)"
							autocomplete="given-name"
							wrapper-class="w-full"
						/>
					</div>
					<div class="flex flex-1 flex-col gap-2.5">
						<label>
							<span class="text-md font-semibold text-contrast">
								{{ formatMessage(formFieldLabels.lastName) }}
								<span class="text-red">*</span>
							</span>
						</label>
						<StyledInput
							v-model="formData.lastName"
							:placeholder="formatMessage(formFieldPlaceholders.lastNamePlaceholder)"
							autocomplete="family-name"
							wrapper-class="w-full"
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
					<StyledInput
						v-model="formData.dateOfBirth"
						type="date"
						:max="maxDate"
						autocomplete="bday"
						wrapper-class="w-full"
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
				<StyledInput
					v-model="formData.physicalAddress.address1"
					:placeholder="formatMessage(formFieldPlaceholders.addressPlaceholder)"
					autocomplete="address-line1"
					wrapper-class="w-full"
				/>
			</div>

			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(formFieldLabels.addressLine2) }}
					</span>
				</label>
				<StyledInput
					v-model="formData.physicalAddress.address2"
					:placeholder="formatMessage(formFieldPlaceholders.address2Placeholder)"
					autocomplete="address-line2"
					wrapper-class="w-full"
				/>
			</div>

			<div class="flex flex-col gap-3 sm:flex-row sm:gap-4">
				<div class="flex flex-1 flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(formFieldLabels.city) }}
							<span class="text-red">*</span>
						</span>
					</label>
					<StyledInput
						v-model="formData.physicalAddress.city"
						:placeholder="formatMessage(formFieldPlaceholders.cityPlaceholder)"
						autocomplete="address-level2"
						wrapper-class="w-full"
					/>
				</div>
				<div class="flex flex-1 flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(formFieldLabels.stateProvince) }}
							<span class="text-red">*</span>
						</span>
					</label>
					<Combobox
						v-if="subdivisionOptions.length > 0"
						v-model="formData.physicalAddress.state"
						:options="subdivisionOptions"
						:placeholder="formatMessage(formFieldPlaceholders.statePlaceholder)"
						searchable
						search-placeholder="Search subdivisions..."
					/>
					<StyledInput
						v-else
						v-model="formData.physicalAddress.state"
						:placeholder="formatMessage(formFieldPlaceholders.statePlaceholder)"
						autocomplete="address-level1"
						wrapper-class="w-full"
					/>
				</div>
			</div>

			<div class="flex flex-col gap-3 sm:flex-row sm:gap-4">
				<div class="flex flex-1 flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(formFieldLabels.postalCode) }}
							<span class="text-red">*</span>
						</span>
					</label>
					<StyledInput
						v-model="formData.physicalAddress.zip"
						:placeholder="formatMessage(formFieldPlaceholders.postalCodePlaceholder)"
						autocomplete="postal-code"
						wrapper-class="w-full"
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
import {
	Chips,
	Combobox,
	defineMessages,
	formFieldLabels,
	formFieldPlaceholders,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
// TODO: Switch to using Muralpay's improved endpoint when it's available.
import iso3166 from 'iso-3166-2'

import { useFormattedCountries } from '@/composables/country.ts'
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

const countryOptions = useFormattedCountries()

const subdivisionOptions = computed(() => {
	const selectedCountry = formData.value.physicalAddress.country
	if (!selectedCountry) return []

	const country = iso3166.country(selectedCountry)
	if (!country) return []

	return Object.entries(country.sub)
		.map(([code, sub]) => ({
			value: code.split('-').slice(1).join('-'),
			label: sub.name,
		}))
		.sort((a, b) => a.label.localeCompare(b.label))
})

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
		defaultMessage: 'Are you withdrawing as an individual or a business?',
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
