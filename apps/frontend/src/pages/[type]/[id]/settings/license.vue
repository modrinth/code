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
						v-model="license"
						name="License selector"
						:options="builtinLicenses"
						:display-name="(chosen: BuiltinLicense) => chosen.friendly"
						placeholder="Select license..."
					/>
				</div>
			</div>

			<div v-if="license.requiresOnlyOrLater" class="adjacent-input">
				<label for="or-later-checkbox">
					<span class="label__title">Later editions</span>
					<span class="label__description">
						The license you selected has an "or later" clause. If you check this box, users may use
						your project under later editions of the license.
					</span>
				</label>

				<Checkbox
					id="or-later-checkbox"
					v-model="allowOrLater"
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
					<span v-if="license?.friendly !== 'Custom'" class="label__description">
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
						v-model="licenseUrl"
						type="url"
						maxlength="2048"
						:placeholder="license?.friendly !== 'Custom' ? `License URL (optional)` : `License URL`"
						:disabled="!hasPermission || licenseId === 'LicenseRef-Unknown'"
						class="w-full"
					/>
				</div>
			</div>

			<div v-if="license?.friendly === 'Custom'" class="adjacent-input">
				<label v-if="!nonSpdxLicense" for="license-spdx">
					<span class="label__title">SPDX identifier</span>
					<span class="label__description">
						If your license does not have an offical
						<a href="https://spdx.org/licenses/" target="_blank" rel="noopener" class="text-link">
							SPDX license identifier</a
						>, check the box and enter the name of the license instead.
					</span>
				</label>
				<label v-else for="license-name">
					<span class="label__title">License name</span>
					<span class="label__description"
						>The full name of the license. If the license has a SPDX identifier, please uncheck the
						checkbox and use the identifier instead.</span
					>
				</label>

				<div class="input-stack w-1/2">
					<input
						v-if="!nonSpdxLicense"
						id="license-spdx"
						v-model="license.short"
						class="w-full"
						type="text"
						maxlength="128"
						placeholder="SPDX identifier"
						:disabled="!hasPermission"
					/>
					<input
						v-else
						id="license-name"
						v-model="license.short"
						class="w-full"
						type="text"
						maxlength="128"
						placeholder="License name"
						:disabled="!hasPermission"
					/>

					<Checkbox
						v-if="license?.friendly === 'Custom'"
						v-model="nonSpdxLicense"
						:disabled="!hasPermission"
						description="License does not have a SPDX identifier"
					>
						License does not have a SPDX identifier
					</Checkbox>
				</div>
			</div>

			<div class="input-stack">
				<button
					type="button"
					class="iconified-button brand-button"
					:disabled="
						!hasChanges ||
						!hasPermission ||
						(license.friendly === 'Custom' && (license.short === '' || licenseUrl === ''))
					"
					@click="saveChanges()"
				>
					<SaveIcon />
					Save changes
				</button>
			</div>
		</section>
	</div>
</template>

<script setup lang="ts">
import { SaveIcon } from '@modrinth/assets'
import { Checkbox, DropdownSelect } from '@modrinth/ui'
import {
	type BuiltinLicense,
	builtinLicenses,
	formatProjectType,
	type Project,
	type TeamMember,
	TeamMemberPermission,
} from '@modrinth/utils'
import { computed, type Ref, ref } from 'vue'

const props = defineProps<{
	project: Project
	currentMember: TeamMember | undefined
	patchProject: (payload: object, quiet?: boolean) => object
}>()

const licenseUrl = ref(props.project.license.url)
const license: Ref<{
	friendly: string
	short: string
	requiresOnlyOrLater?: boolean
}> = ref({
	friendly: '',
	short: '',
	requiresOnlyOrLater: false,
})

const allowOrLater = ref(props.project.license.id.includes('-or-later'))
const nonSpdxLicense = ref(props.project.license.id.includes('LicenseRef-'))

const oldLicenseId = props.project.license.id
const trimmedLicenseId = oldLicenseId
	.replaceAll('-only', '')
	.replaceAll('-or-later', '')
	.replaceAll('LicenseRef-', '')

license.value = builtinLicenses.find((x) => x.short === trimmedLicenseId) ?? {
	friendly: 'Custom',
	short: oldLicenseId.replaceAll('LicenseRef-', ''),
	requiresOnlyOrLater: oldLicenseId.includes('-or-later'),
}

if (oldLicenseId === 'LicenseRef-Unknown') {
	// Mark it as not having a license, forcing the user to select one
	license.value = {
		friendly: '',
		short: oldLicenseId.replaceAll('LicenseRef-', ''),
		requiresOnlyOrLater: false,
	}
}

const hasPermission = computed(() => {
	return (props.currentMember?.permissions ?? 0) & TeamMemberPermission.EDIT_DETAILS
})

const licenseId = computed(() => {
	let id = ''

	if (
		(nonSpdxLicense.value && license.value.friendly === 'Custom') ||
		license.value.short === 'All-Rights-Reserved' ||
		license.value.short === 'Unknown'
	) {
		id += 'LicenseRef-'
	}

	id += license.value.short
	if (license.value.requiresOnlyOrLater) {
		id += allowOrLater.value ? '-or-later' : '-only'
	}

	if (nonSpdxLicense.value && license.value.friendly === 'Custom') {
		id = id.replaceAll(' ', '-')
	}

	return id
})

const patchRequestPayload = computed(() => {
	const payload: {
		license_id?: string
		license_url?: string | null // null = remove url
	} = {}

	if (licenseId.value !== props.project.license.id) {
		payload.license_id = licenseId.value
	}

	if (licenseUrl.value !== props.project.license.url) {
		payload.license_url = licenseUrl.value ? licenseUrl.value : null
	}

	return payload
})

const hasChanges = computed(() => {
	return Object.keys(patchRequestPayload.value).length > 0
})

function saveChanges() {
	props.patchProject(patchRequestPayload.value)
}
</script>
