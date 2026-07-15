<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	InfoIcon,
	IssuesIcon,
	SaveIcon,
	SpinnerIcon,
	TrashIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import { builtinLicenses } from '@modrinth/utils'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import {
	ButtonStyled,
	Chips,
	Combobox,
	type ComboboxOption,
	StyledInput,
	StyledTextarea,
} from '#ui/components'
import { FileInput } from '#ui/components/base'
import { commonMessages } from '#ui/utils'

import { defineMessage, defineMessages, useVIntl } from '../../composables/i18n'
import { injectModrinthClient } from '../../providers'
import {
	attributionLinkToWork,
	attributionProofValidationError,
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
		id: 'external-files.permissions-card.editor.type-label',
		defaultMessage: 'Permission reason',
	},
	licenseLabel: {
		id: 'external-files.permissions-card.editor.license-label',
		defaultMessage: 'License',
	},
	selectLicenseLabel: {
		id: 'external-files.permissions-card.editor.select-license-label',
		defaultMessage: 'Select a license...',
	},
	linkLabel: {
		id: 'external-files.permissions-card.editor.link-label',
		defaultMessage: 'Link to work',
	},
	linkToWorkUrlPlaceholder: {
		id: 'external-files.permissions-card.editor.link-to-work-url-placeholder',
		defaultMessage: 'link-to-work',
	},
	notesLabel: {
		id: 'external-files.permissions-card.editor.notes-label',
		defaultMessage: 'Notes',
	},
	optional: {
		id: 'external-files.permissions-card.editor.input-optional',
		defaultMessage: '(optional)',
	},
	notesPlaceholder: {
		id: 'external-files.permissions-card.editor.notes-placeholder',
		defaultMessage: 'Write something here...',
	},
	proofWarningTitle: {
		id: 'external-files.permissions-card.editor.proof-warning.title',
		defaultMessage: 'Modrinth staff may verify submitted proof',
	},
	proofWarningBody: {
		id: 'external-files.permissions-card.editor.proof-warning.body',
		defaultMessage:
			'If you are found to have lied or manipulated the images uploaded, your project and account may be terminated.',
	},
	saveAttribution: {
		id: 'external-files.permissions-card.editor.save',
		defaultMessage: 'Save attribution',
	},
	addAttribution: {
		id: 'external-files.permissions-card.editor.add',
		defaultMessage: 'Add attribution',
	},
	licenseRequired: {
		id: 'external-files.permissions-card.editor.error.license-required',
		defaultMessage: 'Please select a license.',
	},
	customLicenseLabel: {
		id: 'external-files.permissions-card.editor.custom-license-label',
		defaultMessage: 'Link to license',
	},
	customLicenseMyProjectLabel: {
		id: 'external-files.permissions-card.editor.custom-license-my-project-label',
		defaultMessage: 'License name, preferably a SPDX identifier',
	},
	linkToLicenseUrlPlaceholder: {
		id: 'external-files.permissions-card.editor.link-to-license-url-placeholder',
		defaultMessage: 'link-to-license',
	},
	linkInvalidUrl: {
		id: 'external-files.permissions-card.editor.error.link-invalid-url',
		defaultMessage: 'Link must be a valid URL.',
	},
	proofImagesLabel: {
		id: 'external-files.permissions-card.editor.proof-images-label',
		defaultMessage: 'Proof images',
	},
	proofImagesUploadPrompt: {
		id: 'external-files.permissions-card.editor.proof-images-upload-prompt',
		defaultMessage: 'Drag and drop to upload or click to select an image',
	},
	proofImageThumbnailAlt: {
		id: 'external-files.permissions-card.editor.proof-image-alt',
		defaultMessage: 'Proof screenshot {n}',
	},
	proofImageRemove: {
		id: 'external-files.permissions-card.editor.proof-image-remove',
		defaultMessage: 'Remove image',
	},
	modrinthLinkToWork: {
		id: 'external-files.permissions-card.editor.modrinth-link-to-work',
		defaultMessage: `This appears to be a Modrinth link. If this content is available on Modrinth, your pack was likely exported incorrectly. If you downloaded it from another site, try downloading the Modrinth version instead; sometimes they are not identical files.`,
	},
	arrLabel: {
		id: 'external-files.permissions-card.editor.all-rights-reserved',
		defaultMessage: `All Rights Reserved/No license`,
	},
	exampleSpdxLicense: {
		id: 'external-files.permissions-card.editor.example-spdx-license',
		defaultMessage: 'e.g. MPL-1.1',
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

watch(selectedKind, () => {
	inputError.value = null
})

const permissionReasonFields = computed<ProjectPermissionField[]>(() => {
	return PERMISSION_REASONS[selectedKind.value]?.fields ?? []
})

const selectedPermissionReason = computed(() => PERMISSION_REASONS[selectedKind.value])

const notesAfterProofImages = computed(() => selectedKind.value === 'special_permissions')

const notesInputRows = computed(() => (notesAfterProofImages.value ? 2 : 3))

const proofImagesShowOptional = computed(() => {
	const reason = selectedPermissionReason.value
	return (
		reason.proofRequirement === null ||
		(reason.proofRequirement === 'explanation_or_images' && reason.notesShowsOptional)
	)
})

const attributionFieldSections = computed(() => {
	const fields = permissionReasonFields.value
	const sections: Array<'notes' | 'image_urls'> = []
	if (fields.includes('notes') && !notesAfterProofImages.value) {
		sections.push('notes')
	}
	if (fields.includes('image_urls')) {
		sections.push('image_urls')
	}
	if (fields.includes('notes') && notesAfterProofImages.value) {
		sections.push('notes')
	}
	return sections
})

const isCustomLicense = computed(() => licenseIdInput.value === CUSTOM_LICENSE_VALUE)

const licenseOptions = computed<ComboboxOption<string>[]>(() => [
	...builtinLicenses
		.filter((license) => license.short !== '')
		.map((license) => ({
			value: license.short,
			label:
				license.short === 'All-Rights-Reserved' ? formatMessage(messages.arrLabel) : license.short,
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
				defaultMessage: `Please include a link to your license. If you have none, you should likely select 'All Rights Reserved/No license'`,
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
	const proofError = attributionProofValidationError(
		selectedPermissionReason.value.proofRequirement,
		notes,
		image_urls,
		selectedPermissionReason.value.proofValidationError,
	)
	if (proofError) {
		inputError.value = formatMessage(proofError)
		return null
	}
	const base = {
		notes,
		image_urls,
		updated_by_moderator: false,
	}
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
				...base,
				kind: 'license',
				license,
				link_to_work: linkRaw,
			}
		}
		case 'my_project': {
			const license = buildAttributionLicense()
			if (!license) {
				return null
			}
			return {
				...base,
				kind: 'my_project',
				license,
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
				...base,
				kind: 'special_permissions',
				link_to_work: linkRaw,
			}
		}
		case 'no_permission':
			return {
				...base,
				kind: 'no_permission',
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
			<span
				v-if="
					linkInput.startsWith('https://modrinth.com/') ||
					linkInput.startsWith('https://www.modrinth.com/')
				"
				class="flex text-orange gap-2 font-medium mt-2"
			>
				<IssuesIcon class="shrink-0 mt-0.5" /> {{ formatMessage(messages.modrinthLinkToWork) }}
			</span>
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
				{{
					formatMessage(
						selectedKind === 'my_project'
							? messages.customLicenseMyProjectLabel
							: messages.customLicenseLabel,
					)
				}}
			</span>
			<StyledInput
				v-model="customLicenseInput"
				type="text"
				class="max-w-[40rem]"
				:placeholder="
					selectedKind === 'my_project'
						? formatMessage(messages.exampleSpdxLicense)
						: `https://example.com/${formatMessage(messages.linkToLicenseUrlPlaceholder)}`
				"
			/>
		</div>
		<div class="flex flex-col gap-3">
			<template v-for="section in attributionFieldSections" :key="section">
				<div v-if="section === 'notes'" class="flex flex-col gap-2">
					<div class="flex flex-col gap-1 mt-1">
						<span class="text-contrast font-semibold">
							{{ formatMessage(selectedPermissionReason.notesLabel ?? messages.notesLabel) }}
							<span
								v-if="selectedPermissionReason.notesShowsOptional"
								class="font-normal text-primary"
								>{{ formatMessage(messages.optional) }}</span
							>
						</span>
						<span v-if="selectedPermissionReason.notesDescription">{{
							formatMessage(selectedPermissionReason.notesDescription)
						}}</span>
					</div>
					<StyledTextarea
						v-model="notesInput"
						resize="both"
						:rows="notesInputRows"
						class="max-w-[40rem]"
						:placeholder="formatMessage(messages.notesPlaceholder)"
					/>
				</div>
				<div v-else-if="section === 'image_urls'" class="flex flex-col gap-2">
					<div class="flex flex-col gap-2 mt-1">
						<div class="flex flex-col gap-1 mt-1">
							<span class="text-contrast font-semibold">
								{{ formatMessage(messages.proofImagesLabel) }}
								<span v-if="proofImagesShowOptional" class="font-normal text-primary">{{
									formatMessage(messages.optional)
								}}</span>
							</span>
							<span v-if="selectedPermissionReason.proofImagesDescription">{{
								formatMessage(selectedPermissionReason.proofImagesDescription)
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
						<div class="grid grid-cols-2 gap-4">
							<FileInput
								accept="image/png,image/jpeg,image/gif,image/webp,image/bmp"
								:prompt="formatMessage(messages.proofImagesUploadPrompt)"
								long-style
								should-always-reset
								:max-size="MAX_PROOF_IMAGE_BYTES"
								:disabled="uploadProofImageMutation.isPending.value || saveMutation.isPending.value"
								class="!bg-surface-3 !border-surface-5"
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
			</template>
		</div>
		<div
			v-if="selectedKind === 'my_project' || selectedKind === 'special_permissions'"
			class="grid grid-cols-[auto_1fr] gap-2"
		>
			<div class="flex flex-col items-center">
				<InfoIcon class="size-5 text-blue" />
				<div class="w-[2px] flex-grow bg-blue mt-[-1px]"></div>
			</div>
			<div class="flex flex-col gap-2">
				<span class="font-medium leading-[1.25] text-blue">{{
					formatMessage(messages.proofWarningTitle)
				}}</span>
				<span class="text-contrast">{{ formatMessage(messages.proofWarningBody) }}</span>
			</div>
		</div>

		<p v-if="inputError" class="text-red m-0">{{ inputError }}</p>
		<p v-else-if="saveMutation.isError.value" class="text-red m-0">
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
