<template>
	<div id="license" data-project-field="license">
		<h2 class="settings-section-title">{{ formatMessage(messages.title) }}</h2>
		<section class="universal-card settings-content-card flex flex-col gap-6">
			<ProjectIssueCard
				target="modify_license"
				:field-action="licenseIssueAction"
				class="-mb-2 max-w-[600px]"
			/>

			<fieldset class="m-0 flex min-w-0 flex-col gap-2 border-0 p-0">
				<legend class="mb-2 font-semibold text-contrast">License type</legend>
				<div class="grid grid-cols-1 gap-2 sm:grid-cols-[repeat(3,minmax(0,250px))]">
					<RadioCard
						v-for="option in licenseTypes"
						:key="option.value"
						v-model="licenseType"
						name="project-license-type"
						max-width="250px"
						:value="option.value"
						:label="option.label"
						:description="option.description"
						:icon="option.icon"
						:disabled="saving || !hasPermission"
					/>
				</div>
			</fieldset>
			<div v-if="licenseType === 'standard'" class="flex min-w-0 max-w-[600px] flex-col gap-2">
				<label for="license-multiselect" class="w-fit text-lg font-semibold text-contrast">
					{{ formatMessage(messages.select) }}
				</label>

				<div class="flex min-w-0 flex-col gap-1.5">
					<Combobox
						id="license-multiselect"
						v-model="selectedLicense"
						:options="licenseOptions"
						:search-options="
							allLicenseOptions.filter(
								(option) => option.value && option.value !== 'All-Rights-Reserved',
							)
						"
						:display-value="licenseDisplayName(current.license) || undefined"
						:placeholder="formatMessage(messages.selectPlaceholder)"
						:search-placeholder="formatMessage(messages.selectPlaceholder)"
						:searchable="true"
						sync-with-selection
						select-search-text-on-focus
						:disabled="saving || !hasPermission"
						trigger-type="base"
						class="w-full max-w-[600px]"
					/>
					<ValidationMessage
						:check="licenseSelectionValidation"
						:project-field="project.license.id"
						:current-field="licenseId"
						class="mt-2"
					/>
					<ValidationMessage
						:check="[
							...saveValidation.forField('license'),
							...saveValidation.forField('source-availability', 'source'),
						]"
						class="mt-2"
					/>
				</div>
			</div>

			<div v-if="current.license.requiresOnlyOrLater" class="flex min-w-0 flex-col gap-2">
				<label for="or-later-checkbox" class="w-fit text-lg font-semibold text-contrast">
					{{ formatMessage(messages.later) }}
				</label>

				<Checkbox
					id="or-later-checkbox"
					v-model="current.allowOrLater"
					:disabled="saving || !hasPermission"
					:description="formatMessage(messages.allowLater)"
				>
					{{ formatMessage(messages.allowLater) }}
				</Checkbox>
				<p class="m-0 text-base text-secondary">
					{{ formatMessage(messages.laterDescription) }}
				</p>
			</div>

			<div
				v-if="current.license.friendly === 'Custom'"
				class="flex min-w-0 max-w-[600px] flex-col gap-2"
			>
				<label for="license-name" class="w-fit text-lg font-semibold text-contrast">
					{{ formatMessage(messages.name) }}
				</label>

				<div class="flex min-w-0 flex-col gap-2">
					<Input
						id="license-name"
						v-model="current.license.short"
						wrapper-class="w-full"
						:maxlength="128"
						:placeholder="formatMessage(messages.namePlaceholder)"
						required
						:disabled="saving || !hasPermission"
					/>
					<p class="m-0 text-base text-secondary">
						{{ formatMessage(messages.nameDescription) }}
					</p>
					<ValidationMessage :check="customNameMessage" />
					<ValidationMessage
						:check="
							saveValidation
								.forField('custom-license', 'license')
								.filter((message) => message.values?.missingName)
						"
					/>
				</div>
			</div>

			<div
				v-if="current.license.friendly"
				data-project-field="license-url"
				class="flex min-w-0 max-w-[600px] flex-col gap-2"
			>
				<label for="license-url" class="w-fit text-lg font-semibold text-contrast">
					{{ formatMessage(licenseUrlMessages.url) }}
					<span v-if="current.license.friendly !== 'Custom'" class="font-normal text-secondary">
						({{ formatMessage(messages.optionalLabel) }})
					</span>
				</label>

				<div class="flex min-w-0 flex-col gap-2">
					<Input
						id="license-url"
						v-model="current.licenseUrl"
						type="url"
						:maxlength="2048"
						:placeholder="
							formatMessage(
								current.license.friendly === 'Custom'
									? licenseUrlMessages.url
									: licenseUrlMessages.optionalUrl,
							)
						"
						:disabled="saving || !hasPermission || licenseId === 'LicenseRef-Unknown'"
						wrapper-class="w-full"
					/>
					<p class="m-0 text-base text-secondary">
						{{
							formatMessage(
								current.license.friendly === 'Custom'
									? licenseUrlMessages.customUrlDescription
									: licenseUrlMessages.urlDescription,
							)
						}}
					</p>
					<ValidationMessage :check="customUrlMessage" />
					<ValidationMessage
						:check="
							saveValidation
								.forField('custom-license', 'license')
								.filter((message) => message.values?.missingUrl)
						"
					/>
					<ValidationMessage
						:check="customLicenseValidation"
						:project-field="project.license.id"
						:current-field="licenseId"
					/>
					<ValidationMessage
						:check="effectiveLicenseCheck"
						:project-field="project.license.url ?? ''"
						:current-field="current.licenseUrl"
					/>
					<ValidationMessage
						:check="saveValidation.forField('license-url', 'license')"
						class="mt-2"
					/>
				</div>
			</div>
			<ValidationMessage
				:check="
					saveValidation.withoutFields([
						'license',
						['source-availability', 'source'],
						['license-url', 'license'],
						['custom-license', 'license'],
					])
				"
				class="my-4"
			/>
			<section v-if="licenseSummary" aria-labelledby="license-meaning-title">
				<h3 id="license-meaning-title" class="mb-3 mt-0 text-base font-semibold text-contrast">
					What it means
				</h3>
				<div class="rounded-xl border border-solid border-surface-4 bg-surface-2 p-4">
					<p class="mb-4 mt-0 leading-relaxed">{{ licenseSummary.description }}</p>
					<div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
						<div v-for="group in licenseSummaryGroups" :key="group.key">
							<h4 class="mb-2 mt-0 text-xs font-semibold text-contrast">{{ group.title }}</h4>
							<ul v-if="group.rules.length" class="m-0 flex list-none flex-col gap-2 p-0">
								<li v-for="rule in group.rules" :key="rule.label" class="flex items-start gap-2">
									<component
										:is="group.icon"
										aria-hidden="true"
										class="mt-0.5 size-4 shrink-0"
										:class="group.color"
									/>
									<span v-tooltip="rule.description" tabindex="0">{{ rule.label }}</span>
								</li>
							</ul>
							<p v-else class="m-0 text-sm text-secondary">
								{{
									licenseType === 'none' && group.key === 'permissions'
										? 'None granted by this selection.'
										: 'None listed.'
								}}
							</p>
						</div>
					</div>
				</div>
				<p class="mb-0 mt-2 text-xs text-secondary">
					This summary is provided by
					<a
						:href="licenseSummary.sourceUrl"
						target="_blank"
						rel="noopener noreferrer"
						class="text-link"
						>Choose a License</a
					>
					and is not legal advice.
				</p>
			</section>
			<section
				v-if="licenseType === 'standard' && !licenseId.startsWith('LicenseRef-')"
				aria-labelledby="license-text-title"
			>
				<h3 id="license-text-title" class="mb-3 mt-0 text-base font-semibold text-contrast">
					License text
				</h3>
				<div
					class="rounded-xl border border-solid border-surface-4 bg-surface-2 p-4"
					:aria-busy="loadingLicense"
				>
					<p v-if="loadingLicense" role="status" class="m-0 text-secondary">
						Loading license text…
					</p>
					<p v-else-if="licenseError" role="status" class="m-0 text-red">
						License text could not be retrieved.
					</p>
					<div
						v-else-if="licenseBody"
						role="region"
						tabindex="0"
						aria-label="License text"
						class="max-h-[192px] overflow-y-auto whitespace-pre-wrap break-words font-mono text-xs leading-relaxed text-secondary"
					>
						{{ licenseBody }}
					</div>
					<p v-else class="m-0 text-secondary">License text is not available for this license.</p>
				</div>
			</section>
		</section>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon, FileTextIcon, InfoIcon, LockIcon, ScaleIcon, XIcon } from '@modrinth/assets'
import {
	Checkbox,
	Combobox,
	type ComboboxOption,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	Input,
	RadioCard,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { defaultLicenseIds, isAdmin, TeamMemberPermission } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import ValidationMessage from '@/components/ValidationMessage.vue'
import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import { useProjectIssueFieldAction } from '~/composables/project-issue-field-action'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import { normalizeProjectUrl } from '~/helpers/project-url'
import { licenseUrlMessages } from '~/utils/license-messages'

import {
	getLicenseSummary,
	licenseSummaryRules,
	type LicenseSummaryCategory,
} from './license-summaries'

const { projectV2: project, currentMember, patchProjectV3 } = injectProjectPageContext()
const { labrinth } = injectModrinthClient()

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	custom: { id: 'project.settings.license.custom', defaultMessage: 'Custom' },
	allRights: {
		id: 'project.settings.license.all-rights',
		defaultMessage: 'All Rights Reserved/No License',
	},
	title: { id: 'project.settings.license.title', defaultMessage: 'License' },
	select: {
		id: 'project.settings.license.select',
		defaultMessage: 'License',
	},
	selectPlaceholder: {
		id: 'project.settings.license.select-placeholder',
		defaultMessage: 'Search licenses...',
	},
	later: {
		id: 'project.settings.license.later',
		defaultMessage: 'Later editions',
	},
	laterDescription: {
		id: 'project.settings.license.later-description',
		defaultMessage:
			'The license you selected has an "or later" clause. If you check this box, users may use your project under later editions of the license.',
	},
	allowLater: {
		id: 'project.settings.license.allow-later',
		defaultMessage: 'Allow later editions',
	},
	url: { id: 'project.settings.license.url', defaultMessage: 'License URL' },
	urlPlaceholder: {
		id: 'project.settings.license.url-placeholder',
		defaultMessage: 'License URL',
	},
	optionalLabel: {
		id: 'project.settings.license.optional-label',
		defaultMessage: 'optional',
	},
	urlDescription: {
		id: 'project.settings.license.url-description',
		defaultMessage:
			"The web location of the full license text. If you don't provide a link, the license text will be displayed instead.",
	},
	customUrlDescription: {
		id: 'project.settings.license.custom-url-description',
		defaultMessage: 'The web location of the full license text.',
	},
	name: { id: 'project.settings.license.name', defaultMessage: 'License name' },
	namePlaceholder: {
		id: 'project.settings.license.name-placeholder',
		defaultMessage: 'License name',
	},
	nameDescription: {
		id: 'project.settings.license.name-description',
		defaultMessage: 'The full name of the custom license.',
	},
	missingName: {
		id: 'project.settings.license.custom-name-required',
		defaultMessage: 'Enter a name for your custom license.',
	},
	invalidName: {
		id: 'project.settings.license.custom-name-invalid',
		defaultMessage: 'Custom license names can only contain letters, numbers, periods, and hyphens.',
	},
	missingUrl: {
		id: 'project.settings.license.missing-url',
		defaultMessage: 'Enter a URL to the full text of your custom license.',
	},
	updated: {
		id: 'project.settings.license.updated',
		defaultMessage: 'License updated',
	},
	updatedText: {
		id: 'project.settings.license.updated-text',
		defaultMessage: 'Your license has been updated.',
	},
})

function licenseDisplayName(license: { short: string; friendly: string }) {
	if (license.friendly === 'Custom') return formatMessage(messages.custom)
	if (license.short === 'All-Rights-Reserved') return formatMessage(messages.allRights)
	return license.friendly
}

interface LicenseOption {
	short: string
	friendly: string
	requiresOnlyOrLater?: boolean
}

const orLaterLicenseIds = new Set(['AGPL-3.0', 'LGPL-2.1', 'LGPL-3.0', 'GPL-2.0', 'GPL-3.0'])

const { data: spdxLicenses } = useQuery({
	queryKey: ['tags', 'licenses', 'v2'],
	queryFn: () => labrinth.tags_v2.getLicenses(),
	staleTime: 1000 * 60 * 60,
})

const availableLicenses = computed<LicenseOption[]>(() => {
	const licensesById = new Map(
		(spdxLicenses.value ?? []).map((license) => [license.short, license]),
	)
	const defaultIds = new Set<string>(defaultLicenseIds)
	return [
		{ friendly: 'Custom', short: '', requiresOnlyOrLater: false },
		...defaultLicenseIds.map((short) => ({
			friendly:
				short === 'All-Rights-Reserved'
					? 'All Rights Reserved/No License'
					: (licensesById.get(short)?.name ?? short),
			short,
			requiresOnlyOrLater: orLaterLicenseIds.has(short),
		})),
		...(spdxLicenses.value ?? [])
			.filter((license) => !defaultIds.has(license.short))
			.map((license) => ({ friendly: license.name, short: license.short })),
	]
})

const allLicenseOptions = computed<ComboboxOption<string>[]>(() =>
	availableLicenses.value.map((license) => ({
		value: license.short,
		label: license.short || licenseDisplayName(license),
		subLabel: license.short ? licenseDisplayName(license) : undefined,
		searchTerms: [
			license.short,
			license.short.replaceAll('-', ' '),
			license.friendly,
			license.friendly.replaceAll('-', ' '),
		],
	})),
)

const licenseOptions = computed(() => {
	const defaultIds = new Set<string>(defaultLicenseIds)
	return allLicenseOptions.value.filter(
		(license) =>
			license.value !== '' &&
			license.value !== 'All-Rights-Reserved' &&
			defaultIds.has(license.value),
	)
})

function getInitialLicense() {
	const oldLicenseId = project.value.license.id
	const trimmedLicenseId = oldLicenseId
		.replaceAll('-only', '')
		.replaceAll('-or-later', '')
		.replaceAll('LicenseRef-', '')

	if (oldLicenseId === 'LicenseRef-Unknown') {
		return {
			friendly: '',
			short: oldLicenseId.replaceAll('LicenseRef-', ''),
			requiresOnlyOrLater: false,
		}
	}
	if (oldLicenseId.startsWith('LicenseRef-') && oldLicenseId !== 'LicenseRef-All-Rights-Reserved') {
		return {
			friendly: 'Custom',
			short: oldLicenseId.slice('LicenseRef-'.length),
			requiresOnlyOrLater: false,
		}
	}

	return (
		availableLicenses.value.find((license) => license.short === trimmedLicenseId) ?? {
			friendly: project.value.license.name,
			short: trimmedLicenseId,
			requiresOnlyOrLater: orLaterLicenseIds.has(trimmedLicenseId),
		}
	)
}

const {
	saved,
	current,
	saving,
	hasChanges,
	reset,
	save: saveLicense,
} = useSavable(
	() => ({
		license: getInitialLicense(),
		licenseUrl: project.value.license.url ?? '',
		allowOrLater: project.value.license.id.includes('-or-later'),
	}),
	async () => {
		const payload: {
			license_id?: string
			license_url?: string | null
		} = {}

		if (licenseId.value !== project.value.license.id) {
			payload.license_id = licenseId.value
		}

		if (current.value.licenseUrl !== project.value.license.url) {
			payload.license_url = normalizeProjectUrl(current.value.licenseUrl) || null
		}

		await patchProjectV3(payload, true, true)
	},
)

const licenseSelectionValidation = useProjectNagMessages('license')
const customLicenseValidation = useProjectNagMessages('custom-license')
const effectiveLicenseCheck = useProjectNagMessages('license-url', 'license')

const selectedLicense = computed({
	get: () => (current.value.license.friendly === 'Custom' ? '' : current.value.license.short),
	set: (short: string) => {
		const license = availableLicenses.value.find((option) => option.short === short)
		if (license) current.value.license = license
	},
})

const isAdminUser = computed(() => isAdmin(currentMember.value?.user))
const hasPermission = computed(
	() =>
		isAdminUser.value ||
		Boolean((currentMember.value?.permissions ?? 0) & TeamMemberPermission.EDIT_DETAILS),
)

const saveValidation = useProjectSaveValidation(() => current.value)
const missingCustomName = computed(
	() => current.value.license.friendly === 'Custom' && !current.value.license.short.trim(),
)
const invalidCustomName = computed(
	() =>
		current.value.license.friendly === 'Custom' &&
		/[^a-zA-Z0-9.-]/.test(current.value.license.short.replaceAll(' ', '-')),
)
const missingCustomUrl = computed(
	() => current.value.license.friendly === 'Custom' && !current.value.licenseUrl.trim(),
)
const customNameMessage = computed(() =>
	invalidCustomName.value
		? { severity: 'error' as const, message: messages.invalidName }
		: project.value.status === 'processing' && missingCustomName.value
			? { severity: 'error' as const, message: messages.missingName }
			: null,
)
const customUrlMessage = computed(() =>
	project.value.status === 'processing' && missingCustomUrl.value
		? { severity: 'error' as const, message: messages.missingUrl }
		: null,
)
const canSave = computed(
	() =>
		hasPermission.value &&
		!missingCustomName.value &&
		!invalidCustomName.value &&
		!missingCustomUrl.value &&
		!saveValidation.messages.value.some((message) => message.severity === 'error'),
)

const licenseIssueAction = useProjectIssueFieldAction({
	draft: () => ({
		license: {
			...project.value.license,
			id: licenseId.value,
			url: normalizeProjectUrl(current.value.licenseUrl) || undefined,
		},
	}),
	canSave: () => canSave.value,
	saving,
	validation: saveValidation,
	save: async () => {
		const license_id = licenseId.value
		const submittedUrl = current.value.licenseUrl
		const license_url = normalizeProjectUrl(submittedUrl) || null
		if (
			license_id !== project.value.license.id ||
			(license_url ?? '') !== (project.value.license.url ?? '')
		) {
			await patchProjectV3({ license_id, license_url }, true, true)
		}
		if (current.value.licenseUrl === submittedUrl) current.value.licenseUrl = license_url ?? ''
	},
})

async function save() {
	if (!canSave.value || saving.value) return
	const submittedState = saveValidation.snapshot()
	try {
		await saveLicense()
		saveValidation.clear()
		addNotification({
			title: formatMessage(messages.updated),
			text: formatMessage(messages.updatedText),
			type: 'success',
		})
	} catch (error) {
		if (!saveValidation.capture(error, submittedState)) throw error
	}
}

const licenseId = computed(() => {
	let id = ''

	if (
		current.value.license.friendly === 'Custom' ||
		current.value.license.short === 'All-Rights-Reserved' ||
		current.value.license.short === 'Unknown'
	) {
		id += 'LicenseRef-'
	}

	id += current.value.license.short
	if (current.value.license.requiresOnlyOrLater) {
		id += current.value.allowOrLater ? '-or-later' : '-only'
	}

	if (current.value.license.friendly === 'Custom') {
		id = id.replaceAll(' ', '-')
	}

	return id
})
const licenseTypes = [
	{
		value: 'standard',
		icon: ScaleIcon,
		label: 'Standard license',
		description: 'Pick one of the commonly used licenses from our list.',
	},
	{
		value: 'none',
		icon: LockIcon,
		label: 'No license',
		description: 'You reserve all rights to your work unless you explicitly grant them.',
	},
	{
		value: 'custom',
		icon: FileTextIcon,
		label: 'Custom license',
		description: 'Link to your own license that is not on our list.',
	},
]
const licenseType = computed({
	get: () =>
		current.value.license.friendly === 'Custom'
			? 'custom'
			: current.value.license.short === 'All-Rights-Reserved'
				? 'none'
				: 'standard',
	set: (type: string) => {
		const license = availableLicenses.value.find((option) =>
			type === 'custom'
				? option.friendly === 'Custom'
				: type === 'none'
					? option.short === 'All-Rights-Reserved'
					: Boolean(option.short) && option.short !== 'All-Rights-Reserved',
		)
		if (license) {
			current.value.license = { ...license }
			current.value.allowOrLater = false
		}
	},
})
const licenseSummary = computed(() =>
	licenseType.value !== 'custom' ? getLicenseSummary(licenseId.value) : undefined,
)
const licenseSummaryGroups = computed(() => {
	const groups = [
		{ key: 'permissions', title: 'Permissions', icon: CheckIcon, color: 'text-brand' },
		{ key: 'limitations', title: 'Limitations', icon: XIcon, color: 'text-red' },
		{ key: 'conditions', title: 'Conditions', icon: InfoIcon, color: 'text-blue' },
	] satisfies {
		key: LicenseSummaryCategory
		title: string
		icon: typeof CheckIcon
		color: string
	}[]
	return groups.map((group) => ({
		...group,
		rules: (licenseSummary.value?.[group.key] ?? []).map(
			(tag) => licenseSummaryRules[group.key][tag],
		),
	}))
})

const {
	data: licenseBody,
	isPending: loadingLicense,
	isError: licenseError,
} = useQuery({
	queryKey: computed(() => ['license', 'v2', licenseId.value] as const),
	queryFn: async () => (await labrinth.tags_v2.getLicenseText(licenseId.value)).body,
	enabled: computed(
		() => licenseType.value === 'standard' && !licenseId.value.startsWith('LicenseRef-'),
	),
	staleTime: 1000 * 60 * 10,
})

defineExpose({ saved, current, saving, hasChanges, canSave, reset, save })
</script>
