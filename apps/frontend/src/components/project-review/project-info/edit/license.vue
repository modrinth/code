<template>
	<EditModal
		ref="modal"
		:section="formatMessage(reviewMessages.license)"
		:saving="saving"
		:can-save="canSave"
		@cancel="reset"
		@save="save"
	>
		<div class="flex min-w-0 flex-col gap-6">
			<div class="flex min-w-0 flex-col gap-2">
				<label class="font-semibold text-contrast">
					{{ formatMessage(messages.select) }}
				</label>
				<Combobox
					v-model="selectedLicense"
					:options="licenseOptions"
					:display-value="licenseDisplayName(currentLicense)"
					:placeholder="formatMessage(messages.selectPlaceholder)"
					:disabled="saving"
					trigger-type="base"
					class="w-full"
				/>
				<p class="m-0 text-sm text-secondary">
					{{ formatMessage(messages.selectDescription) }}
				</p>
			</div>

			<div v-if="currentLicense.requiresOnlyOrLater" class="flex flex-col gap-2">
				<label class="font-semibold text-contrast">
					{{ formatMessage(messages.later) }}
				</label>
				<Checkbox
					v-model="allowOrLater"
					:disabled="saving"
					:description="formatMessage(messages.allowLater)"
				>
					{{ formatMessage(messages.allowLater) }}
				</Checkbox>
				<p class="m-0 text-sm text-secondary">
					{{ formatMessage(messages.laterDescription) }}
				</p>
			</div>

			<div v-if="currentLicense.friendly" class="flex min-w-0 flex-col gap-2">
				<label for="project-review-license-url" class="font-semibold text-contrast">
					{{ formatMessage(licenseUrlMessages.url) }}
				</label>
				<Input
					id="project-review-license-url"
					v-model="licenseUrl"
					type="url"
					:maxlength="2048"
					:placeholder="
						formatMessage(
							currentLicense.friendly === 'Custom'
								? licenseUrlMessages.url
								: licenseUrlMessages.optionalUrl,
						)
					"
					:disabled="saving || licenseId === 'LicenseRef-Unknown'"
					wrapper-class="w-full"
				/>
				<p class="m-0 text-sm text-secondary">
					{{
						formatMessage(
							currentLicense.friendly === 'Custom'
								? licenseUrlMessages.customUrlDescription
								: licenseUrlMessages.urlDescription,
						)
					}}
				</p>
			</div>

			<div v-if="currentLicense.friendly === 'Custom'" class="flex min-w-0 flex-col gap-2">
				<label for="project-review-license-name" class="font-semibold text-contrast">
					{{ formatMessage(hasSpdxLicense ? messages.spdx : messages.name) }}
				</label>
				<Input
					id="project-review-license-name"
					v-model="currentLicense.short"
					:maxlength="128"
					:placeholder="formatMessage(hasSpdxLicense ? messages.spdx : messages.name)"
					:disabled="saving"
					wrapper-class="w-full"
				/>
				<Checkbox
					v-model="hasSpdxLicense"
					:disabled="saving"
					:description="formatMessage(messages.hasSpdx)"
				>
					{{ formatMessage(messages.hasSpdx) }}
				</Checkbox>
			</div>

			<p v-if="validationMessage" class="m-0 text-sm text-red" role="alert">
				{{ validationMessage }}
			</p>
		</div>
	</EditModal>
</template>

<script setup lang="ts">
import {
	Checkbox,
	Combobox,
	type ComboboxOption,
	defineMessages,
	injectModrinthClient,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { defaultLicenseIds } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, reactive, ref, useTemplateRef } from 'vue'

import { normalizeProjectUrl } from '~/helpers/project-url'
import { licenseUrlMessages } from '~/utils/license-messages'

import { projectReviewMessages as reviewMessages } from '../../messages'
import EditModal from './edit-modal.vue'
import { useProjectInfoEdit } from './use-project-edit'

interface LicenseChoice {
	friendly: string
	short: string
	requiresOnlyOrLater?: boolean
}

const messages = defineMessages({
	custom: { id: 'project.settings.license.custom', defaultMessage: 'Custom' },
	allRights: {
		id: 'project.settings.license.all-rights',
		defaultMessage: 'All Rights Reserved/No License',
	},
	select: {
		id: 'project.settings.license.select',
		defaultMessage: 'Select a license',
	},
	selectDescription: {
		id: 'project.settings.license.select-description',
		defaultMessage: "How users are and aren't allowed to use your project.",
	},
	selectPlaceholder: {
		id: 'project.settings.license.select-placeholder',
		defaultMessage: 'Select license...',
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
	spdx: {
		id: 'project.settings.license.spdx',
		defaultMessage: 'SPDX identifier',
	},
	name: { id: 'project.settings.license.name', defaultMessage: 'License name' },
	hasSpdx: {
		id: 'project.settings.license.has-spdx',
		defaultMessage: 'Use SPDX identifier',
	},
	missingName: {
		id: 'project.settings.license.missing-name',
		defaultMessage: 'Enter a name or SPDX identifier for your custom license.',
	},
	missingUrl: {
		id: 'project.settings.license.missing-url',
		defaultMessage: 'Enter a URL to the full text of your custom license.',
	},
	invalidUrl: {
		id: 'project.settings.license.invalid-url',
		defaultMessage: 'Enter a valid HTTP or HTTPS license URL.',
	},
})

const { formatMessage } = useVIntl()
const { project, saving, beginEditing, saveProject } = useProjectInfoEdit()
const modal = useTemplateRef<InstanceType<typeof EditModal>>('modal')
const currentLicense = reactive<LicenseChoice>({ friendly: '', short: '' })
const licenseUrl = ref('')
const allowOrLater = ref(false)
const hasSpdxLicense = ref(true)

function licenseDisplayName(license: LicenseChoice) {
	if (license.friendly === 'Custom') return formatMessage(messages.custom)
	if (license.short === 'All-Rights-Reserved') return formatMessage(messages.allRights)
	return license.friendly
}

const orLaterLicenseIds = new Set(['AGPL-3.0', 'LGPL-2.1', 'LGPL-3.0', 'GPL-2.0', 'GPL-3.0'])
const { labrinth } = injectModrinthClient()
const { data: spdxLicenses } = useQuery({
	queryKey: ['tags', 'licenses', 'v2'],
	queryFn: () => labrinth.tags_v2.getLicenses(),
	staleTime: 1000 * 60 * 60,
})
const availableLicenses = computed<LicenseChoice[]>(() => {
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

const licenseOptions = computed<ComboboxOption<string>[]>(() =>
	availableLicenses.value.map((license) => ({
		value: license.short,
		label: licenseDisplayName(license),
	})),
)

function initialLicense(): LicenseChoice {
	const oldLicenseId = project.value?.license.id ?? 'LicenseRef-Unknown'
	const trimmedLicenseId = oldLicenseId
		.replaceAll('-only', '')
		.replaceAll('-or-later', '')
		.replaceAll('LicenseRef-', '')

	if (oldLicenseId === 'LicenseRef-Unknown') {
		return { friendly: '', short: 'Unknown', requiresOnlyOrLater: false }
	}

	return (
		availableLicenses.value.find((license) => license.short === trimmedLicenseId) ?? {
			friendly: 'Custom',
			short: trimmedLicenseId,
			requiresOnlyOrLater: oldLicenseId.includes('-or-later'),
		}
	)
}

function reset() {
	const license = initialLicense()
	Object.assign(currentLicense, license, {
		requiresOnlyOrLater: license.requiresOnlyOrLater ?? false,
	})
	licenseUrl.value = project.value?.license.url ?? ''
	allowOrLater.value = project.value?.license.id.includes('-or-later') ?? false
	hasSpdxLicense.value = !project.value?.license.id.includes('LicenseRef-')
}

function show() {
	beginEditing()
	reset()
	modal.value?.show()
}

const selectedLicense = computed({
	get: () => (currentLicense.friendly === 'Custom' ? '' : currentLicense.short),
	set: (short: string) => {
		const license = availableLicenses.value.find((option) => option.short === short)
		if (!license) return
		Object.assign(currentLicense, license, {
			requiresOnlyOrLater: license.requiresOnlyOrLater ?? false,
		})
	},
})

const licenseId = computed(() => {
	let id = ''
	if (
		(!hasSpdxLicense.value && currentLicense.friendly === 'Custom') ||
		currentLicense.short === 'All-Rights-Reserved' ||
		currentLicense.short === 'Unknown'
	) {
		id += 'LicenseRef-'
	}
	id += currentLicense.short
	if (currentLicense.requiresOnlyOrLater) {
		id += allowOrLater.value ? '-or-later' : '-only'
	}
	if (!hasSpdxLicense.value && currentLicense.friendly === 'Custom') id = id.replaceAll(' ', '-')
	return id
})
const normalizedLicenseUrl = computed(() => normalizeProjectUrl(licenseUrl.value))
const urlValid = computed(() => {
	if (!normalizedLicenseUrl.value) return true
	try {
		return ['http:', 'https:'].includes(new URL(normalizedLicenseUrl.value).protocol)
	} catch {
		return false
	}
})
const validationMessage = computed(() => {
	if (currentLicense.friendly === 'Custom' && !currentLicense.short.trim()) {
		return formatMessage(messages.missingName)
	}
	if (currentLicense.friendly === 'Custom' && !normalizedLicenseUrl.value) {
		return formatMessage(messages.missingUrl)
	}
	if (!urlValid.value) return formatMessage(messages.invalidUrl)
	return ''
})
const hasChanges = computed(
	() =>
		licenseId.value !== project.value?.license.id ||
		normalizedLicenseUrl.value !== (project.value?.license.url ?? ''),
)
const canSave = computed(() => hasChanges.value && !validationMessage.value)

async function save() {
	if (!canSave.value) return
	if (
		await saveProject({
			license_id: licenseId.value,
			license_url: normalizedLicenseUrl.value || null,
		})
	) {
		modal.value?.hide()
	}
}

defineExpose({ show })
</script>
