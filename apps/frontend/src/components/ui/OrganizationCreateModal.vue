<template>
	<NewModal ref="modal" header="Creating an organization">
		<div class="flex flex-col gap-3">
			<Admonition
				v-if="disabled"
				class="max-w-[30rem]"
				:type="'info'"
				header="Organization creation temporarily disabled"
				body="Due to abuse, we've temporarily disabled the creation of new organizations. We're working hard to restore this feature, thank you for your patience."
			/>
			<div class="flex flex-col gap-2" :class="{ 'opacity-50': disabled }">
				<label for="name">
					<span class="text-lg font-semibold text-contrast">
						Name
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<input
					id="name"
					v-model="name"
					type="text"
					maxlength="64"
					:placeholder="`Enter organization name...`"
					autocomplete="off"
					:disabled="disabled"
					@input="updateSlug"
				/>
			</div>
			<div class="flex flex-col gap-2" :class="{ 'opacity-50': disabled }">
				<label for="slug">
					<span class="text-lg font-semibold text-contrast">
						URL
						<span class="text-brand-red">*</span>
					</span>
				</label>
				<div class="text-input-wrapper">
					<div class="text-input-wrapper__before">https://modrinth.com/organization/</div>
					<input
						id="slug"
						v-model="slug"
						type="text"
						maxlength="64"
						autocomplete="off"
						:disabled="disabled"
						@input="setManualSlug"
					/>
				</div>
			</div>
			<div class="flex flex-col gap-2" :class="{ 'opacity-50': disabled }">
				<label for="additional-information" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">
						Summary
						<span class="text-brand-red">*</span>
					</span>
					<span>A sentence or two that describes your organization.</span>
				</label>
				<div class="textarea-wrapper">
					<textarea
						id="additional-information"
						v-model="description"
						:disabled="disabled"
						maxlength="256"
					/>
				</div>
			</div>
			<p class="m-0 max-w-[30rem]" :class="{ 'opacity-50': disabled }">
				You will be the owner of this organization, but you can invite other members and transfer
				ownership at any time.
			</p>
			<div class="flex gap-2">
				<ButtonStyled color="brand">
					<button :disabled="disabled" @click="createOrganization">
						<PlusIcon aria-hidden="true" />
						Create organization
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button @click="hide">
						<XIcon aria-hidden="true" />
						Cancel
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { PlusIcon, XIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, injectNotificationManager, NewModal } from '@modrinth/ui'
import { ref } from 'vue'

const router = useNativeRouter()
const { addNotification } = injectNotificationManager()

const name = ref<string>('')
const slug = ref<string>('')
const description = ref<string>('')
const manualSlug = ref<boolean>(false)
const modal = ref<InstanceType<typeof NewModal>>()

const disabled = ref(true)

async function createOrganization(): Promise<void> {
	startLoading()
	try {
		const value = {
			name: name.value.trim(),
			description: description.value.trim(),
			slug: slug.value.trim().replace(/ +/g, ''),
		}

		const result: any = await useBaseFetch('organization', {
			method: 'POST',
			body: JSON.stringify(value),
			apiVersion: 3,
		})

		modal.value?.hide()

		await router.push(`/organization/${result.slug}`)
	} catch (err: any) {
		console.error(err)
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}

function show(event?: MouseEvent): void {
	name.value = ''
	description.value = ''
	modal.value?.show(event)
}

function hide(): void {
	modal.value?.hide()
}

function updateSlug(): void {
	if (!manualSlug.value) {
		slug.value = name.value
			.trim()
			.toLowerCase()
			.replaceAll(' ', '-')
			.replaceAll(/[^a-zA-Z0-9!@$()`.+,_"-]/g, '')
			.replaceAll(/--+/gm, '-')
	}
}

function setManualSlug(): void {
	manualSlug.value = true
}

defineExpose({
	show,
})
</script>

<style scoped lang="scss">
.modal-creation {
	input {
		width: 20rem;
		max-width: 100%;
	}

	.text-input-wrapper {
		width: 100%;
	}

	textarea {
		min-height: 5rem;
	}

	.input-group {
		margin-top: var(--gap-md);
	}
}
</style>
