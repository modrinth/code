<script setup lang="ts">
import { TrashIcon, UploadIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	commonMessages,
	commonProjectSettingsMessages,
	ConfirmModal,
	defineMessages,
	Field,
	FileButton,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	Textarea,
	UnsavedChangesPopup,
	useGlobals,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { useObjectUrl } from '@vueuse/core'
import { computed, ref, shallowRef, useId, useTemplateRef, watch } from 'vue'

import DangerZonePanel from '~/components/project-settings/general/danger-zone/panel.vue'
import SettingsSection from '~/components/project-settings/settings-section.vue'
import { injectOrganizationContext } from '~/providers/organization-context'

const messages = defineMessages({
	about: { id: 'organization.settings.about.title', defaultMessage: 'About' },
	name: { id: 'organization.settings.about.name', defaultMessage: 'Name' },
	summary: { id: 'organization.settings.about.summary', defaultMessage: 'Summary' },
	icon: { id: 'organization.settings.about.icon', defaultMessage: 'Icon' },
	optional: { id: 'organization.settings.about.optional', defaultMessage: '(optional)' },
	url: { id: 'organization.settings.about.url', defaultMessage: 'URL' },
	characterCount: {
		id: 'organization.settings.about.character-count',
		defaultMessage: '{count, number} / {limit, number}',
	},
	dangerZone: { id: 'organization.settings.danger-zone', defaultMessage: 'Danger zone' },
	delete: { id: 'organization.settings.delete.title', defaultMessage: 'Delete organization' },
	deleteDescription: {
		id: 'organization.settings.delete.description',
		defaultMessage:
			'Deleting this organization is permanent. All of its projects will transfer to the organization owner.',
	},
	deleteConfirmationTitle: {
		id: 'organization.settings.delete.confirmation.title',
		defaultMessage: 'Are you sure you want to delete this organization?',
	},
	deleteConfirmationAction: {
		id: 'organization.settings.delete.confirmation.action',
		defaultMessage: 'Permanently delete organization',
	},
	updated: { id: 'organization.settings.updated', defaultMessage: 'Organization updated' },
	deleted: { id: 'organization.settings.deleted', defaultMessage: 'Organization deleted' },
	error: { id: 'organization.settings.error', defaultMessage: 'An error occurred' },
})

const nameLimit = 64
const summaryLimit = 256
const { formatMessage } = useVIntl()
const { data: globals } = useGlobals()
const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { organization, canEditDetails, canDelete, deleteIcon, patchIcon, patchOrganization } =
	injectOrganizationContext()
const orgId = useRouteId('organization')
const confirmModal = useTemplateRef('confirmModal')
const aboutTitleId = useId()
const nameId = useId()
const summaryId = useId()
const slugId = useId()
const dangerZoneTitleId = useId()

const saved = computed(() => ({
	name: organization.value?.name ?? '',
	slug: organization.value?.slug ?? '',
	description: organization.value?.description ?? '',
}))
const current = ref({ ...saved.value })
const icon = shallowRef<File>()
const deletedIcon = ref(false)
const previewImage = useObjectUrl(icon)
const original = computed(() => ({ ...saved.value, iconChanged: false }))
const modified = computed(() => ({
	...current.value,
	iconChanged: !!icon.value || deletedIcon.value,
}))

watch(saved, (next, previous) => {
	if (
		current.value.name === previous.name &&
		current.value.slug === previous.slug &&
		current.value.description === previous.description
	) {
		current.value = { ...next }
	}
})

function reset() {
	current.value = { ...saved.value }
	icon.value = undefined
	deletedIcon.value = false
}

function markIconForDeletion() {
	icon.value = undefined
	deletedIcon.value = !!organization.value?.icon_url
}

function showPreviewImage([file]: File[]) {
	if (!file) return
	icon.value = file
	deletedIcon.value = false
}

function notifyError(error: unknown) {
	const data = (error as { data?: { description?: string } } | null)?.data
	addNotification({
		title: formatMessage(messages.error),
		text: data?.description || (error instanceof Error ? error.message : undefined),
		type: 'error',
	})
}

const { mutate: saveChanges, isPending: saving } = useMutation({
	mutationFn: async (id: string) => {
		const changes = {
			...(current.value.name !== saved.value.name && { name: current.value.name }),
			...(current.value.slug !== saved.value.slug && { slug: current.value.slug.toLowerCase() }),
			...(current.value.description !== saved.value.description && {
				description: current.value.description,
			}),
		}
		const fieldsChanged = Object.keys(changes).length > 0
		if (fieldsChanged) await patchOrganization(changes)

		let iconSaved = false
		try {
			if (deletedIcon.value) {
				await deleteIcon()
				deletedIcon.value = false
				iconSaved = true
			} else if (icon.value) {
				await patchIcon(icon.value)
				icon.value = undefined
				iconSaved = true
			}
		} finally {
			try {
				if (fieldsChanged || iconSaved) {
					const latest = await client.labrinth.organizations_v3.get(id)
					const { name, slug, description } = latest
					queryClient.setQueryData(['organization', orgId], latest)
					if (fieldsChanged) current.value = { name, slug, description }
					if (changes.slug) queryClient.setQueryData(['organization', slug], latest)
				}
			} finally {
				if (changes.slug && orgId !== id) {
					await navigateTo(`/organization/${changes.slug}/settings`, { replace: true })
				}
			}
		}
	},
	onSuccess: () => addNotification({ title: formatMessage(messages.updated), type: 'success' }),
	onError: notifyError,
})

const { mutate: deleteOrganization, isPending: deleting } = useMutation({
	mutationFn: (id: string) =>
		client.request<void>(`/organization/${id}`, { api: 'labrinth', version: 3, method: 'DELETE' }),
	onSuccess: async () => {
		await navigateTo('/dashboard/organizations')
		addNotification({ title: formatMessage(messages.deleted), type: 'success' })
	},
	onError: notifyError,
})

const busy = computed(() => saving.value || deleting.value)

function save() {
	if (organization.value && canEditDetails.value && !busy.value) saveChanges(organization.value.id)
}

function confirmDelete() {
	if (organization.value && canDelete.value && !busy.value)
		deleteOrganization(organization.value.id)
}
</script>

<template>
	<div v-if="organization" class="flex flex-col gap-8">
		<SettingsSection :title="formatMessage(messages.about)" :title-id="aboutTitleId">
			<div
				class="flex flex-col gap-6 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4 sm:p-6"
			>
				<Field
					:label="formatMessage(messages.name)"
					:input-id="nameId"
					class="max-w-[600px]"
				>
					<template #right>
						<span class="text-xs text-secondary">
							{{
								formatMessage(messages.characterCount, {
									count: current.name.length,
									limit: nameLimit,
								})
							}}
						</span>
					</template>
					<Input
						:id="nameId"
						v-model="current.name"
						:maxlength="nameLimit"
						wrapper-class="w-full"
						:disabled="!canEditDetails || busy"
					/>
				</Field>

				<Field
					:label="formatMessage(messages.summary)"
					:input-id="summaryId"
					class="max-w-[600px]"
				>
					<template #right>
						<span class="text-xs text-secondary">
							{{
								formatMessage(messages.characterCount, {
									count: current.description.length,
									limit: summaryLimit,
								})
							}}
						</span>
					</template>
					<Textarea
						:id="summaryId"
						v-model="current.description"
						:maxlength="summaryLimit"
						:disabled="!canEditDetails || busy"
						resize="vertical"
					/>
				</Field>

				<Field class="max-w-[600px]">
					<template #label>
						{{ formatMessage(messages.icon) }}
						<span class="font-normal text-secondary">{{ formatMessage(messages.optional) }}</span>
					</template>
					<template #default="{ labelId }">
						<div class="flex items-start gap-2" role="group" :aria-labelledby="labelId">
							<Avatar
								:src="deletedIcon ? null : (previewImage ?? organization.icon_url)"
								:alt="organization.name"
								size="md"
							/>
							<div class="flex flex-col items-start gap-2">
								<FileButton
									:max-size="globals?.max_organization_icon_size"
									accept="image/png,image/jpeg,image/gif,image/webp"
									class="button-like choose-image"
									:prompt="formatMessage(commonProjectSettingsMessages.upload)"
									:disabled="!globals?.max_organization_icon_size || !canEditDetails || busy"
									@change="showPreviewImage"
								>
									<UploadIcon aria-hidden="true" />
								</FileButton>
								<Button
									v-if="!deletedIcon && (previewImage || organization.icon_url)"
									:disabled="!canEditDetails || busy"
									@click="markIconForDeletion"
								>
									<TrashIcon aria-hidden="true" />
									{{ formatMessage(commonMessages.removeButton) }}
								</Button>
							</div>
						</div>
					</template>
				</Field>

				<Field
					:label="formatMessage(messages.url)"
					:input-id="slugId"
					class="max-w-[600px]"
				>
					<Input
						:id="slugId"
						v-model="current.slug"
						:maxlength="64"
						autocomplete="off"
						wrapper-class="w-full"
						:disabled="!canEditDetails || busy"
					>
						<template #prefix>
							<span class="whitespace-nowrap">
								<span class="hidden sm:inline">https://modrinth.com</span>/organization/
							</span>
						</template>
					</Input>
				</Field>
			</div>
		</SettingsSection>

		<SettingsSection
			:title="formatMessage(messages.dangerZone)"
			:title-id="dangerZoneTitleId"
			danger
		>
			<ConfirmModal
				ref="confirmModal"
				:title="formatMessage(messages.deleteConfirmationTitle)"
				:description="formatMessage(messages.deleteDescription)"
				:confirmation-text="organization.name"
				:proceed-label="formatMessage(messages.deleteConfirmationAction)"
				:markdown="false"
				has-to-type
				@proceed="confirmDelete"
			/>
			<DangerZonePanel>
				<div class="danger-zone-row">
					<div>
						<h3 class="mb-1 mt-0 text-base font-semibold text-contrast">
							{{ formatMessage(messages.delete) }}
						</h3>
						<p class="m-0 leading-normal text-secondary">
							{{ formatMessage(messages.deleteDescription) }}
						</p>
					</div>
					<Button
						size="sm"
						type="colored"
						color="red"
						:disabled="!canDelete || busy"
						@click="confirmModal?.show()"
					>
						{{ formatMessage(messages.delete) }}
					</Button>
				</div>
			</DangerZonePanel>
		</SettingsSection>

		<UnsavedChangesPopup
			:original="original"
			:modified="modified"
			:saving="saving"
			:can-save="canEditDetails && !busy"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>
