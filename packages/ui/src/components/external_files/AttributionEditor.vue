<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, SaveIcon, SpinnerIcon, TrashIcon, UploadIcon, XIcon } from '@modrinth/assets'
import { builtinLicenses } from '@modrinth/utils'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import {
	Admonition,
	ButtonStyled,
	Chips,
	Combobox,
	type ComboboxOption,
	StyledInput,
} from '#ui/components'
import { FileInput } from '#ui/components/base'
import { commonMessages } from '#ui/utils'

import { defineMessage, defineMessages, useVIntl } from '../../composables/i18n'
import { injectModrinthClient } from '../../providers'
import {
	attributionLinkToWork,
	CUSTOM_LICENSE_VALUE,
	isHttpUrl,
	parseAttributionLicense,
	parseInitialAttribution,
	PERMISSION_REASONS,
	permissionKinds,
	type ProjectPermissionField,
} from './external-project-utils'

const props = defineProps<{
	projectId: string
	groupId: string
	attribution?: Labrinth.Attribution.Internal.AttributionResolution | null | undefined
	flameProjectUrl?: string | null
	/** Increments when the parent resumes editing so local fields reset */
	resumeKey: number
}>()

const emit = defineEmits<{
	(e: 'saved' | 'updated' | 'cancel'): void
}>()

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const queryClient = useQueryClient()

const initialAttribution = computed<Labrinth.Attribution.Internal.AttributionResolution | null>(
	() => parseInitialAttribution(props.attribution),
)

const isAttributed = computed(() => initialAttribution.value !== null)

const messages = defineMessages({
	typeLabel: {
		id: 'external-files.permissions-card.type-label',
		defaultMessage: 'Permission reason',
	},
	licenseLabel: {
		id: 'external-files.permissions-card.license-label',
		defaultMessage: 'License',
	},
	selectLicenseLabel: {
		id: 'external-files.permissions-card.select-license-label',
		defaultMessage: 'Select a license...',
	},
	linkLabel: {
		id: 'external-files.permissions-card.link-label',
		defaultMessage: 'Link to work',
	},
	linkToWorkUrlPlaceholder: {
		id: 'external-files.permissions-card.link-to-work-url-placeholder',
		defaultMessage: 'link-to-work',
	},
	notesLabel: {
		id: 'external-files.permissions-card.notes-label',
		defaultMessage: 'Notes',
	},
	optional: {
		id: 'external-files.permissions-card.input-optional',
		defaultMessage: '(optional)',
	},
	notesPlaceholder: {
		id: 'external-files.permissions-card.notes-placeholder',
		defaultMessage: 'Write something here...',
	},
	proofWarningTitle: {
		id: 'external-files.permissions-card.proof-warning.title',
		defaultMessage: 'Modrinth staff may verify submitted proof',
	},
	proofWarningBody: {
		id: 'external-files.permissions-card.proof-warning.body',
		defaultMessage:
			'If you are found to have lied or manipulated the images uploaded, your project and account may be terminated.',
	},
	saveAttribution: {
		id: 'external-files.permissions-card.save',
		defaultMessage: 'Save attribution',
	},
	addAttribution: {
		id: 'external-files.permissions-card.add',
		defaultMessage: 'Add attribution',
	},
	licenseRequired: {
		id: 'external-files.permissions-card.error.license-required',
		defaultMessage: 'Please select a license.',
	},
	customLicenseLabel: {
		id: 'external-files.permissions-card.custom-license-label',
		defaultMessage: 'Link to license',
	},
	linkToLicenseUrlPlaceholder: {
		id: 'external-files.permissions-card.link-to-license-url-placeholder',
		defaultMessage: 'link-to-license',
	},
	linkInvalidUrl: {
		id: 'external-files.permissions-card.error.link-invalid-url',
		defaultMessage: 'Link must be a valid URL.',
	},
	proofImagesLabel: {
		id: 'external-files.permissions-card.proof-images-label',
		defaultMessage: 'Proof images',
	},
	proofImagesUploadPrompt: {
		id: 'external-files.permissions-card.proof-images-upload-prompt',
		defaultMessage: 'Drag and drop to upload or click to select an image',
	},
	proofImageThumbnailAlt: {
		id: 'external-files.permissions-card.proof-image-alt',
		defaultMessage: 'Proof screenshot {n}',
	},
	proofImageRemove: {
		id: 'external-files.permissions-card.proof-image-remove',
		defaultMessage: 'Remove image',
	},
})

const MAX_PROOF_IMAGE_BYTES = 1_048_576

const selectedKind = ref<Labrinth.Attribution.Internal.AttributionResolutionKind>(
	initialAttribution.value?.kind ?? 'license',
)

const licenseIdInput = ref('')
const customLicenseInput = ref('')
const linkInput = ref('')
const notesInput = ref('')
const inputError = ref<string | null>(null)
const proofImageUrls = ref<string[]>([])

function extFromImageFile(file: File): Labrinth.Images.v3.ImageExtension | null {
	const byMime: Partial<Record<string, Labrinth.Images.v3.ImageExtension>> = {
		'image/png': 'png',
		'image/gif': 'gif',
		'image/webp': 'webp',
		'image/bmp': 'bmp',
		'image/jpeg': 'jpg',
	}
	const mime = byMime[file.type]
	if (mime) {
		return mime
	}
	const ext = file.name.toLowerCase().split('.').pop()
	if (ext === 'jpg' || ext === 'jpeg') {
		return 'jpg'
	}
	if (ext === 'png' || ext === 'gif' || ext === 'webp' || ext === 'bmp') {
		return ext
	}
	return null
}

function resetInputs() {
	const payload = initialAttribution.value
	selectedKind.value = payload?.kind ?? 'license'
	const license =
		payload && (payload.kind === 'license' || payload.kind === 'my_project')
			? parseAttributionLicense(payload.license)
			: { spdx: '', custom: '' }
	licenseIdInput.value = license.spdx
	customLicenseInput.value = license.custom
	const linkFallback = props.flameProjectUrl ?? ''
	linkInput.value = attributionLinkToWork(payload) ?? linkFallback
	notesInput.value = payload?.notes ?? ''
	proofImageUrls.value = payload?.image_urls ?? []
	inputError.value = null
}

resetInputs()

watch(licenseIdInput, (value) => {
	if (value !== CUSTOM_LICENSE_VALUE) {
		customLicenseInput.value = ''
	}
})

watch(
	() => props.attribution,
	() => {
		resetInputs()
	},
	{ deep: true },
)

watch(
	() => props.resumeKey,
	() => {
		resetInputs()
	},
)

const permissionReasonFields = computed<ProjectPermissionField[]>(() => {
	return PERMISSION_REASONS[selectedKind.value]?.fields ?? []
})

const isCustomLicense = computed(() => licenseIdInput.value === CUSTOM_LICENSE_VALUE)

const licenseOptions = computed<ComboboxOption<string>[]>(() => [
	...builtinLicenses
		.filter((license) => license.short !== '' && license.short !== 'All-Rights-Reserved')
		.map((license) => ({
			value: license.short,
			label: license.short,
		})),
	{
		value: CUSTOM_LICENSE_VALUE,
		label: formatMessage(
			defineMessage({
				id: 'external-files.permissions-card.custom-license-option',
				defaultMessage: 'Other',
			}),
		),
	},
])

function buildAttributionLicense(): Labrinth.Attribution.Internal.AttributionLicense | null {
	const custom = isCustomLicense.value
	if (!licenseIdInput.value) {
		inputError.value = formatMessage(messages.licenseRequired)
		return null
	}
	const customLicense = customLicenseInput.value.trim()
	if (custom && !customLicense) {
		inputError.value = formatMessage(
			defineMessage({
				id: 'external-files.permissions-card.error.custom-license-required',
				defaultMessage: 'Please describe the custom license.',
			}),
		)
		return null
	}
	return custom ? { name: customLicense } : licenseIdInput.value
}

function buildEditedData(): Labrinth.Attribution.Internal.AttributionResolution | null {
	inputError.value = null
	const notes = notesInput.value.trim()
	const image_urls = [...proofImageUrls.value]
	switch (selectedKind.value) {
		case 'license': {
			const license = buildAttributionLicense()
			if (!license) {
				return null
			}
			const linkRaw = linkInput.value.trim()
			if (!linkRaw) {
				inputError.value = formatMessage(
					defineMessage({
						id: 'external-files.permissions-card.error.link-required',
						defaultMessage: 'Please provide a link.',
					}),
				)
				return null
			}
			if (!isHttpUrl(linkRaw)) {
				inputError.value = formatMessage(messages.linkInvalidUrl)
				return null
			}
			return {
				kind: 'license',
				license,
				link_to_work: linkRaw,
				notes,
				image_urls,
			}
		}
		case 'my_project': {
			const license = buildAttributionLicense()
			if (!license) {
				return null
			}
			return {
				kind: 'my_project',
				license,
				notes,
				image_urls,
			}
		}
		case 'special_permissions': {
			const linkRaw = linkInput.value.trim()
			if (!linkRaw) {
				inputError.value = formatMessage(
					defineMessage({
						id: 'external-files.permissions-card.error.link-required',
						defaultMessage: 'Please provide a link.',
					}),
				)
				return null
			}
			if (!isHttpUrl(linkRaw)) {
				inputError.value = formatMessage(messages.linkInvalidUrl)
				return null
			}
			return {
				kind: 'special_permissions',
				link_to_work: linkRaw,
				notes,
				image_urls,
			}
		}
		case 'no_permission':
			return {
				kind: 'no_permission',
				notes,
				image_urls,
			}
	}
	return null
}

const uploadProofImageMutation = useMutation({
	mutationFn: async (file: File) => {
		const ext = extFromImageFile(file)
		if (!ext) {
			throw new Error(
				formatMessage(
					defineMessage({
						id: 'external-files.permissions-card.error.proof-image-invalid-type',
						defaultMessage: 'Please upload a PNG, JPEG, GIF, WebP, or BMP image.',
					}),
				),
			)
		}
		const result = await client.labrinth.images_v3.uploadImage(file, ext, {
			context: 'project',
			project_id: props.projectId,
		}).promise
		return result.url
	},
	onSuccess(url) {
		proofImageUrls.value = [...proofImageUrls.value, url]
	},
})

function handleProofImagesSelected(files: File[]) {
	const file = files[0]
	if (!file) {
		return
	}
	inputError.value = null
	uploadProofImageMutation.mutate(file)
}

function removeProofImage(index: number) {
	proofImageUrls.value = proofImageUrls.value.filter((_, i) => i !== index)
}

const saveMutation = useMutation({
	mutationFn: (payload: Labrinth.Attribution.Internal.AttributionResolution) =>
		client.labrinth.attribution_internal.updateGroup(props.groupId, {
			attribution: payload,
		}),
	onSuccess: async () => {
		await queryClient.invalidateQueries({ queryKey: ['project-attribution', props.projectId] })
		emit('updated')
		emit('saved')
	},
})

function handleSave() {
	const data = buildEditedData()
	if (!data) {
		return
	}
	saveMutation.mutate(data)
}

function cancelEditing() {
	resetInputs()
	if (isAttributed.value) {
		emit('cancel')
	}
}
</script>

<template>
	<div class="flex flex-col gap-3">
		<span class="text-contrast font-semibold">
			{{ formatMessage(messages.typeLabel) }}
		</span>
		<Chips
			v-model="selectedKind"
			:items="permissionKinds.filter((kind) => kind !== 'globally_allowed')"
			:format-label="(kind) => formatMessage(PERMISSION_REASONS[kind].label)"
			:capitalize="false"
		/>
		<span>{{ formatMessage(PERMISSION_REASONS[selectedKind].description) }}</span>
		<div v-if="permissionReasonFields.includes('link_to_work')" class="flex flex-col gap-2">
			<span class="text-contrast font-semibold mt-1">
				{{ formatMessage(messages.linkLabel) }}
			</span>
			<StyledInput
				v-model="linkInput"
				type="text"
				class="max-w-[40rem]"
				:placeholder="`https://example.com/${formatMessage(messages.linkToWorkUrlPlaceholder)}`"
			/>
		</div>
		<div v-if="permissionReasonFields.includes('license_id')" class="flex flex-col gap-2">
			<span class="text-contrast font-semibold mt-1">
				{{ formatMessage(messages.licenseLabel) }}
			</span>
			<Combobox
				v-model="licenseIdInput"
				class="max-w-80"
				:options="licenseOptions"
				searchable
				:search-placeholder="formatMessage(messages.selectLicenseLabel)"
			/>
		</div>
		<div
			v-if="permissionReasonFields.includes('custom_license') && isCustomLicense"
			class="flex flex-col gap-2"
		>
			<span class="text-contrast font-semibold mt-1">
				{{ formatMessage(messages.customLicenseLabel) }}
			</span>
			<StyledInput
				v-model="customLicenseInput"
				type="text"
				class="max-w-[40rem]"
				:placeholder="`https://example.com/${formatMessage(messages.linkToLicenseUrlPlaceholder)}`"
			/>
		</div>
		<div v-if="permissionReasonFields.includes('notes')" class="flex flex-col gap-2">
			<span class="text-contrast font-semibold mt-1">
				{{ formatMessage(messages.notesLabel) }}
				<span class="font-normal text-primary">{{ formatMessage(messages.optional) }}</span>
			</span>
			<StyledInput
				v-model="notesInput"
				type="text"
				resize="both"
				multiline
				class="max-w-[40rem]"
				:placeholder="formatMessage(messages.notesPlaceholder)"
			/>
		</div>
		<div v-if="permissionReasonFields.includes('image_urls')" class="flex flex-col gap-2">
			<div class="flex flex-col gap-2 mt-1">
				<div class="flex flex-col gap-1 mt-1">
					<span class="text-contrast font-semibold">
						{{ formatMessage(messages.proofImagesLabel) }}
						<span
							v-if="!!PERMISSION_REASONS[selectedKind].proofImagesOptional"
							class="font-normal text-primary"
							>{{ formatMessage(messages.optional) }}</span
						>
					</span>
					<span v-if="PERMISSION_REASONS[selectedKind].proofImagesDescription">{{
						formatMessage(PERMISSION_REASONS[selectedKind].proofImagesDescription!)
					}}</span>
				</div>
				<div v-if="proofImageUrls.length > 0" class="grid grid-cols-2 gap-4">
					<div
						v-for="(src, idx) in proofImageUrls"
						:key="`${src}-${idx}`"
						class="relative rounded-xl border-[1px] border-solid border-surface-5 overflow-hidden shrink-0"
					>
						<img
							:src="src"
							:alt="formatMessage(messages.proofImageThumbnailAlt, { n: idx + 1 })"
							class="flex w-full object-contain bg-surface-3"
						/>
						<div class="absolute top-2 right-2">
							<ButtonStyled circular>
								<button
									v-tooltip="formatMessage(messages.proofImageRemove)"
									type="button"
									@click="removeProofImage(idx)"
								>
									<TrashIcon />
								</button>
							</ButtonStyled>
						</div>
					</div>
				</div>
				<div class="grid grid-cols-2 gap-4 mt-2">
					<FileInput
						accept="image/png,image/jpeg,image/gif,image/webp,image/bmp"
						:prompt="formatMessage(messages.proofImagesUploadPrompt)"
						long-style
						should-always-reset
						:max-size="MAX_PROOF_IMAGE_BYTES"
						:disabled="uploadProofImageMutation.isPending.value || saveMutation.isPending.value"
						class="!bg-surface-3"
						@change="handleProofImagesSelected"
					>
						<UploadIcon class="size-5 shrink-0" />
					</FileInput>
				</div>
				<p v-if="uploadProofImageMutation.isError.value" class="text-red text-sm m-0">
					{{ String(uploadProofImageMutation.error.value) }}
				</p>
			</div>
		</div>
		<Admonition
			v-if="selectedKind === 'special_permissions'"
			type="warning"
			:header="formatMessage(messages.proofWarningTitle)"
			:body="formatMessage(messages.proofWarningBody)"
		/>

		<p v-if="inputError" class="text-red text-sm m-0">{{ inputError }}</p>
		<p v-else-if="saveMutation.isError.value" class="text-red text-sm m-0">
			{{ String(saveMutation.error.value) }}
		</p>

		<hr class="mt-1 bg-surface-5 border-none h-[1px] w-full" />
		<div class="flex items-center gap-2 justify-end">
			<ButtonStyled v-if="isAttributed" type="outlined">
				<button
					:disabled="saveMutation.isPending.value || uploadProofImageMutation.isPending.value"
					@click="cancelEditing"
				>
					<XIcon /> {{ formatMessage(commonMessages.cancelButton) }}
				</button>
			</ButtonStyled>
			<ButtonStyled color="brand">
				<button
					:disabled="saveMutation.isPending.value || uploadProofImageMutation.isPending.value"
					@click="handleSave"
				>
					<template v-if="saveMutation.isPending.value">
						<SpinnerIcon class="animate-spin" />
						{{ formatMessage(commonMessages.savingButton) }}
					</template>
					<template v-else-if="isAttributed">
						<SaveIcon /> {{ formatMessage(messages.saveAttribution) }}
					</template>
					<template v-else> <CheckIcon /> {{ formatMessage(messages.addAttribution) }} </template>
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>
