<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.entityQuestion) }}
					<span class="text-brand-red">*</span>
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
						{{ formatMessage(messages.businessName) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<input
					v-model="formData.businessName"
					type="text"
					:placeholder="formatMessage(messages.businessNamePlaceholder)"
					class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
				/>
			</div>

			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(messages.email) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<input
					v-model="formData.email"
					type="email"
					:placeholder="formatMessage(messages.emailPlaceholder)"
					class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
				/>
			</div>

			<div v-if="entityType === 'individual'" class="flex flex-col gap-6">
				<div class="flex gap-4">
					<div class="flex flex-1 flex-col gap-2.5">
						<label>
							<span class="text-md font-semibold text-contrast">
								{{ formatMessage(messages.firstName) }}
								<span class="text-brand-red">*</span>
							</span>
						</label>
						<input
							v-model="formData.firstName"
							type="text"
							:placeholder="formatMessage(messages.firstNamePlaceholder)"
							class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
						/>
					</div>
					<div class="flex flex-1 flex-col gap-2.5">
						<label>
							<span class="text-md font-semibold text-contrast">
								{{ formatMessage(messages.lastName) }}
								<span class="text-brand-red">*</span>
							</span>
						</label>
						<input
							v-model="formData.lastName"
							type="text"
							:placeholder="formatMessage(messages.lastNamePlaceholder)"
							class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
						/>
					</div>
				</div>

				<div class="flex flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(messages.dateOfBirth) }}
							<span class="text-brand-red">*</span>
						</span>
					</label>
					<input
						v-model="formData.dateOfBirth"
						type="date"
						:max="maxDate"
						class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
					/>
				</div>
			</div>

			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(messages.addressLine) }}
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<input
					v-model="formData.physicalAddress.address1"
					type="text"
					:placeholder="formatMessage(messages.addressPlaceholder)"
					class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
				/>
			</div>

			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast">
						{{ formatMessage(messages.addressLine2) }}
					</span>
				</label>
				<input
					v-model="formData.physicalAddress.address2"
					type="text"
					:placeholder="formatMessage(messages.address2Placeholder)"
					class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
				/>
			</div>

			<div class="flex gap-4">
				<div class="flex flex-1 flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(messages.city) }}
							<span class="text-brand-red">*</span>
						</span>
					</label>
					<input
						v-model="formData.physicalAddress.city"
						type="text"
						:placeholder="formatMessage(messages.cityPlaceholder)"
						class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
					/>
				</div>
				<div class="flex flex-1 flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(messages.stateProvince) }}
							<span class="text-brand-red">*</span>
						</span>
					</label>
					<input
						v-model="formData.physicalAddress.state"
						type="text"
						:placeholder="formatMessage(messages.statePlaceholder)"
						class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
					/>
				</div>
			</div>

			<div class="flex gap-4">
				<div class="flex flex-1 flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(messages.postalCode) }}
							<span class="text-brand-red">*</span>
						</span>
					</label>
					<input
						v-model="formData.physicalAddress.zip"
						type="text"
						:placeholder="formatMessage(messages.postalCodePlaceholder)"
						class="bg-raised w-full rounded-[14px] px-4 py-2.5 text-contrast placeholder:text-secondary"
					/>
				</div>
				<div class="flex flex-1 flex-col gap-2.5">
					<label>
						<span class="text-md font-semibold text-contrast">
							{{ formatMessage(messages.country) }}
							<span class="text-brand-red">*</span>
						</span>
					</label>
					<Combobox
						v-model="formData.physicalAddress.country"
						:options="countryOptions"
						:placeholder="formatMessage(messages.countryPlaceholder)"
						searchable
						search-placeholder="Search countries..."
					/>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Chips, Combobox } from '@modrinth/ui'
import { useVIntl } from '@vintl/vintl'
import { all } from 'iso-3166-1'

import { useWithdrawContext } from '@/providers/creator-withdraw.ts'

const withdrawContext = useWithdrawContext()
const { formatMessage } = useVIntl()

const entityType = ref<'individual' | 'business' | null>(
	withdrawContext.withdrawData.value.kycData?.type ?? null,
)

// Scroll fade is now handled automatically by the scroll overflow indicator composable

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

// if user has switched stages use what was in withdraw context
const existingKycData = withdrawContext.withdrawData.value.kycData
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
			existingKycData?.physicalAddress?.country ??
			withdrawContext.withdrawData.value.selectedCountry?.id ??
			'',
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

const countryOptions = computed(() =>
	all().map((x) => ({
		value: x.alpha2,
		label: x.alpha2 === 'TW' ? 'Taiwan' : x.country,
	})),
)

watch(
	[entityType, formData],
	() => {
		if (!entityType.value) {
			withdrawContext.withdrawData.value.kycData = null
			return
		}

		if (entityType.value === 'individual') {
			if (formData.value.dateOfBirth) {
				withdrawContext.withdrawData.value.kycData = {
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
			withdrawContext.withdrawData.value.kycData = {
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
	email: {
		id: 'dashboard.creator-withdraw-modal.kyc.email',
		defaultMessage: 'Email',
	},
	emailPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.kyc.email-placeholder',
		defaultMessage: 'Enter email address',
	},
	firstName: {
		id: 'dashboard.creator-withdraw-modal.kyc.first-name',
		defaultMessage: 'First name',
	},
	firstNamePlaceholder: {
		id: 'dashboard.creator-withdraw-modal.kyc.first-name-placeholder',
		defaultMessage: 'Enter first name',
	},
	lastName: {
		id: 'dashboard.creator-withdraw-modal.kyc.last-name',
		defaultMessage: 'Last name',
	},
	lastNamePlaceholder: {
		id: 'dashboard.creator-withdraw-modal.kyc.last-name-placeholder',
		defaultMessage: 'Enter last name',
	},
	dateOfBirth: {
		id: 'dashboard.creator-withdraw-modal.kyc.date-of-birth',
		defaultMessage: 'Date of birth',
	},
	businessName: {
		id: 'dashboard.creator-withdraw-modal.kyc.business-name',
		defaultMessage: 'Business name',
	},
	businessNamePlaceholder: {
		id: 'dashboard.creator-withdraw-modal.kyc.business-name-placeholder',
		defaultMessage: 'Enter business name',
	},
	addressLine: {
		id: 'dashboard.creator-withdraw-modal.kyc.address-line',
		defaultMessage: 'Address line',
	},
	addressPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.kyc.address-placeholder',
		defaultMessage: 'Enter address',
	},
	addressLine2: {
		id: 'dashboard.creator-withdraw-modal.kyc.address-line-2',
		defaultMessage: 'Address line 2 (optional)',
	},
	address2Placeholder: {
		id: 'dashboard.creator-withdraw-modal.kyc.address-2-placeholder',
		defaultMessage: 'Apartment, suite, etc.',
	},
	city: {
		id: 'dashboard.creator-withdraw-modal.kyc.city',
		defaultMessage: 'City',
	},
	cityPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.kyc.city-placeholder',
		defaultMessage: 'Enter city',
	},
	stateProvince: {
		id: 'dashboard.creator-withdraw-modal.kyc.state-province',
		defaultMessage: 'State/province',
	},
	statePlaceholder: {
		id: 'dashboard.creator-withdraw-modal.kyc.state-placeholder',
		defaultMessage: 'Enter state/province',
	},
	postalCode: {
		id: 'dashboard.creator-withdraw-modal.kyc.postal-code',
		defaultMessage: 'Postal code',
	},
	postalCodePlaceholder: {
		id: 'dashboard.creator-withdraw-modal.kyc.postal-code-placeholder',
		defaultMessage: 'Enter postal code',
	},
	country: {
		id: 'dashboard.creator-withdraw-modal.kyc.country',
		defaultMessage: 'Country',
	},
	countryPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.kyc.country-placeholder',
		defaultMessage: 'Select country',
	},
})
</script>
