<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { PlusIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { useMutation } from '@tanstack/vue-query'
import { computed, nextTick, ref, useTemplateRef } from 'vue'

import {
	Accordion,
	ButtonStyled,
	Combobox,
	type ComboboxOption,
	NewModal,
	StyledInput,
} from '#ui/components'

import { injectModrinthClient, injectNotificationManager } from '../../providers'
import AttributionGroupFilePicker from './AttributionGroupFilePicker.vue'
import {
	attributionKindToDefaultExternalStatus,
	buildExternalLicenseProofFromAttribution,
	groupLinkForExternalLicense,
	moderatorAttributionGroupTitle,
	parseInitialAttribution,
} from './external-project-utils'
import type { ExternalLicenseStatus } from './types.ts'

const props = defineProps<{
	group: Labrinth.Attribution.Internal.AttributionGroup
}>()

const emit = defineEmits<{
	(e: 'success'): void
}>()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()

const modalRef = useTemplateRef<InstanceType<typeof NewModal>>('modalRef')

const statusOptions: ComboboxOption<ExternalLicenseStatus>[] = [
	{ value: 'yes', label: 'Yes' },
	{ value: 'with-attribution-and-source', label: 'With attribution and source' },
	{ value: 'with-attribution', label: 'With attribution' },
	{ value: 'no', label: 'No' },
	{ value: 'permanent-no', label: 'Permanent no' },
	{ value: 'unidentified', label: 'Unidentified' },
]

const title = ref('')
const link = ref('')
const flameProjectId = ref('')
const proof = ref('')
const status = ref<ExternalLicenseStatus | undefined>(undefined)
const selectedSha1s = ref<Set<string>>(new Set())

const hasMultipleFiles = computed(() => props.group.files.length > 1)

const createMutation = useMutation({
	mutationFn: async () => {
		if (!status.value) {
			throw new Error('Status is required')
		}

		const parsedFlameProjectId = Number.parseInt(flameProjectId.value.trim(), 10)
		const files = props.group.files
			.filter((file) => selectedSha1s.value.has(file.sha1))
			.map((file) => ({
				sha1: file.sha1,
				name: file.name.split('/').pop() ?? file.name,
			}))

		return client.labrinth.external_projects_internal.create({
			title: title.value.trim() || undefined,
			status: status.value,
			link: link.value.trim() || undefined,
			proof: proof.value.trim() || undefined,
			flame_project_id: Number.isFinite(parsedFlameProjectId) ? parsedFlameProjectId : undefined,
			files,
		})
	},
	onSuccess: async () => {
		addNotification({
			type: 'success',
			title: 'Added to global database',
		})
		await nextTick()
		hide()
		emit('success')
	},
	onError: (error: Error) => {
		addNotification({
			type: 'error',
			title: 'Could not add to global database',
			text: error.message,
		})
	},
})

const canSubmit = computed(
	() =>
		selectedSha1s.value.size > 0 && status.value !== undefined && !createMutation.isPending.value,
)

function resetForm() {
	const attribution = parseInitialAttribution(props.group.attribution)
	title.value = moderatorAttributionGroupTitle(props.group)
	link.value = groupLinkForExternalLicense(props.group, attribution)
	flameProjectId.value = props.group.flame_project?.id?.toString() ?? ''
	proof.value = attribution ? buildExternalLicenseProofFromAttribution(attribution) : ''
	status.value = attribution ? attributionKindToDefaultExternalStatus(attribution.kind) : undefined
	selectedSha1s.value = new Set(props.group.files.map((file) => file.sha1))
}

function handleSubmit() {
	createMutation.mutate()
}

function show(event?: MouseEvent) {
	resetForm()
	modalRef.value?.show(event)
}

function hide() {
	modalRef.value?.hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modalRef"
		header="Adding to global permissions database"
		max-width="640px"
		:disable-close="createMutation.isPending.value"
	>
		<div class="flex flex-col gap-4">
			<div class="flex flex-col gap-2">
				<label class="font-semibold text-contrast" for="add-global-title">Title</label>
				<StyledInput
					id="add-global-title"
					v-model="title"
					type="text"
					:disabled="createMutation.isPending.value"
				/>
			</div>
			<div class="flex flex-col gap-2">
				<label class="font-semibold text-contrast" for="add-global-link">Link</label>
				<StyledInput
					id="add-global-link"
					v-model="link"
					type="text"
					:disabled="createMutation.isPending.value"
				/>
			</div>
			<div class="grid grid-cols-2 gap-4">
				<div class="flex flex-col gap-2">
					<label class="font-semibold text-contrast" for="add-global-flame-id">
						CurseForge project ID
					</label>
					<StyledInput
						id="add-global-flame-id"
						v-model="flameProjectId"
						type="text"
						placeholder="1234567"
						input-class="h-[40px]"
						:disabled="createMutation.isPending.value"
					/>
				</div>
				<div class="flex flex-col gap-2">
					<label class="font-semibold text-contrast" for="add-global-status">Allowed?</label>
					<Combobox
						id="add-global-status"
						v-model="status"
						:options="statusOptions"
						placeholder="Select status"
						class="!w-full min-w-[18rem]"
						:disabled="createMutation.isPending.value"
					/>
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<label class="font-semibold text-contrast" for="add-global-proof">Proof or notes</label>
				<StyledInput
					id="add-global-proof"
					v-model="proof"
					type="text"
					multiline
					input-class="min-h-[6rem]"
					resize="vertical"
					:disabled="createMutation.isPending.value"
				/>
			</div>
			<Accordion
				v-if="hasMultipleFiles"
				class="w-full bg-surface-4 border border-solid border-surface-5 rounded-2xl overflow-clip"
				button-class="p-4 w-full border-b border-solid border-b-surface-5 bg-surface-2 -mb-px hover:brightness-[--hover-brightness] group"
				open-by-default
			>
				<template #title>
					<span class="flex items-center gap-3 text-contrast group-active:scale-[0.98]">
						Select files to add
					</span>
				</template>
				<div>
					<AttributionGroupFilePicker
						v-model:selected-sha1s="selectedSha1s"
						:files="group.files"
						:disabled="createMutation.isPending.value"
					/>
				</div>
			</Accordion>
			<div class="flex justify-end gap-2 w-full">
				<ButtonStyled type="outlined">
					<button type="button" :disabled="createMutation.isPending.value" @click="hide">
						<XIcon class="size-4 shrink-0" />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button type="button" :disabled="!canSubmit" @click="handleSubmit">
						<SpinnerIcon
							v-if="createMutation.isPending.value"
							class="size-4 shrink-0 animate-spin"
						/>
						<PlusIcon v-else class="size-4 shrink-0" />
						Add to global database
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>
