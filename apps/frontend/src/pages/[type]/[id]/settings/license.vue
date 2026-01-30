<template>
	<div>
		<section class="universal-card">
			<h2 class="label__title size-card-header">License</h2>
			<p class="label__description">
				It is important to choose a proper license for your
				{{ formatProjectType(project.project_type).toLowerCase() }}. You may choose one from our
				list or provide a custom license. You may also provide a custom URL to your chosen license;
				otherwise, the license text will be displayed. See our
				<nuxt-link
					to="/news/article/licensing-guide/"
					target="_blank"
					rel="noopener"
					class="text-link"
				>
					licensing guide
				</nuxt-link>
				for more information.
			</p>

			<div class="adjacent-input">
				<label for="license-multiselect">
					<span class="label__title">Select a license</span>
					<span class="label__description">
						How users are and aren't allowed to use your project.
					</span>
				</label>

				<div class="w-1/2">
					<DropdownSelect
						v-model="current.license"
						name="License selector"
						:options="builtinLicenses"
						:display-name="(chosen: BuiltinLicense) => chosen.friendly"
						placeholder="Select license..."
					/>
				</div>
			</div>

			<div v-if="current.license.requiresOnlyOrLater" class="adjacent-input">
				<label for="or-later-checkbox">
					<span class="label__title">Later editions</span>
					<span class="label__description">
						The license you selected has an "or later" clause. If you check this box, users may use
						your project under later editions of the license.
					</span>
				</label>

				<Checkbox
					id="or-later-checkbox"
					v-model="current.allowOrLater"
					:disabled="!hasPermission"
					description="Allow later editions"
					class="w-1/2"
				>
					Allow later editions
				</Checkbox>
			</div>

			<div class="adjacent-input">
				<label for="license-url">
					<span class="label__title">License URL</span>
					<span v-if="current.license?.friendly !== 'Custom'" class="label__description">
						The web location of the full license text. If you don't provide a link, the license text
						will be displayed instead.
					</span>
					<span v-else class="label__description">
						The web location of the full license text. You have to provide a link since this is a
						custom license.
					</span>
				</label>

				<div class="w-1/2">
					<input
						id="license-url"
						v-model="current.licenseUrl"
						type="url"
						maxlength="2048"
						:placeholder="
							current.license?.friendly !== 'Custom' ? `License URL (optional)` : `License URL`
						"
						:disabled="!hasPermission || licenseId === 'LicenseRef-Unknown'"
						class="w-full"
					/>
				</div>
			</div>

			<div v-if="current.license?.friendly === 'Custom'" class="adjacent-input">
				<label v-if="!current.nonSpdxLicense" for="license-spdx">
					<span class="label__title">SPDX identifier</span>
					<span class="label__description">
						If your license does not have an offical
						<a href="https://spdx.org/licenses/" target="_blank" rel="noopener" class="text-link">
							SPDX license identifier</a
						>, check the box and enter the name of the license instead.
					</span>
				</label>
				<label v-if="current.nonSpdxLicense" for="license-name">
					<span class="label__title">License name</span>
					<span class="label__description"
						>The full name of the license. If the license has a SPDX identifier, please uncheck the
						checkbox and use the identifier instead.</span
					>
				</label>

				<div class="input-stack w-1/2">
					<input
						v-if="!current.nonSpdxLicense"
						id="license-spdx"
						v-model="current.license.short"
						class="w-full"
						type="text"
						maxlength="128"
						placeholder="SPDX identifier"
						:disabled="!hasPermission"
					/>
					<input
						v-else
						id="license-name"
						v-model="current.license.short"
						class="w-full"
						type="text"
						maxlength="128"
						placeholder="License name"
						:disabled="!hasPermission"
					/>

					<Checkbox
						v-if="current.license?.friendly === 'Custom'"
						v-model="current.nonSpdxLicense"
						:disabled="!hasPermission"
						description="License does not have a SPDX identifier"
					>
						License does not have a SPDX identifier
					</Checkbox>
				</div>
			</div>
		</section>
		<UnsavedChangesPopup
			:original="saved"
			:modified="current"
			:saving="saving"
			:can-save="
				hasPermission &&
				!(
					current.license.friendly === 'Custom' &&
					(current.license.short === '' || current.licenseUrl === '')
				)
			"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>

<script setup lang="ts">
import {
	Checkbox,
	DropdownSelect,
	injectProjectPageContext,
	UnsavedChangesPopup,
	useSavable,
} from '@modrinth/ui'
import {
	type BuiltinLicense,
	builtinLicenses,
	formatProjectType,
	TeamMemberPermission,
} from '@modrinth/utils'
import { computed } from 'vue'

const { projectV2: project, currentMember, patchProject } = injectProjectPageContext()

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

	return (
		builtinLicenses.find((x) => x.short === trimmedLicenseId) ?? {
			friendly: 'Custom',
			short: oldLicenseId.replaceAll('LicenseRef-', ''),
			requiresOnlyOrLater: oldLicenseId.includes('-or-later'),
		}
	)
}

const { saved, current, saving, reset, save } = useSavable(
	() => ({
		license: getInitialLicense(),
		licenseUrl: project.value.license.url ?? '',
		allowOrLater: project.value.license.id.includes('-or-later'),
		nonSpdxLicense: project.value.license.id.includes('LicenseRef-'),
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
			payload.license_url = current.value.licenseUrl ? current.value.licenseUrl : null
		}

		await patchProject(payload)
	},
)

const hasPermission = computed(() => {
	return (currentMember.value?.permissions ?? 0) & TeamMemberPermission.EDIT_DETAILS
})

const licenseId = computed(() => {
	let id = ''

	if (
		(current.value.nonSpdxLicense && current.value.license.friendly === 'Custom') ||
		current.value.license.short === 'All-Rights-Reserved' ||
		current.value.license.short === 'Unknown'
	) {
		id += 'LicenseRef-'
	}

	id += current.value.license.short
	if (current.value.license.requiresOnlyOrLater) {
		id += current.value.allowOrLater ? '-or-later' : '-only'
	}

	if (current.value.nonSpdxLicense && current.value.license.friendly === 'Custom') {
		id = id.replaceAll(' ', '-')
	}

	return id
})
</script>
