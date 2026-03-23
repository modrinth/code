<script setup>
import { TrashIcon, UploadIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	ConfirmModal,
	FileInput,
	injectNotificationManager,
	StyledInput,
	UnsavedChangesPopup,
	useSavable,
} from '@modrinth/ui'

import { injectOrganizationContext } from '~/providers/organization-context.ts'

const { addNotification } = injectNotificationManager()
const {
	organization,
	refresh: refreshOrganization,
	hasPermission,
	deleteIcon,
	patchIcon,
	patchOrganization,
} = injectOrganizationContext()

// Icon state (separate from useSavable, like collection page)
const icon = ref(null)
const deletedIcon = ref(false)
const previewImage = ref(null)

const {
	saved,
	current,
	saving,
	hasChanges: hasFieldChanges,
	reset: resetFields,
} = useSavable(
	() => ({
		name: organization.value.name,
		slug: organization.value.slug,
		summary: organization.value.description,
	}),
	async ({ name, slug, summary }) => {
		await patchOrganization({
			...(name !== undefined && { name }),
			...(slug !== undefined && { slug }),
			...(summary !== undefined && { description: summary }),
		})
	},
)

// Combined state for UnsavedChangesPopup
const originalState = computed(() => ({
	...saved.value,
	iconChanged: false,
}))

const modifiedState = computed(() => ({
	...current.value,
	iconChanged: !!(deletedIcon.value || icon.value),
}))

const reset = () => {
	resetFields()
	icon.value = null
	deletedIcon.value = false
	previewImage.value = null
}

const markIconForDeletion = () => {
	deletedIcon.value = true
	icon.value = null
	previewImage.value = null
}

const showPreviewImage = (files) => {
	const reader = new FileReader()

	icon.value = files[0]
	deletedIcon.value = false

	reader.readAsDataURL(icon.value)
	reader.onload = (event) => {
		previewImage.value = event.target.result
	}
}

const orgId = useRouteId()

const save = async () => {
	// Save field changes via useSavable
	if (hasFieldChanges.value) {
		await patchOrganization({
			...(current.value.name !== organization.value.name && { name: current.value.name }),
			...(current.value.slug !== organization.value.slug && { slug: current.value.slug }),
			...(current.value.summary !== organization.value.description && {
				description: current.value.summary,
			}),
		})
	}

	// Handle icon deletion / upload separately
	if (deletedIcon.value) {
		await deleteIcon()
		deletedIcon.value = false
	} else if (icon.value) {
		await patchIcon(icon.value)
		icon.value = null
	}

	// Always refresh after any change
	await refreshOrganization()

	addNotification({
		title: 'Organization updated',
		text: 'Your organization has been updated.',
		type: 'success',
	})
}

const onDeleteOrganization = useClientTry(async () => {
	await useBaseFetch(`organization/${orgId}`, {
		method: 'DELETE',
		apiVersion: 3,
	})

	addNotification({
		title: 'Organization deleted',
		text: 'Your organization has been deleted.',
		type: 'success',
	})

	await navigateTo('/dashboard/organizations')
})
</script>

<template>
	<div class="normal-page__content">
		<ConfirmModal
			ref="modal_deletion"
			:title="`Are you sure you want to delete ${organization.name}?`"
			description="This will delete this organization forever (like *forever* ever)."
			:has-to-type="true"
			proceed-label="Delete"
			:confirmation-text="organization.name"
			@proceed="onDeleteOrganization"
		/>
		<div class="universal-card">
			<div class="label">
				<h3>
					<span class="label__title size-card-header">Organization information</span>
				</h3>
			</div>
			<label for="project-icon">
				<span class="label__title">Icon</span>
			</label>
			<div class="input-group">
				<Avatar
					:src="deletedIcon ? null : previewImage ? previewImage : organization.icon_url"
					:alt="organization.name"
					size="md"
					class="project__icon"
				/>
				<div class="input-stack">
					<FileInput
						id="project-icon"
						:max-size="262144"
						:show-icon="true"
						accept="image/png,image/jpeg,image/gif,image/webp"
						class="btn"
						prompt="Upload icon"
						:disabled="!hasPermission"
						@change="showPreviewImage"
					>
						<UploadIcon />
					</FileInput>
					<Button
						v-if="!deletedIcon && (previewImage || organization.icon_url)"
						:disabled="!hasPermission"
						@click="markIconForDeletion"
					>
						<TrashIcon />
						Remove icon
					</Button>
				</div>
			</div>

			<label for="project-name">
				<span class="label__title">Name</span>
			</label>
			<StyledInput
				id="project-name"
				v-model="current.name"
				:maxlength="2048"
				:disabled="!hasPermission"
			/>

			<label for="project-slug">
				<span class="label__title">URL</span>
			</label>
			<div class="text-input-wrapper">
				<div class="text-input-wrapper__before">https://modrinth.com/organization/</div>
				<StyledInput
					id="project-slug"
					v-model="current.slug"
					:maxlength="64"
					autocomplete="off"
					:disabled="!hasPermission"
				/>
			</div>

			<label for="project-summary">
				<span class="label__title">Summary</span>
			</label>
			<StyledInput
				id="project-summary"
				v-model="current.summary"
				multiline
				:maxlength="256"
				:disabled="!hasPermission"
				resize="vertical"
			/>
		</div>
		<div class="universal-card">
			<div class="label">
				<h3>
					<span class="label__title size-card-header">Delete organization</span>
				</h3>
			</div>
			<p>
				Deleting your organization will transfer all of its projects to the organization owner. This
				action cannot be undone.
			</p>
			<Button color="danger" @click="() => $refs.modal_deletion.show()">
				<TrashIcon />
				Delete organization
			</Button>
		</div>
		<UnsavedChangesPopup
			:original="originalState"
			:modified="modifiedState"
			:saving="saving"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>
