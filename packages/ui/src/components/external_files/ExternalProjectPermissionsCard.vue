<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	ChevronDownIcon,
	CurseForgeIcon,
	EditIcon,
	ExternalIcon,
	FileIcon,
	PlusIcon,
	SaveIcon,
	SpinnerIcon,
	TagIcon,
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
import { defineMessages, useVIntl } from '../../composables/i18n'
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
		defaultMessage: 'Type',
	},
	typeLicense: {
		id: 'external-files.permissions-card.type.license',
		defaultMessage: 'License',
	},
	typeMyProject: {
		id: 'external-files.permissions-card.type.my-project',
		defaultMessage: 'My project',
	},
	typeSpecialPermission: {
		id: 'external-files.permissions-card.type.special-permission',
		defaultMessage: 'Special permission',
	},
	typeNoPermission: {
		id: 'external-files.permissions-card.type.no-permission',
		defaultMessage: 'No permission',
	},
	licenseDescription: {
		id: 'external-files.permissions-card.license.description',
		defaultMessage: 'The license of this work permits you to redistribute it in your modpack.',
	},
	myProjectDescription: {
		id: 'external-files.permissions-card.my-project.description',
		defaultMessage: 'Original work created by you.',
	},
	specialPermissionDescription: {
		id: 'external-files.permissions-card.special-permission.description',
		defaultMessage:
			'You have obtained special permission to redistribute this work in your modpack.',
	},
	noPermissionDescription: {
		id: 'external-files.permissions-card.no-permission.description',
		defaultMessage: `You don't have permission to use this work.`,
	},
	licenseLabel: {
		id: 'external-files.permissions-card.license-label',
		defaultMessage: 'License',
	},
	licensedAs: {
		id: 'external-files.permissions-card.licensed-as',
		defaultMessage: 'Licensed:',
	},
	linkLabel: {
		id: 'external-files.permissions-card.link-label',
		defaultMessage: 'Link to work',
	},
	linkPlaceholder: {
		id: 'external-files.permissions-card.link-placeholder',
		defaultMessage: 'https://example.com/link-to-work',
	},
	notesLabel: {
		id: 'external-files.permissions-card.notes-label',
		defaultMessage: 'Notes:',
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
	proofDescription: {
		id: 'external-files.permissions-card.proof-description',
		defaultMessage:
			'Include screenshots of messages, emails, or replies from the copyright owner showing that they granted you permission to redistribute their work in your modpack.',
	},
	proofWarningTitle: {
		id: 'external-files.permissions-card.proof-warning.title',
		defaultMessage: 'Modrinth staff may attempt to verify submitted proof',
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
	linkRequired: {
		id: 'external-files.permissions-card.error.link-required',
		defaultMessage: 'Please provide a link.',
	},
	proofRequired: {
		id: 'external-files.permissions-card.error.proof-required',
		defaultMessage: 'Please provide proof and explanation.',
	},
	defaultGroupTitle: {
		id: 'external-files.permissions-card.fallback-group-title',
		defaultMessage: 'Attribution group {id}',
	},
	unnamedMultiGroupTitle: {
		id: 'external-files.permissions-card.unnamed-multi-group-title',
		defaultMessage: '{filename} + {count} more',
	},
	proofImagesLabel: {
		id: 'external-files.permissions-card.proof-images-label',
		defaultMessage: 'Proof images',
	},
	proofImagesHint: {
		id: 'external-files.permissions-card.proof-images-hint',
		defaultMessage:
			'Optional: upload supporting screenshots (PNG, JPEG, GIF, WebP, or BMP, max 1 MB each).',
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
	proofImageInvalidType: {
		id: 'external-files.permissions-card.error.proof-image-invalid-type',
		defaultMessage: 'Please upload a PNG, JPEG, GIF, WebP, or BMP image.',
	},
	splitFile: {
		id: 'external-files.permissions-card.split-file',
		defaultMessage: 'Remove from group',
	},
	addFilesToGroup: {
		id: 'external-files.permissions-card.add-files-to-group',
		defaultMessage: 'Add files...',
	},
	addFilesModalLoadErrorTitle: {
		id: 'external-files.permissions-card.add-files-modal.load-error.title',
		defaultMessage: 'Could not load files',
	},
	splitFileErrorTitle: {
		id: 'external-files.permissions-card.split-file-error.title',
		defaultMessage: 'Could not split file',
	},
	assignFilesErrorTitle: {
		id: 'external-files.permissions-card.assign-files-error.title',
		defaultMessage: 'Could not add files',
	},
})

const collapsed = ref(true)
const showVersions = ref(false)

const MAX_PROOF_IMAGE_BYTES = 1_048_576

const isAttributed = computed(
	() => props.group.attribution !== null && props.group.attribution !== undefined,
)
const fileCount = computed(() => props.group.files?.length ?? 0)
const firstFileName = computed(() => props.group.files[0]?.name ?? props.group.files[0]?.sha1 ?? '')

const effectiveTitle = computed(() => {
	if (props.group.flame_project_title) {
		return props.group.flame_project_title
	}
	if (firstFileName.value) {
		const base = firstFileName.value.split('/').pop() ?? firstFileName.value
		if (fileCount.value === 1) {
			return base
		}
		return formatMessage(messages.unnamedMultiGroupTitle, {
			filename: base,
			count: fileCount.value - 1,
		})
	}
	return formatMessage(messages.defaultGroupTitle, { id: props.group.id })
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

const licenseIdInput = ref('')
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
	licenseIdInput.value =
		payload && (payload.type === 'license' || payload.type === 'my_project')
			? payload.license_id
			: ''
	linkInput.value =
		payload && (payload.type === 'license' || payload.type === 'special_permission')
			? (payload.link ?? '')
			: ''
	proofInput.value = payload?.type === 'special_permission' ? (payload.proof ?? '') : ''
	notesInput.value = payload?.notes ?? ''
	proofImageUrls.value =
		payload && payload.type !== 'no_permission' && Array.isArray(payload.proof_image_urls)
			? [...payload.proof_image_urls]
			: []
	inputError.value = null
}
resetInputs()

watch(
	() => props.group.id,
	() => {
		editing.value = !isAttributed.value
		resetInputs()
	},
)

watch(
	() => props.group.attribution,
	() => {
		resetInputs()
	},
	{ deep: true },
)

watch(selectedType, (t) => {
	if (t === 'no_permission') {
		proofImageUrls.value = []
	}
})

const PERMISSION_TYPE_LABELS = {
	license: formatMessage(messages.typeLicense),
	my_project: formatMessage(messages.typeMyProject),
	special_permission: formatMessage(messages.typeSpecialPermission),
	no_permission: formatMessage(messages.typeNoPermission),
} satisfies Record<Labrinth.Attribution.Internal.AttributionPermissionType, string>

const licenseOptions = computed<ComboboxOption<string>[]>(() =>
	builtinLicenses
		.filter((license) => license.short !== '')
		.map((license) => ({
			value: license.short,
			label: license.friendly,
		})),
)

const friendlyLicenseLabel = computed(() => {
	if (!initialAttribution.value) {
		return ''
	}
	if (
		initialAttribution.value.type !== 'license' &&
		initialAttribution.value.type !== 'my_project'
	) {
		return ''
	}
	const builtin = builtinLicenses.find(
		(license) => license.short === initialAttribution.value!.license_id,
	)
	return builtin?.friendly ?? initialAttribution.value.license_id
})

function buildEditedData(): Labrinth.Attribution.Internal.AttributionData | null {
	inputError.value = null
	const notes = notesInput.value.trim() || undefined
	switch (selectedType.value) {
		case 'license': {
			if (!licenseIdInput.value) {
				inputError.value = formatMessage(messages.licenseRequired)
				return null
			}
			return {
				type: 'license',
				license_id: licenseIdInput.value,
				link: linkInput.value.trim() || undefined,
				notes,
				...(proofImageUrls.value.length > 0 ? { proof_image_urls: [...proofImageUrls.value] } : {}),
			}
		}
		case 'my_project': {
			if (!licenseIdInput.value) {
				inputError.value = formatMessage(messages.licenseRequired)
				return null
			}
			return {
				type: 'my_project',
				license_id: licenseIdInput.value,
				notes,
				...(proofImageUrls.value.length > 0 ? { proof_image_urls: [...proofImageUrls.value] } : {}),
			}
		}
		case 'special_permission': {
			const link = linkInput.value.trim()
			const proof = proofInput.value.trim()
			if (!link) {
				inputError.value = formatMessage(messages.linkRequired)
				return null
			}
			if (!proof) {
				inputError.value = formatMessage(messages.proofRequired)
				return null
			}
			return {
				type: 'special_permission',
				link,
				proof,
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
			throw new Error(formatMessage(messages.proofImageInvalidType))
		}
		const result = await client.request<Labrinth.Images.v3.UploadedImage>('/image', {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			params: {
				context: 'project',
				project_id: props.projectId,
				ext,
			},
			body: file,
			headers: {
				'Content-Type': '',
			},
		})
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
			title: formatMessage(messages.assignFilesErrorTitle),
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
			title: formatMessage(messages.splitFileErrorTitle),
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
			title: formatMessage(messages.addFilesModalLoadErrorTitle),
			text: error instanceof Error ? error.message : String(error),
		})
	}
}
</script>

<template>
	<div
		class="bg-surface-2 p-0 rounded-2xl flex flex-col border-[1px] border-solid border-surface-5 overflow-hidden"
	>
		<div class="flex items-center bg-surface-3 pr-4 gap-3">
			<button
				class="flex grow items-center m-0 appearance-none p-4 bg-transparent group transition-all gap-3 text-left min-w-0"
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
							v-if="isAttributed"
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
				v-if="!!group.flame_project_link"
				:href="group.flame_project_link"
				target="_blank"
				rel="noopener"
				class="text-link flex items-center"
			>
				<CurseForgeIcon class="size-4 shrink-0 mr-2" aria-hidden="true" />
				View on CurseForge
				<ExternalIcon class="size-3 shrink-0 mb-2 ml-1" />
			</a>
		</div>

		<Collapsible
			:collapsed="collapsed"
			class="border-0 border-solid border-t border-surface-5 rounded-b-2xl"
		>
			<div class="flex flex-col gap-3 p-4">
				<template v-if="true || group.files?.length > 1">
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
				</template>
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
					class="rounded-2xl p-4 mt-2 bg-surface-3 flex flex-col gap-3"
				>
					<div class="flex items-start justify-between gap-3">
						<span class="text-contrast font-semibold">
							{{
								formatMessage(
									messages[
										initialAttribution.type === 'license'
											? 'typeLicense'
											: initialAttribution.type === 'my_project'
												? 'typeMyProject'
												: initialAttribution.type === 'special_permission'
													? 'typeSpecialPermission'
													: 'typeNoPermission'
									],
								)
							}}
						</span>
						<ButtonStyled>
							<button @click="startEditing">
								<EditIcon /> {{ formatMessage(commonMessages.editButton) }}
							</button>
						</ButtonStyled>
					</div>
					<div
						v-if="initialAttribution.type === 'license' || initialAttribution.type === 'my_project'"
						class="flex flex-col gap-3"
					>
						<div class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2 items-baseline">
							<span class="text-secondary font-medium">
								{{ formatMessage(messages.licensedAs) }}
							</span>
							<span class="text-primary">{{ friendlyLicenseLabel }}</span>
							<template v-if="initialAttribution.type === 'license' && initialAttribution.link">
								<span class="text-secondary font-medium">
									{{ formatMessage(messages.linkLabel) }}
								</span>
								<a
									:href="initialAttribution.link"
									target="_blank"
									rel="noopener"
									class="text-link truncate"
								>
									{{ initialAttribution.link }}
								</a>
							</template>
							<template v-if="initialAttribution.notes">
								<span class="text-secondary font-medium">
									{{ formatMessage(messages.notesLabel) }}
								</span>
								<span class="text-primary whitespace-pre-wrap break-words">
									{{ initialAttribution.notes }}
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
					<div
						v-else-if="initialAttribution.type === 'special_permission'"
						class="flex flex-col gap-3"
					>
						<div class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2 items-baseline">
							<template v-if="initialAttribution.link">
								<span class="text-secondary font-medium">
									{{ formatMessage(messages.linkLabel) }}
								</span>
								<a
									:href="initialAttribution.link"
									target="_blank"
									rel="noopener"
									class="text-link truncate"
								>
									{{ initialAttribution.link }}
								</a>
							</template>
							<template v-if="initialAttribution.proof">
								<span class="text-secondary font-medium">
									{{ formatMessage(messages.proofLabel) }}
								</span>
								<span class="text-primary whitespace-pre-wrap break-words">
									{{ initialAttribution.proof }}
								</span>
							</template>
							<template v-if="initialAttribution.notes">
								<span class="text-secondary font-medium">
									{{ formatMessage(messages.notesLabel) }}
								</span>
								<span class="text-primary whitespace-pre-wrap break-words">
									{{ initialAttribution.notes }}
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
					<div
						v-else-if="initialAttribution.type === 'no_permission' && initialAttribution.notes"
						class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2 items-baseline"
					>
						<span class="text-secondary font-medium">
							{{ formatMessage(messages.notesLabel) }}
						</span>
						<span class="text-primary whitespace-pre-wrap break-words">
							{{ initialAttribution.notes }}
						</span>
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
						:format-label="(type) => PERMISSION_TYPE_LABELS[type]"
						:capitalize="false"
					/>
					<template v-if="selectedType === 'license'">
						<span>{{ formatMessage(messages.licenseDescription) }}</span>
						<span class="text-contrast font-semibold mt-1">
							{{ formatMessage(messages.licenseLabel) }}
						</span>
						<Combobox
							v-model="licenseIdInput"
							class="max-w-80"
							:options="licenseOptions"
							:placeholder="formatMessage(messages.licenseLabel)"
						/>
						<span class="text-contrast font-semibold mt-1">
							{{ formatMessage(messages.linkLabel) }}
						</span>
						<StyledInput
							v-model="linkInput"
							type="text"
							class="max-w-[30rem]"
							:placeholder="formatMessage(messages.linkPlaceholder)"
						/>
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
					</template>
					<template v-else-if="selectedType === 'my_project'">
						<span>{{ formatMessage(messages.myProjectDescription) }}</span>
						<span class="text-contrast font-semibold mt-1">
							{{ formatMessage(messages.licenseLabel) }}
						</span>
						<Combobox
							v-model="licenseIdInput"
							class="max-w-80"
							:options="licenseOptions"
							:placeholder="formatMessage(messages.licenseLabel)"
						/>
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
					</template>
					<template v-else-if="selectedType === 'special_permission'">
						<span>{{ formatMessage(messages.specialPermissionDescription) }}</span>
						<span class="text-contrast font-semibold mt-1">
							{{ formatMessage(messages.linkLabel) }}
						</span>
						<StyledInput
							v-model="linkInput"
							type="text"
							class="max-w-[30rem]"
							:placeholder="formatMessage(messages.linkPlaceholder)"
						/>
						<div class="flex flex-col gap-1 mt-1">
							<span class="text-contrast font-semibold">
								{{ formatMessage(messages.proofLabel) }}
							</span>
							<span>{{ formatMessage(messages.proofDescription) }}</span>
						</div>
						<StyledInput
							v-model="proofInput"
							type="text"
							resize="both"
							multiline
							class="max-w-[40rem]"
							:placeholder="formatMessage(messages.notesPlaceholder)"
						/>
					</template>
					<template v-if="selectedType !== 'no_permission'">
						<div class="flex flex-col gap-2 mt-1">
							<span class="text-contrast font-semibold">
								{{ formatMessage(messages.proofImagesLabel) }}
							</span>
							<div v-if="proofImageUrls.length > 0" class="grid grid-cols-2 gap-4">
								<div
									v-for="(src, idx) in proofImageUrls"
									:key="`${src}-${idx}`"
									class="relative rounded-xl border-[1px] border-solid border-surface-5 overflow-hidden shrink-0"
								>
									<img
										src="https://cdn.modrinth.com/data/mOgUt4GM/images/155dd2b006883b168b1279ec0ff21e753946518b.png"
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
					</template>
					<Admonition
						v-if="selectedType === 'special_permission'"
						type="warning"
						:header="formatMessage(messages.proofWarningTitle)"
						:body="formatMessage(messages.proofWarningBody)"
					/>
					<template v-if="selectedType === 'no_permission'">
						<span>{{ formatMessage(messages.noPermissionDescription) }}</span>
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
					</template>

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

				<Collapsible
					v-if="isAttributed && containingVersions.length > 0"
					:collapsed="!showVersions"
				>
					<div
						class="rounded-2xl p-4 mt-2 border-[1px] border-solid border-surface-5 flex flex-wrap gap-2"
					>
						<TagItem v-for="version in containingVersions" :key="version.id">
							<TagIcon /> {{ version.version_number }}
						</TagItem>
					</div>
				</Collapsible>
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
