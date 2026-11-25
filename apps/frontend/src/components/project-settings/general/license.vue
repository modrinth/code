<template>
	<SettingsSection
		id="license"
		data-project-field="license"
		:title="formatMessage(messages.title)"
		:subtitle="formatMessage(messages.licenseHelp, { contentType: projectCopyType(projectV3) })"
	>
		<section class="universal-card settings-content-card flex flex-col gap-6">
			<ProjectIssueCard
				target="modify_license"
				:field-action="licenseIssueAction"
				class="-mb-2 max-w-[600px]"
			/>

			<Field :label="formatMessage(messages.licenseType)">
				<template #default="{ labelId }">
					<div
						role="radiogroup"
						:aria-labelledby="labelId"
						class="grid grid-cols-1 gap-2 sm:grid-cols-[repeat(3,minmax(0,250px))]"
					>
						<RadioCard
							v-for="option in licenseTypes"
							:key="option.value"
							v-model="licenseType"
							name="project-license-type"
							max-width="250px"
							:value="option.value"
							:label="formatMessage(option.label)"
							:description="
								formatMessage(option.description, { contentType: projectCopyType(projectV3) })
							"
							:icon="option.icon"
							:disabled="saving || !hasPermission"
						/>
					</div>
				</template>
				<template #bottom>
					<ValidationMessage
						v-if="licenseType === ''"
						:check="licenseSelectionValidation"
						:project-field="project.license.id"
						:current-field="licenseId"
						class="max-w-[600px]"
					/>
				</template>
			</Field>
			<Button
				v-if="
					licenseType === 'none' &&
					saved.license.short === 'All-Rights-Reserved' &&
					saved.licenseUrl
				"
				type="outlined"
				class="self-start"
				:disabled="saving || !hasPermission || !current.licenseUrl"
				@click="clearLicenseUrl"
			>
				{{ formatMessage(messages.removeUrl) }}
			</Button>
			<Field
				v-if="licenseType === 'standard'"
				:label="formatMessage(messages.select)"
				:input-id="licenseSelectId"
				class="max-w-[600px]"
			>
				<Combobox
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
					:search-input-id="licenseSelectId"
					sync-with-selection
					select-search-text-on-focus
					:disabled="saving || !hasPermission"
					trigger-type="base"
					:show-chevron="false"
					search-autocomplete="off"
					class="w-full max-w-[384px]"
				/>
				<template #bottom>
					<ValidationMessage
						:check="licenseSelectionValidation"
						:project-field="project.license.id"
						:current-field="licenseId || project.license.id"
					/>
					<ValidationMessage
						:check="[
							...saveValidation.forField('license'),
							...saveValidation.forField('source-availability', 'source'),
						]"
					/>
				</template>
			</Field>

			<div
				v-if="current.license.requiresOnlyOrLater"
				class="flex min-w-0 max-w-[600px] flex-col gap-2"
			>
				<Checkbox
					v-model="current.allowOrLater"
					:disabled="saving || !hasPermission"
					:description="formatMessage(messages.allowLater)"
				>
					{{ formatMessage(messages.allowLater) }}
				</Checkbox>
				<p class="m-0 leading-tight text-secondary">
					{{ formatMessage(messages.laterDescription) }}
				</p>
			</div>

			<Field
				v-if="current.license.friendly === 'Custom'"
				:label="formatMessage(messages.name)"
				:input-id="licenseNameId"
				class="max-w-[600px]"
			>
				<Input
					:id="licenseNameId"
					v-model="current.license.short"
					wrapper-class="w-full"
					:maxlength="128"
					:placeholder="formatMessage(messages.namePlaceholder)"
					required
					:disabled="saving || !hasPermission"
				/>
				<template #bottom>
					<ValidationMessage :check="customNameMessage" />
					<ValidationMessage
						:check="
							saveValidation
								.forField('custom-license', 'license')
								.filter((message) => message.values?.missingName)
						"
					/>
				</template>
			</Field>

			<div
				v-if="licenseType === 'custom' || (licenseType === 'standard' && selectedLicense)"
				data-project-field="license-url"
				class="flex min-w-0 max-w-[600px] flex-col gap-2"
			>
				<Checkbox
					v-if="licenseType === 'standard'"
					v-model="current.useStandardLicenseUrl"
					:disabled="saving || !hasPermission"
					:label="formatMessage(messages.linkOwnCopy)"
				/>
				<Field
					v-if="licenseType !== 'standard' || current.useStandardLicenseUrl"
					:label="formatMessage(messages.url)"
					:input-id="licenseUrlId"
				>
					<template v-if="licenseType === 'standard'" #label>
						<span class="sr-only">{{ formatMessage(messages.url) }}</span>
					</template>
					<Input
						:id="licenseUrlId"
						v-model="current.licenseUrl"
						type="url"
						:maxlength="2048"
						:placeholder="formatMessage(messages.urlPlaceholder)"
						required
						:disabled="saving || !hasPermission"
						:wrapper-class="licenseType === 'standard' ? 'w-full max-w-[384px]' : 'w-full'"
					/>
					<template #subtext>
						{{
							licenseType === 'standard'
								? formatMessage(messages.standardUrlHelp, {
										license: licenseDisplayName(current.license),
									})
								: formatMessage(messages.customUrlHelp, {
										contentType: projectCopyType(projectV3),
									})
						}}
					</template>
					<template #bottom>
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
						<ValidationMessage :check="saveValidation.forField('license-url', 'license')" />
					</template>
				</Field>
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
			<SettingsSection
				v-if="licenseSummary"
				:title="formatMessage(messages.whatItMeans)"
				:title-id="licenseMeaningTitleId"
				heading="h3"
			>
				<div class="rounded-xl border border-solid border-surface-4 bg-surface-2 p-4">
					<p class="mb-4 mt-0 leading-normal">{{ licenseSummary.description }}</p>
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
										? formatMessage(messages.noneGranted)
										: formatMessage(messages.noneListed)
								}}
							</p>
						</div>
					</div>
				</div>
				<p class="mb-0 mt-2 leading-tight text-secondary">
					<IntlFormatted :message-id="messages.summaryAttribution">
						<template #source-link="{ children }">
							<a
								:href="licenseSummary.sourceUrl"
								target="_blank"
								rel="noopener noreferrer"
								class="text-link"
							>
								<component :is="() => children" />
							</a>
						</template>
					</IntlFormatted>
				</p>
			</SettingsSection>
			<SettingsSection
				v-if="showLicenseText"
				:title="formatMessage(messages.licenseText)"
				:title-id="licenseTextTitleId"
				heading="h3"
			>
				<div
					class="group/license-text relative rounded-xl border border-solid border-surface-4 bg-surface-2 px-1 py-4"
					:aria-busy="loadingLicense"
				>
					<p v-if="loadingLicense" role="status" class="m-0 text-secondary">
						{{ formatMessage(messages.loadingLicenseText) }}
					</p>
					<p v-else-if="licenseError" role="status" class="m-0 text-red">
						{{ formatMessage(messages.licenseTextError) }}
					</p>
					<template v-else-if="licenseBody">
						<div
							role="region"
							tabindex="0"
							:aria-label="formatMessage(messages.licenseText)"
							class="max-h-[192px] overflow-y-auto whitespace-pre-wrap break-words rounded-2xl px-3 font-mono text-xs leading-relaxed text-secondary"
						>
							{{ licenseBody }}
						</div>
						<IconButton
							type="quiet"
							:label="formatMessage(messages.expandLicenseText)"
							aria-haspopup="dialog"
							class="!absolute right-4 top-2 opacity-0 transition-opacity group-focus-within/license-text:opacity-100 group-hover/license-text:opacity-100 motion-reduce:transition-none [@media(hover:none)]:opacity-100"
							@click="licenseTextModal?.show($event)"
						>
							<ExpandIcon aria-hidden="true" />
						</IconButton>
					</template>
					<p v-else class="m-0 text-secondary">
						{{ formatMessage(messages.licenseTextUnavailable) }}
					</p>
				</div>
				<NewModal
					ref="licenseTextModal"
					:header="formatMessage(messages.licenseText)"
					width="60rem"
					style="
						box-sizing: border-box;
						height: fit-content;
						max-height: min(calc(100vh - 100px), 800px);
					"
					scrollable
					max-content-height="none"
				>
					<div
						class="whitespace-pre-wrap break-words font-mono text-sm leading-relaxed text-secondary"
					>
						{{ licenseBody }}
					</div>
				</NewModal>
			</SettingsSection>
		</section>
	</SettingsSection>
</template>

<script setup lang="ts">
import {
	CheckIcon,
	ExpandIcon,
	FileTextIcon,
	InfoIcon,
	LockIcon,
	ScaleIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	Checkbox,
	Combobox,
	type ComboboxOption,
	defineMessages,
	Field,
	IconButton,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	Input,
	IntlFormatted,
	NewModal,
	RadioCard,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { defaultLicenseIds, isAdmin, TeamMemberPermission } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, shallowRef, useId, useTemplateRef } from 'vue'

import ValidationMessage from '@/components/ValidationMessage.vue'
import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import { useProjectIssueFieldAction } from '~/composables/project-issue-field-action'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import { normalizeProjectUrl } from '~/helpers/project-url'
import { projectCopyType } from '~/utils/project-messages'

import SettingsSection from '../settings-section.vue'
import {
	getLicenseSummary,
	type LicenseSummaryCategory,
	licenseSummaryRules,
} from './license-summaries'
const { projectV2: project, projectV3, currentMember, patchProjectV3 } = injectProjectPageContext()
const { labrinth } = injectModrinthClient()

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const licenseTextModal = useTemplateRef('licenseTextModal')
const licenseSelectId = useId()
const licenseNameId = useId()
const licenseUrlId = useId()
const licenseMeaningTitleId = useId()
const licenseTextTitleId = useId()
const messages = defineMessages({
	url: { id: 'project.settings.license.url', defaultMessage: 'License URL' },
	removeUrl: {
		id: 'project.settings.license.remove-url',
		defaultMessage: 'Remove license URL',
	},
	licenseHelp: {
		id: 'project.settings.license.creator-use-help',
		defaultMessage:
			'The {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}}’s license governs how other creators can use your {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}}.',
	},
	standardUrlHelp: {
		id: 'project.settings.license.standard-url-help',
		defaultMessage:
			'If your license text is not a copy of the {license}, you may want to select Custom License instead.',
	},
	customUrlHelp: {
		id: 'project.settings.license.custom-full-text-help',
		defaultMessage:
			'This should take users directly to the full text of your {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}}’s License.',
	},
	licenseType: {
		id: 'project.settings.license.type',
		defaultMessage: 'License type',
	},
	standardLicense: {
		id: 'project.settings.license.standard-type',
		defaultMessage: 'Standard license',
	},
	allRightsReserved: {
		id: 'project.settings.license.all-rights-reserved',
		defaultMessage: 'All Rights Reserved',
	},
	allRightsReservedDescription: {
		id: 'project.settings.license.all-rights-reserved-help',
		defaultMessage:
			'No license provided, you reserve all rights to this {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}} unless explicitly granted.',
	},
	customLicense: {
		id: 'project.settings.license.custom-type',
		defaultMessage: 'Custom license',
	},
	whatItMeans: {
		id: 'project.settings.license.what-it-means',
		defaultMessage: 'What it means',
	},
	noneGranted: {
		id: 'project.settings.license.none-granted',
		defaultMessage: 'None granted by this selection.',
	},
	noneListed: {
		id: 'project.settings.license.none-listed',
		defaultMessage: 'None listed.',
	},
	summaryAttribution: {
		id: 'project.settings.license.summary-attribution',
		defaultMessage:
			'This summary is provided by <source-link>Choose a License</source-link> and is not legal advice.',
	},
	licenseText: {
		id: 'project.settings.license.text',
		defaultMessage: 'License text',
	},
	expandLicenseText: {
		id: 'project.settings.license.expand-text',
		defaultMessage: 'Expand license text',
	},
	loadingLicenseText: {
		id: 'project.settings.license.text-loading',
		defaultMessage: 'Loading license text…',
	},
	licenseTextError: {
		id: 'project.settings.license.text-error',
		defaultMessage: 'License text could not be retrieved.',
	},
	licenseTextUnavailable: {
		id: 'project.settings.license.text-unavailable',
		defaultMessage: 'License text is not available for this license.',
	},
	permissions: {
		id: 'project.settings.license.summary-permissions',
		defaultMessage: 'Permissions',
	},
	limitations: {
		id: 'project.settings.license.summary-limitations',
		defaultMessage: 'Limitations',
	},
	conditions: {
		id: 'project.settings.license.summary-conditions',
		defaultMessage: 'Conditions',
	},
	standardLicenseDescription: {
		id: 'project.settings.license.standard-options-help',
		defaultMessage: 'Pick from a list of commonly used licenses.',
	},
	customLicenseDescription: {
		id: 'project.settings.license.custom-terms-help',
		defaultMessage:
			'Link to this {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}}’s unique licensing terms.',
	},
	custom: { id: 'project.settings.license.custom', defaultMessage: 'Custom' },
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
	linkOwnCopy: {
		id: 'project.settings.license.link-to-standard-text',
		defaultMessage:
			'Link to the full text of your license, instead of displaying the standard license text below.',
	},
	urlPlaceholder: {
		id: 'project.settings.license.example-url-placeholder',
		defaultMessage: 'https://example.com/license.txt',
	},
	name: { id: 'project.settings.license.name', defaultMessage: 'License name' },
	namePlaceholder: {
		id: 'project.settings.license.full-name-placeholder',
		defaultMessage: 'Full name of license',
	},
	invalidName: {
		id: 'project.settings.license.custom-name-invalid',
		defaultMessage: 'Custom license names can only contain letters, numbers, periods, and hyphens.',
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
	if (license.short === 'All-Rights-Reserved') return formatMessage(messages.allRightsReserved)
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
					? 'All Rights Reserved'
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
	reset: resetLicense,
	save: saveLicense,
} = useSavable(
	() => ({
		license: getInitialLicense(),
		licenseUrl: project.value.license.url ?? '',
		useStandardLicenseUrl: Boolean(project.value.license.url),
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

		if (licenseUrl.value !== (project.value.license.url ?? null)) {
			payload.license_url = licenseUrl.value
		}

		await patchProjectV3(payload, true, true)
	},
)

const licenseUrl = computed(() => {
	if (licenseType.value === 'none') return null
	if (licenseType.value === 'standard' && !current.value.useStandardLicenseUrl) return null
	return normalizeProjectUrl(current.value.licenseUrl) || null
})

function clearLicenseUrl() {
	current.value.licenseUrl = ''
	current.value.useStandardLicenseUrl = false
}

const licenseSelectionValidation = useProjectNagMessages('license')
const customLicenseValidation = useProjectNagMessages('custom-license')
const effectiveLicenseCheck = useProjectNagMessages('license-url', 'license')

const lastStandardLicense = shallowRef<{ license: LicenseOption; allowOrLater: boolean }>()
const selectedLicense = computed({
	get: () => (current.value.license.friendly === 'Custom' ? '' : current.value.license.short),
	set: (short: string) => {
		if (!short) return
		const license = availableLicenses.value.find((option) => option.short === short)
		if (!license) return
		current.value.license = license
	},
})

function reset() {
	lastStandardLicense.value = undefined
	resetLicense()
}

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
	invalidCustomName.value ? { severity: 'error' as const, message: messages.invalidName } : null,
)
const canSave = computed(
	() =>
		hasPermission.value &&
		!(licenseType.value === 'standard' && !selectedLicense.value) &&
		!(
			licenseType.value === 'standard' &&
			current.value.useStandardLicenseUrl &&
			!licenseUrl.value
		) &&
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
			url: licenseUrl.value ?? undefined,
		},
	}),
	canSave: () => canSave.value,
	saving,
	validation: saveValidation,
	save: async () => {
		const license_id = licenseId.value
		const submittedUrl = current.value.licenseUrl
		const license_url = licenseUrl.value
		if (
			license_id !== project.value.license.id ||
			(license_url ?? '') !== (project.value.license.url ?? '')
		) {
			await patchProjectV3({ license_id, license_url }, true, true)
		}
		if (current.value.licenseUrl === submittedUrl) {
			current.value.licenseUrl = license_url ?? ''
			current.value.useStandardLicenseUrl = Boolean(license_url)
		}
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
		label: messages.standardLicense,
		description: messages.standardLicenseDescription,
	},
	{
		value: 'none',
		icon: LockIcon,
		label: messages.allRightsReserved,
		description: messages.allRightsReservedDescription,
	},
	{
		value: 'custom',
		icon: FileTextIcon,
		label: messages.customLicense,
		description: messages.customLicenseDescription,
	},
]
const licenseType = computed({
	get: () => {
		const { friendly, short } = current.value.license
		if (friendly === 'Custom') return 'custom'
		if (short === 'Unknown' && !friendly) return ''
		if (short === 'All-Rights-Reserved') return 'none'
		return 'standard'
	},
	set: (type: string) => {
		if (type === licenseType.value) return

		if (licenseType.value === 'standard' && selectedLicense.value) {
			lastStandardLicense.value = {
				license: { ...current.value.license },
				allowOrLater: current.value.allowOrLater,
			}
		}

		if (type === 'standard') {
			const previous = lastStandardLicense.value
			current.value.license = previous ? { ...previous.license } : { short: '', friendly: '' }
			current.value.allowOrLater = previous?.allowOrLater ?? false
			return
		}

		const license = availableLicenses.value.find((option) =>
			type === 'custom' ? option.friendly === 'Custom' : option.short === 'All-Rights-Reserved',
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
		{
			key: 'permissions',
			title: formatMessage(messages.permissions),
			icon: CheckIcon,
			color: 'text-brand',
		},
		{
			key: 'limitations',
			title: formatMessage(messages.limitations),
			icon: XIcon,
			color: 'text-red',
		},
		{
			key: 'conditions',
			title: formatMessage(messages.conditions),
			icon: InfoIcon,
			color: 'text-blue',
		},
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

const showLicenseText = computed(
	() =>
		licenseType.value === 'standard' &&
		!(
			current.value.useStandardLicenseUrl &&
			saved.value.useStandardLicenseUrl &&
			licenseId.value === project.value.license.id
		) &&
		!!licenseId.value &&
		!licenseId.value.startsWith('LicenseRef-'),
)
const {
	data: licenseBody,
	isPending: loadingLicense,
	isError: licenseError,
} = useQuery({
	queryKey: computed(() => ['license', 'v2', licenseId.value] as const),
	queryFn: async () => (await labrinth.tags_v2.getLicenseText(licenseId.value)).body,
	enabled: showLicenseText,
	staleTime: 1000 * 60 * 10,
})

defineExpose({ saved, current, saving, hasChanges, canSave, reset, save })
</script>
