<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	ChevronDownIcon,
	EditIcon,
	ExternalIcon,
	FileIcon,
	PlusIcon,
	SaveIcon,
	SpinnerIcon,
	TrashIcon,
	UploadIcon,
	UserRoundIcon,
	VersionIcon,
	XIcon,
} from '@modrinth/assets'
import { builtinLicenses } from '@modrinth/utils'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import {
	Admonition,
	ButtonStyled,
	Chips,
	Collapsible,
	Combobox,
	type ComboboxOption,
	IntlFormatted,
	StyledInput,
	TagItem,
} from '#ui/components'
import { Avatar, FileInput } from '#ui/components/base'
import { commonMessages } from '#ui/utils'

import { useFormatDateTime } from '../../composables/format-date-time'
import {
	defineMessage,
	defineMessages,
	type MessageDescriptor,
	useVIntl,
} from '../../composables/i18n'
import {
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
} from '../../providers'
import ExternalProjectAddFilesToGroupModal from './ExternalProjectAddFilesToGroupModal.vue'

const props = defineProps<{
	projectId: string
	group: Labrinth.Attribution.Internal.AttributionGroup
}>()

const emit = defineEmits<{
	(e: 'updated'): void
}>()

const { formatMessage } = useVIntl()
const formatDate = useFormatDateTime({ dateStyle: 'long' })
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { allMembers } = injectProjectPageContext()

const attributorMember = computed(() => {
	const userId = props.group.attributed_by
	if (!userId || !allMembers.value) {
		return null
	}
	return allMembers.value.find((member) => member.user.id === userId) ?? null
})

const attributorLink = computed(() => {
	const id = props.group.attributed_by
	if (!id) return null
	if (attributorMember.value) return `/user/${attributorMember.value.user.username}`
	return `/user/${id}`
})

const attributorLabel = computed(() => {
	if (attributorMember.value) return attributorMember.value.user.username
	return props.group.attributed_by ?? ''
})

const messages = defineMessages({
	pendingBadge: {
		id: 'external-files.permissions-card.badge.pending',
		defaultMessage: 'Pending',
	},
	attributedBadge: {
		id: 'external-files.permissions-card.badge.attributed',
		defaultMessage: 'Attributed',
	},
	noPermissionBadge: {
		id: 'external-files.permissions-card.badge.no-permission',
		defaultMessage: 'No permission',
	},
	fileCount: {
		id: 'external-files.permissions-card.file-count',
		defaultMessage: '{count, plural, one {# file} other {# files}}',
	},
	includedInVersions: {
		id: 'external-files.permissions-card.included-in-versions',
		defaultMessage: 'Included in {count, plural, one {# version} other {# versions}}:',
	},
	includedFiles: {
		id: 'external-files.permissions-card.included-files',
		defaultMessage: 'Included files:',
	},
	notUsedInVersions: {
		id: 'external-files.permissions-card.not-used-in-versions',
		defaultMessage: 'These files are not currently used by any version.',
	},
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
	licensedAs: {
		id: 'external-files.permissions-card.licensed-as',
		defaultMessage: 'Licensed:',
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
	notesOptional: {
		id: 'external-files.permissions-card.notes-optional',
		defaultMessage: '(optional)',
	},
	notesPlaceholder: {
		id: 'external-files.permissions-card.notes-placeholder',
		defaultMessage: 'Write something here...',
	},
	proofLabel: {
		id: 'external-files.permissions-card.proof-label',
		defaultMessage: 'Proof and explanation',
	},
	proofWarningTitle: {
		id: 'external-files.permissions-card.proof-warning.title',
		defaultMessage: 'Modrinth staff may verify submitted proof',
	},
	proofWarningBody: {
		id: 'external-files.permissions-card.proof-warning.body',
		defaultMessage: `If you are found to have lied or manipulated the images uploaded, your project and account may be terminated.`,
	},
	saveAttribution: {
		id: 'external-files.permissions-card.save',
		defaultMessage: 'Save attribution',
	},
	addAttribution: {
		id: 'external-files.permissions-card.add',
		defaultMessage: 'Add attribution',
	},
	lastUpdated: {
		id: 'external-files.permissions-card.last-updated',
		defaultMessage: 'Last updated on {date} by {user}',
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
	splitFile: {
		id: 'external-files.permissions-card.split-file',
		defaultMessage: 'Remove from group',
	},
	addFilesToGroup: {
		id: 'external-files.permissions-card.add-files-to-group',
		defaultMessage: 'Add files...',
	},
	originalProjectPage: {
		id: 'external-files.permissions-card.original-project-page',
		defaultMessage: 'Original project',
	},
})

const collapsed = ref(true)

const MAX_PROOF_IMAGE_BYTES = 1_048_576

const isAttributed = computed(
	() => props.group.attribution !== null && props.group.attribution !== undefined,
)
const fileCount = computed(() => props.group.files?.length ?? 0)

function isValidUrl(raw: string): boolean {
	const s = raw.trim()
	if (!s) return false
	let parsed: URL
	try {
		parsed = new URL(s)
	} catch {
		return false
	}
	return parsed.protocol === 'http:' || parsed.protocol === 'https:'
}

const effectiveTitle = computed(() => {
	if (props.group.flame_project_title) {
		return props.group.flame_project_title
	}
	const firstFileName = props.group.files[0]?.name ?? props.group.files[0]?.sha1 ?? ''
	if (firstFileName) {
		const base = firstFileName.split('/').pop() ?? firstFileName
		if (fileCount.value === 1) {
			return base
		}
		return formatMessage(
			defineMessage({
				id: 'external-files.permissions-card.unnamed-multi-group-title',
				defaultMessage: '{filename} + {count} more',
			}),
			{ filename: base, count: fileCount.value - 1 },
		)
	}
	return formatMessage(
		defineMessage({
			id: 'external-files.permissions-card.fallback-group-title',
			defaultMessage: 'Attribution group {id}',
		}),
		{ id: props.group.id },
	)
})

const containingVersions = computed(() => {
	const versionIds = new Set<string>()
	for (const file of props.group.files ?? []) {
		for (const versionId of file.versions ?? []) {
			versionIds.add(versionId)
		}
	}
	return props.group.versions?.filter((v) => versionIds.has(v.id))
})

const permissionTypes: Labrinth.Attribution.Internal.AttributionPermissionType[] = [
	'license',
	'my_project',
	'special_permission',
	'no_permission',
]

const initialAttribution = computed<Labrinth.Attribution.Internal.AttributionData | null>(() => {
	const raw = props.group.attribution
	if (!raw || typeof raw !== 'object') {
		return null
	}
	const obj = raw as Record<string, unknown>
	const type = obj.type
	if (typeof type !== 'string' || !(permissionTypes as string[]).includes(type)) {
		return null
	}
	return obj as Labrinth.Attribution.Internal.AttributionData
})

const editing = ref(!isAttributed.value)
const selectedType = ref<Labrinth.Attribution.Internal.AttributionPermissionType>(
	initialAttribution.value?.type ?? 'license',
)

/** Combobox value when the user picks a non-SPDX custom license (stored in `custom_license`). */
const CUSTOM_LICENSE_VALUE = '__custom__'

const licenseIdInput = ref('')
const customLicenseInput = ref('')
const linkInput = ref('')
const proofInput = ref('')
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
	selectedType.value = payload?.type ?? 'license'
	if (payload?.type === 'license') {
		if (payload.custom_license) {
			licenseIdInput.value = CUSTOM_LICENSE_VALUE
			customLicenseInput.value = payload.custom_license
		} else {
			licenseIdInput.value = payload.license_id
			customLicenseInput.value = ''
		}
	} else {
		licenseIdInput.value = ''
		customLicenseInput.value = ''
	}
	const linkFallback = props.group.flame_project_link ?? ''
	linkInput.value = !payload
		? linkFallback
		: 'link' in payload
			? (payload.link ?? '')
			: linkFallback
	proofInput.value = payload?.type === 'special_permission' ? (payload.proof ?? '') : ''
	notesInput.value = payload?.notes ?? ''
	proofImageUrls.value =
		payload && 'proof_image_urls' in payload ? (payload.proof_image_urls ?? []) : []
	inputError.value = null
}

resetInputs()

watch(licenseIdInput, (value) => {
	if (value !== CUSTOM_LICENSE_VALUE) {
		customLicenseInput.value = ''
	}
})

watch(
	() => props.group.attribution,
	() => {
		resetInputs()
	},
	{ deep: true },
)

type ProjectPermissionField =
	| 'license_id'
	| 'custom_license'
	| 'link'
	| 'notes'
	| 'proof_image_urls'

const PERMISSION_REASONS = {
	license: {
		label: defineMessage({
			id: 'external-files.permissions-card.reason.license',
			defaultMessage: 'License',
		}),
		description: defineMessage({
			id: 'external-files.permissions-card.license.description',
			defaultMessage: 'The license of this work permits you to redistribute it in your modpack.',
		}),
		proofImagesDescription: defineMessage({
			id: 'external-files.permissions-card.proof-images-description.license',
			defaultMessage:
				'Optional: upload supporting documentation related to this license (PNG, JPEG, GIF, WebP, or BMP, max 1 MB each).',
		}),
		fields: ['license_id', 'custom_license', 'link', 'notes', 'proof_image_urls'] as const,
	},
	my_project: {
		label: defineMessage({
			id: 'external-files.permissions-card.reason.my-project',
			defaultMessage: 'My project',
		}),
		description: defineMessage({
			id: 'external-files.permissions-card.my-project.description',
			defaultMessage: 'Original work created by you.',
		}),
		proofImagesDescription: defineMessage({
			id: 'external-files.permissions-card.proof-images-description.my-project',
			defaultMessage:
				'Optional: upload files that help verify you created this work (PNG, JPEG, GIF, WebP, or BMP, max 1 MB each).',
		}),
		fields: ['notes', 'proof_image_urls'] as const,
	},
	special_permission: {
		label: defineMessage({
			id: 'external-files.permissions-card.reason.special-permission',
			defaultMessage: 'Special permission',
		}),
		description: defineMessage({
			id: 'external-files.permissions-card.special-permission.description',
			defaultMessage:
				'You have obtained special permission to redistribute this work in your modpack.',
		}),
		proofImagesDescription: defineMessage({
			id: 'external-files.permissions-card.proof-description',
			defaultMessage:
				'Include screenshots of messages, emails, or replies from the copyright owner showing that they granted you permission to redistribute their work in your modpack.',
		}),
		fields: ['link', 'notes', 'proof_image_urls'] as const,
	},
	no_permission: {
		label: defineMessage({
			id: 'external-files.permissions-card.reason.no-permission',
			defaultMessage: 'No permission',
		}),
		description: defineMessage({
			id: 'external-files.permissions-card.no-permission.description',
			defaultMessage: "You don't have permission to use this work.",
		}),
		fields: ['notes'] as const,
	},
} satisfies Record<
	Labrinth.Attribution.Internal.AttributionPermissionType,
	{
		label: MessageDescriptor
		description: MessageDescriptor
		proofImagesDescription?: MessageDescriptor
		fields: ProjectPermissionField[]
	}
>

const permissionReasonFields = computed<ProjectPermissionField[]>(() => {
	return PERMISSION_REASONS[selectedType.value]?.fields ?? []
})

const readViewFields = computed<ProjectPermissionField[]>(() => {
	const type = initialAttribution.value?.type
	if (!type) return []
	return PERMISSION_REASONS[type]?.fields ?? []
})

const unknownLicenseMessage = defineMessage({
	id: 'external-files.permissions-card.license.unknown',
	defaultMessage: 'Unknown',
})

const notesNoneMessage = defineMessage({
	id: 'external-files.permissions-card.notes-none',
	defaultMessage: 'None',
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

const licenseReadDisplay = computed(() => {
	const attr = initialAttribution.value
	if (!attr || attr.type !== 'license') return null
	if (attr.custom_license) {
		return { kind: 'custom' as const, value: attr.custom_license }
	}
	const licenseId = attr.license_id
	if (licenseId) {
		const friendly =
			builtinLicenses.find((license) => license.short === licenseId)?.friendly ?? licenseId
		return { kind: 'standard' as const, value: friendly }
	}
	return { kind: 'unknown' as const, value: formatMessage(unknownLicenseMessage) }
})

function buildEditedData(): Labrinth.Attribution.Internal.AttributionData | null {
	inputError.value = null
	const notes = notesInput.value.trim() || undefined
	switch (selectedType.value) {
		case 'license': {
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
			const linkRaw = linkInput.value.trim()
			let link: string | undefined
			if (linkRaw) {
				if (!isValidUrl(linkRaw)) {
					inputError.value = formatMessage(messages.linkInvalidUrl)
					return null
				}
				link = linkRaw.trim()
			}
			return {
				type: 'license',
				license_id: custom ? '' : licenseIdInput.value,
				...(custom ? { custom_license: customLicense } : {}),
				link,
				notes,
				...(proofImageUrls.value.length > 0 ? { proof_image_urls: [...proofImageUrls.value] } : {}),
			}
		}
		case 'my_project': {
			return {
				type: 'my_project',
				notes,
				...(proofImageUrls.value.length > 0 ? { proof_image_urls: [...proofImageUrls.value] } : {}),
			}
		}
		case 'special_permission': {
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
			if (!isValidUrl(linkRaw)) {
				inputError.value = formatMessage(messages.linkInvalidUrl)
				return null
			}
			return {
				type: 'special_permission',
				link: linkRaw.trim(),
				proof: proofInput.value.trim(),
				notes,
				...(proofImageUrls.value.length > 0 ? { proof_image_urls: [...proofImageUrls.value] } : {}),
			}
		}
		case 'no_permission':
			return {
				type: 'no_permission',
				notes,
			}
	}
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

const addFilesModalRef = ref<InstanceType<typeof ExternalProjectAddFilesToGroupModal> | null>(null)
const pendingSplitSha1 = ref<string | null>(null)

const assignFilesMutation = useMutation({
	mutationFn: async (sha1s: string[]) => {
		for (const sha1 of sha1s) {
			await client.labrinth.attribution_internal.assignFileToGroup({
				sha1,
				target_group_id: props.group.id,
				project_id: props.projectId,
			})
		}
	},
	onSuccess: async () => {
		await queryClient.invalidateQueries({ queryKey: ['project-attribution', props.projectId] })
		emit('updated')
	},
	onError: (error: Error) => {
		addNotification({
			type: 'error',
			title: formatMessage(
				defineMessage({
					id: 'external-files.permissions-card.assign-files-error.title',
					defaultMessage: 'Could not add files',
				}),
			),
			text: error.message,
		})
	},
})

const splitFileMutation = useMutation({
	mutationFn: (sha1: string) =>
		client.labrinth.attribution_internal.splitFile({
			sha1,
			project_id: props.projectId,
		}),
	onMutate(sha1) {
		pendingSplitSha1.value = sha1
	},
	onSettled() {
		pendingSplitSha1.value = null
	},
	onSuccess: async () => {
		await queryClient.invalidateQueries({ queryKey: ['project-attribution', props.projectId] })
		emit('updated')
	},
	onError: (error: Error) => {
		addNotification({
			type: 'error',
			title: formatMessage(
				defineMessage({
					id: 'external-files.permissions-card.split-file-error.title',
					defaultMessage: 'Could not split file',
				}),
			),
			text: error.message,
		})
	},
})

const saveMutation = useMutation({
	mutationFn: (payload: Labrinth.Attribution.Internal.AttributionData) =>
		client.labrinth.attribution_internal.updateGroup(props.group.id, {
			attribution: payload,
		}),
	onSuccess: async () => {
		await queryClient.invalidateQueries({ queryKey: ['project-attribution', props.projectId] })
		editing.value = false
		emit('updated')
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
		editing.value = false
	}
}

function startEditing() {
	editing.value = true
	collapsed.value = false
	resetInputs()
}

function handleSplitFile(sha1: string) {
	splitFileMutation.mutate(sha1)
}

function handleConfirmAddFiles(sha1s: string[]) {
	assignFilesMutation.mutate(sha1s)
}

async function handleAddFilesToGroup(event: MouseEvent) {
	try {
		const groups = await queryClient.ensureQueryData({
			queryKey: ['project-attribution', props.projectId],
			queryFn: () => client.labrinth.attribution_internal.listProjectAttribution(props.projectId),
		})
		addFilesModalRef.value?.show(event, groups)
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(
				defineMessage({
					id: 'external-files.permissions-card.add-files-modal.load-error.title',
					defaultMessage: 'Could not load files',
				}),
			),
			text: error instanceof Error ? error.message : String(error),
		})
	}
}
</script>

<template>
	<div
		class="bg-surface-2 p-0 rounded-2xl flex flex-col border-[1px] border-solid border-surface-5 overflow-hidden"
	>
		<div class="flex items-center bg-surface-3 gap-3">
			<button
				class="flex grow items-center m-0 appearance-none p-4 bg-transparent group transition-all gap-3 text-left min-w-0 outline-offset-[-3px] rounded-2xl"
				:class="{ 'rounded-b-none': !collapsed, 'rounded-r-none': group.flame_project_link }"
				@click="collapsed = !collapsed"
			>
				<ChevronDownIcon
					class="size-6 text-primary transition-transform duration-300 shrink-0 mb-auto"
					:class="{ 'rotate-180': !collapsed }"
				/>
				<span class="flex flex-col items-start min-w-0 group-active:scale-[0.98]">
					<span class="flex items-center gap-2 text-contrast font-semibold min-w-0">
						<span class="truncate">{{ effectiveTitle }}</span>
						<TagItem
							v-if="isAttributed && group.attribution?.type === 'no_permission'"
							:style="{ '--_bg-color': 'var(--color-red-bg)', '--_color': 'var(--color-red)' }"
						>
							{{ formatMessage(messages.noPermissionBadge) }}
						</TagItem>
						<TagItem
							v-else-if="isAttributed"
							:style="{ '--_bg-color': 'var(--color-green-bg)', '--_color': 'var(--color-green)' }"
						>
							{{ formatMessage(messages.attributedBadge) }}
						</TagItem>
						<TagItem
							v-else
							:style="{
								'--_bg-color': 'var(--color-orange-bg)',
								'--_color': 'var(--color-orange)',
							}"
						>
							{{ formatMessage(messages.pendingBadge) }}
						</TagItem>
					</span>
					<span v-if="fileCount > 1" class="text-secondary text-sm font-normal">
						{{ formatMessage(messages.fileCount, { count: fileCount }) }}
					</span>
				</span>
			</button>
			<a
				v-if="!!group.flame_project_link || !!initialAttribution?.link"
				:href="initialAttribution?.link ?? group.flame_project_link"
				target="_blank"
				rel="noopener"
				class="text-link flex items-center mr-4 outline-offset-[4px] rounded-sm"
			>
				{{ formatMessage(messages.originalProjectPage) }}
				<ExternalIcon class="size-3 shrink-0 mb-2 ml-1" />
			</a>
		</div>

		<Collapsible
			:collapsed="collapsed"
			class="border-0 border-solid border-t border-surface-5 rounded-b-2xl"
		>
			<div class="flex flex-col gap-3 p-4">
				<span class="text-contrast font-semibold">
					{{ formatMessage(messages.includedFiles) }}
				</span>
				<div class="grid grid-cols-2 gap-2">
					<span
						v-for="file in group.files"
						:key="file.sha1"
						class="pl-3 rounded-xl grid grid-cols-[auto_1fr_auto] items-center gap-2 border-[1px] border-solid border-surface-5 bg-surface-2"
					>
						<FileIcon class="size-4 shrink-0 my-2" />
						<span class="max-w-[22rem] truncate my-2">
							{{ file.name.split('/').pop() }}
						</span>
						<ButtonStyled v-if="group.files.length > 1" circular size="small">
							<button
								v-tooltip="formatMessage(messages.splitFile)"
								class="m-1"
								type="button"
								:disabled="splitFileMutation.isPending.value"
								@click="handleSplitFile(file.sha1)"
							>
								<SpinnerIcon
									v-if="splitFileMutation.isPending.value && pendingSplitSha1 === file.sha1"
									class="size-4 shrink-0 animate-spin"
								/>
								<XIcon v-else class="size-4 shrink-0" />
							</button>
						</ButtonStyled>
					</span>
					<div>
						<ButtonStyled>
							<button type="button" @click="handleAddFilesToGroup($event)">
								<PlusIcon class="size-4 shrink-0" /> {{ formatMessage(messages.addFilesToGroup) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
				<template v-if="containingVersions.length > 0">
					<span class="text-contrast font-semibold">
						{{ formatMessage(messages.includedInVersions, { count: containingVersions.length }) }}
					</span>
					<div class="flex flex-wrap gap-2">
						<nuxt-link
							v-for="version in containingVersions"
							:key="version.id"
							:to="`/project/${projectId}/version/${version.id}`"
							target="_blank"
							class="px-3 py-2 rounded-xl flex items-center gap-2 border-[1px] border-solid border-surface-5 bg-surface-3"
						>
							<VersionIcon class="size-4 shrink-0" />
							<span class="max-w-[22rem] truncate">
								{{ version.version_number }}
							</span>
						</nuxt-link>
					</div>
				</template>
				<template v-else>
					<span class="text-secondary text-sm">
						{{ formatMessage(messages.notUsedInVersions) }}
					</span>
				</template>

				<div
					v-if="!editing && initialAttribution"
					class="flex gap-4 flex-col rounded-2xl p-4 mt-2 bg-surface-3"
				>
					<div class="flex gap-4">
						<div class="flex flex-col gap-3 w-full">
							<div class="flex items-start justify-between gap-3">
								<span class="text-contrast font-semibold">
									{{ formatMessage(PERMISSION_REASONS[initialAttribution.type].label) }}
								</span>
							</div>
							<div class="flex flex-col gap-3">
								<div class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2 items-baseline">
									<template v-if="initialAttribution.type === 'license'">
										<span class="text-secondary font-medium">
											{{ formatMessage(messages.licensedAs) }}
										</span>
										<a
											v-if="
												licenseReadDisplay?.kind === 'custom' &&
												isValidUrl(licenseReadDisplay.value)
											"
											:href="licenseReadDisplay.value"
											target="_blank"
											rel="noopener"
											class="text-link truncate"
										>
											{{ licenseReadDisplay.value }}
										</a>
										<span v-else class="text-primary whitespace-pre-wrap break-words">
											{{ licenseReadDisplay?.value }}
										</span>
									</template>
									<template v-if="readViewFields.includes('link') && 'link' in initialAttribution">
										<span class="text-secondary font-medium">
											{{ formatMessage(messages.linkLabel) }}
										</span>
										<a
											v-if="initialAttribution.link"
											:href="initialAttribution.link"
											target="_blank"
											rel="noopener"
											class="text-link truncate"
										>
											{{ initialAttribution.link }}
										</a>
										<span v-else class="text-primary">{{ formatMessage(notesNoneMessage) }}</span>
									</template>
									<template v-if="readViewFields.includes('notes')">
										<span class="text-secondary font-medium">
											{{ formatMessage(messages.notesLabel) }}
										</span>
										<span class="text-primary whitespace-pre-wrap break-words">
											{{
												initialAttribution.notes?.trim()
													? initialAttribution.notes
													: formatMessage(notesNoneMessage)
											}}
										</span>
									</template>
								</div>
								<div v-if="initialAttribution.proof_image_urls?.length" class="flex flex-col gap-2">
									<span class="text-secondary font-medium">
										{{ formatMessage(messages.proofImagesLabel) }}
									</span>
									<div class="flex flex-wrap gap-2">
										<a
											v-for="(src, idx) in initialAttribution.proof_image_urls"
											:key="`${src}-${idx}`"
											:href="src"
											target="_blank"
											rel="noopener"
											class="block rounded-xl border-[1px] border-solid border-surface-5 overflow-hidden shrink-0"
										>
											<img
												:src="src"
												:alt="formatMessage(messages.proofImageThumbnailAlt, { n: idx + 1 })"
												class="max-h-40 max-w-full object-contain"
											/>
										</a>
									</div>
								</div>
							</div>
						</div>
						<div>
							<ButtonStyled>
								<button @click="startEditing">
									<EditIcon /> {{ formatMessage(commonMessages.editButton) }}
								</button>
							</ButtonStyled>
						</div>
					</div>

					<div
						v-if="group.attributed_at"
						class="inline-flex items-center flex-wrap gap-x-2 gap-y-1 pt-3 mt-1 border-0 border-t border-solid border-surface-5"
					>
						<IntlFormatted
							:message-id="messages.lastUpdated"
							:values="{ date: formatDate(group.attributed_at) }"
						>
							<template #user>
								<nuxt-link
									:to="attributorLink"
									class="inline-flex items-center gap-1.5 text-primary font-medium hover:underline max-w-full min-w-0"
								>
									<Avatar
										v-if="attributorMember"
										:src="attributorMember.user.avatar_url"
										:alt="attributorMember.user.username"
										size="1rem"
										class="shrink-0"
									/>
									<UserRoundIcon v-else class="size-4 shrink-0" />
									<span class="truncate">{{ attributorLabel }}</span>
								</nuxt-link>
							</template>
						</IntlFormatted>
					</div>
				</div>

				<div
					v-else
					class="rounded-2xl p-4 mt-2 border-[1px] border-solid border-surface-5 flex flex-col gap-3"
				>
					<span class="text-contrast font-semibold">
						{{ formatMessage(messages.typeLabel) }}
					</span>
					<Chips
						v-model="selectedType"
						:items="permissionTypes"
						:format-label="(type) => formatMessage(PERMISSION_REASONS[type].label)"
						:capitalize="false"
					/>
					<span>{{ formatMessage(PERMISSION_REASONS[selectedType].description) }}</span>
					<div v-if="permissionReasonFields.includes('link')" class="flex flex-col gap-2">
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
							<span class="font-normal text-primary">{{
								formatMessage(messages.notesOptional)
							}}</span>
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
					<div
						v-if="permissionReasonFields.includes('proof_image_urls')"
						class="flex flex-col gap-2"
					>
						<div class="flex flex-col gap-2 mt-1">
							<div class="flex flex-col gap-1 mt-1">
								<span class="text-contrast font-semibold">
									{{ formatMessage(messages.proofImagesLabel) }}
								</span>
								<span v-if="PERMISSION_REASONS[selectedType].proofImagesDescription">{{
									formatMessage(PERMISSION_REASONS[selectedType].proofImagesDescription!)
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
									:disabled="
										uploadProofImageMutation.isPending.value || saveMutation.isPending.value
									"
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
						v-if="selectedType === 'special_permission'"
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
						<ButtonStyled v-if="editing && isAttributed" type="outlined">
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
								<template v-else>
									<CheckIcon /> {{ formatMessage(messages.addAttribution) }}
								</template>
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</Collapsible>

		<ExternalProjectAddFilesToGroupModal
			ref="addFilesModalRef"
			:group-id="group.id"
			:pending="assignFilesMutation.isPending.value"
			@confirm="handleConfirmAddFiles"
		/>
	</div>
</template>
