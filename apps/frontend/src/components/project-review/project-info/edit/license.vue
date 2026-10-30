<template>
	<EditModal
		ref="modal"
		section="License"
		:saving="saving"
		:can-save="canSave"
		@cancel="reset"
		@save="save"
	>
		<div class="flex min-w-0 flex-col gap-6">
			<div class="flex min-w-0 flex-col gap-2">
				<label class="font-semibold text-contrast"> Select a license </label>
				<Combobox
					v-model="selectedLicense"
					:options="licenseOptions"
					:search-options="allLicenseOptions"
					:display-value="licenseDisplayName(currentLicense)"
					placeholder="Select license..."
					search-placeholder="Select license..."
					:searchable="true"
					sync-with-selection
					select-search-text-on-focus
					:disabled="saving"
					trigger-type="base"
					class="w-full"
				/>
				<p class="m-0 text-sm text-secondary">
					How users are and aren't allowed to use your project.
				</p>
			</div>

			<div v-if="currentLicense.requiresOnlyOrLater" class="flex flex-col gap-2">
				<label class="font-semibold text-contrast"> Later editions </label>
				<Checkbox v-model="allowOrLater" :disabled="saving" description="Allow later editions">
					Allow later editions
				</Checkbox>
				<p class="m-0 text-sm text-secondary">
					The license you selected has an "or later" clause. If you check this box, users may use
					your project under later editions of the license.
				</p>
			</div>

			<div v-if="currentLicense.friendly" class="flex min-w-0 flex-col gap-2">
				<label for="project-review-license-url" class="font-semibold text-contrast">
					License URL
				</label>
				<Input
					id="project-review-license-url"
					v-model="licenseUrl"
					type="url"
					:maxlength="2048"
					:placeholder="
						currentLicense.friendly === 'Custom' ? 'License URL' : 'License URL (optional)'
					"
					:disabled="saving || licenseId === 'LicenseRef-Unknown'"
					wrapper-class="w-full"
				/>
				<p class="m-0 text-sm text-secondary">
					{{
						currentLicense.friendly === 'Custom'
							? 'The web location of the full license text. You have to provide a link since this is a custom license.'
							: "The web location of the full license text. If you don't provide a link, the license text will be displayed instead."
					}}
				</p>
			</div>

			<div v-if="currentLicense.friendly === 'Custom'" class="flex min-w-0 flex-col gap-2">
				<label for="project-review-license-name" class="font-semibold text-contrast">
					{{ hasSpdxLicense ? 'SPDX identifier' : 'License name' }}
				</label>
				<Input
					id="project-review-license-name"
					v-model="currentLicense.short"
					:maxlength="128"
					:placeholder="hasSpdxLicense ? 'SPDX identifier' : 'License name'"
					:disabled="saving"
					wrapper-class="w-full"
				/>
				<Checkbox v-model="hasSpdxLicense" :disabled="saving" description="Use SPDX identifier">
					Use SPDX identifier
				</Checkbox>
			</div>

			<p v-if="validationMessage" class="m-0 text-sm text-red" role="alert">
				{{ validationMessage }}
			</p>
		</div>
	</EditModal>
</template>

<script setup lang="ts">
import { Checkbox, Combobox, type ComboboxOption, injectModrinthClient, Input } from '@modrinth/ui'
import { defaultLicenseIds } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, reactive, ref, useTemplateRef } from 'vue'

import { normalizeProjectUrl } from '~/helpers/project-url'

import EditModal from './edit-modal.vue'
import { useProjectInfoEdit } from './use-project-edit'

interface LicenseChoice {
	friendly: string
	short: string
	requiresOnlyOrLater?: boolean
}

const { project, saving, beginEditing, saveProject } = useProjectInfoEdit()
const modal = useTemplateRef<InstanceType<typeof EditModal>>('modal')
const currentLicense = reactive<LicenseChoice>({ friendly: '', short: '' })
const licenseUrl = ref('')
const allowOrLater = ref(false)
const hasSpdxLicense = ref(true)

function licenseDisplayName(license: LicenseChoice) {
	if (license.friendly === 'Custom') return 'Custom'
	if (license.short === 'All-Rights-Reserved') return 'All Rights Reserved/No License'
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

const allLicenseOptions = computed<ComboboxOption<string>[]>(() =>
	availableLicenses.value.map((license) => ({
		value: license.short,
		label: licenseDisplayName(license),
		searchTerms: [license.short, license.short.replaceAll('-', ' ')],
	})),
)
const licenseOptions = computed(() => {
	const defaultIds = new Set<string>(defaultLicenseIds)
	return allLicenseOptions.value.filter(
		(license) => license.value === '' || defaultIds.has(license.value),
	)
})

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
		return 'Enter a name or SPDX identifier for your custom license.'
	}
	if (currentLicense.friendly === 'Custom' && !normalizedLicenseUrl.value) {
		return 'Enter a URL to the full text of your custom license.'
	}
	if (!urlValid.value) return 'Enter a valid HTTP or HTTPS license URL.'
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
