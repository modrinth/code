<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title)"
		max-width="550px"
		:initial-focus="getNameInput"
	>
		<div class="flex w-full flex-col gap-6">
			<RestrictedActionAlert v-model="isRestricted" scope="ORGANIZATION_CREATE" @navigate="hide" />
			<CreateLimitAlert v-model="hasHitLimit" type="org" />
			<Field :label="formatMessage(messages.nameLabel)" :input-id="nameId">
				<Input
					:id="nameId"
					ref="nameInput"
					v-model="name"
					:maxlength="64"
					:placeholder="formatMessage(messages.namePlaceholder)"
					autocomplete="off"
					wrapper-class="w-full"
					aria-required="true"
					:disabled="hasHitLimit"
					@update:model-value="updateSlug"
				/>
			</Field>
			<Field :label="formatMessage(messages.urlLabel)" :input-id="slugId">
				<Input
					:id="slugId"
					v-model="slug"
					:maxlength="64"
					wrapper-class="w-full"
					autocomplete="off"
					autocapitalize="off"
					:spellcheck="false"
					:disabled="hasHitLimit"
					:error="slugTaken"
					:aria-describedby="slugStatus ? slugStatusId : undefined"
					aria-required="true"
					@update:model-value="manualSlug = true"
				>
					<template #prefix>
						<span
							class="block max-w-[48vw] truncate sm:max-w-none"
							title="https://modrinth.com/organization/"
						>
							https://modrinth.com/organization/
						</span>
					</template>
				</Input>
				<template #bottom>
					<div :id="slugStatusId" aria-live="polite" :class="{ 'sr-only': !slugStatus }">
						<p v-if="slugStatus" class="m-0 leading-tight text-secondary">
							{{ slugStatus }}
						</p>
					</div>
				</template>
			</Field>
			<Field :label="formatMessage(messages.summaryLabel)" :input-id="summaryId">
				<Textarea
					:id="summaryId"
					v-model="description"
					:maxlength="256"
					:placeholder="formatMessage(messages.summaryPlaceholder)"
					:disabled="hasHitLimit"
					:aria-describedby="summaryHelpId"
					aria-required="true"
				/>
				<template #subtext>
					<span :id="summaryHelpId">{{ formatMessage(messages.summaryDescription) }}</span>
				</template>
			</Field>
			<div class="flex flex-wrap justify-end gap-2.5">
				<Button type="outlined" @click="hide">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					v-tooltip="validationTooltip"
					type="colored"
					color="brand"
					:disabled="disableCreate"
					@click="createOrganization"
				>
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.createOrganization) }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { ModrinthApiError } from '@modrinth/api-client'
import { PlusIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	Field,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	NewModal,
	Textarea,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { refDebounced } from '@vueuse/core'
import { computed, ref, useId } from 'vue'

import { generateUrlSlug } from '~/composables/project-slug-suggestions'

import CreateLimitAlert from './CreateLimitAlert.vue'
import RestrictedActionAlert from './RestrictedActionAlert.vue'

const router = useNativeRouter()
const { labrinth } = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'create.organization.title',
		defaultMessage: 'Creating an organization',
	},
	nameLabel: {
		id: 'create.organization.name-label',
		defaultMessage: 'Name',
	},
	namePlaceholder: {
		id: 'create.organization.name-placeholder',
		defaultMessage: 'Enter organization name...',
	},
	urlLabel: {
		id: 'create.organization.url-label',
		defaultMessage: 'URL',
	},
	summaryLabel: {
		id: 'create.organization.summary-label',
		defaultMessage: 'Summary',
	},
	summaryDescription: {
		id: 'create.organization.summary-description',
		defaultMessage: 'A sentence or two that describes your organization.',
	},
	summaryPlaceholder: {
		id: 'create.organization.summary-placeholder',
		defaultMessage: 'An organization for...',
	},
	createOrganization: {
		id: 'create.organization.create-organization',
		defaultMessage: 'Create organization',
	},
	invalidName: {
		id: 'create.organization.invalid-name',
		defaultMessage: 'Enter an organization name of 3 to 64 characters.',
	},
	invalidSummary: {
		id: 'create.organization.invalid-summary',
		defaultMessage: 'Enter an organization summary of 3 to 256 characters.',
	},
	invalidUrl: {
		id: 'create.organization.invalid-url',
		defaultMessage:
			'Enter a URL of 3 to 64 characters using letters, numbers, dots, underscores, or hyphens.',
	},
	checkingUrl: {
		id: 'create.organization.checking-url',
		defaultMessage: 'Checking URL availability…',
	},
	urlTaken: {
		id: 'create.organization.url-taken',
		defaultMessage: 'This organization URL is already taken.',
	},
})

const name = ref<string>('')
const slug = ref<string>('')
const description = ref<string>('')
const manualSlug = ref<boolean>(false)
const hasHitLimit = ref<boolean>(false)
const isRestricted = ref<boolean>(false)
const modal = ref<InstanceType<typeof NewModal>>()
const nameInput = ref<InstanceType<typeof Input>>()
const nameId = useId()
const summaryId = useId()
const summaryHelpId = useId()
const slugId = useId()
const slugStatusId = useId()
const isCreating = ref(false)

const trimmedSlug = computed(() => slug.value.trim())
const validSlug = computed(() => /^[a-zA-Z0-9._-]{3,64}$/.test(trimmedSlug.value))
const debouncedSlug = refDebounced(trimmedSlug, 300)
const { data: slugTaken, isFetching: isCheckingSlug } = useQuery({
	queryKey: computed(() => ['organization', 'slug-lookup', trimmedSlug.value] as const),
	queryFn: async ({ queryKey: [, , slug] }) => {
		try {
			await labrinth.organizations_v3.get(slug)
			return true
		} catch (error) {
			if (error instanceof ModrinthApiError && error.statusCode === 404) return false
			throw error
		}
	},
	enabled: computed(
		() =>
			!import.meta.server &&
			!hasHitLimit.value &&
			!isRestricted.value &&
			validSlug.value &&
			debouncedSlug.value === trimmedSlug.value,
	),
	retry: false,
})

const checkingSlug = computed(
	() => validSlug.value && (debouncedSlug.value !== trimmedSlug.value || isCheckingSlug.value),
)
const slugStatus = computed(() => {
	if (checkingSlug.value) return formatMessage(messages.checkingUrl)
	if (slugTaken.value) return formatMessage(messages.urlTaken)
	return ''
})

const validationTooltip = computed(() => {
	const nameLength = [...name.value.trim()].length
	const summaryLength = [...description.value.trim()].length
	if (nameLength < 3 || nameLength > 64) return formatMessage(messages.invalidName)
	if (!validSlug.value) return formatMessage(messages.invalidUrl)
	if (summaryLength < 3 || summaryLength > 256) return formatMessage(messages.invalidSummary)
	return slugStatus.value
})
const disableCreate = computed(
	() => hasHitLimit.value || isRestricted.value || isCreating.value || !!validationTooltip.value,
)

async function createOrganization(): Promise<void> {
	if (disableCreate.value) return
	isCreating.value = true
	startLoading()
	try {
		const value = {
			name: name.value.trim(),
			description: description.value.trim(),
			slug: trimmedSlug.value,
		}

		const result: any = await useBaseFetch('organization', {
			method: 'POST',
			body: JSON.stringify(value),
			apiVersion: 3,
		})

		queryClient.setQueryData(['organization', 'slug-lookup', value.slug], true)
		modal.value?.hide()

		await router.push(`/organization/${result.slug}`)
	} catch (err: any) {
		console.error(err)
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	} finally {
		isCreating.value = false
		stopLoading()
	}
}

function show(event?: MouseEvent): void {
	name.value = ''
	slug.value = ''
	description.value = ''
	manualSlug.value = false
	modal.value?.show(event)
}

function getNameInput() {
	const root: HTMLElement | undefined = nameInput.value?.$el
	return root?.querySelector<HTMLInputElement>('input:enabled') ?? null
}

function hide(): void {
	modal.value?.hide()
}

function updateSlug(): void {
	if (!manualSlug.value) {
		slug.value = generateUrlSlug(name.value)
	}
}

defineExpose({
	show,
})
</script>
