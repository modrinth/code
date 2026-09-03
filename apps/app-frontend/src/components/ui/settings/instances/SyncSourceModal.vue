<script setup lang="ts">
import { RefreshCwIcon, SearchIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	CheckCircleButton,
	commonMessages,
	defineMessages,
	Input,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref, useTemplateRef } from 'vue'

import { getInstanceIconUrl } from '@/helpers/instance'

const props = defineProps<{
	description: string
	sources: {
		id: string
		name: string
		icon_path?: string | null
		eligible: boolean
	}[]
	loading?: boolean
	pending?: boolean
	error?: boolean
}>()

const emit = defineEmits<{
	confirm: []
	close: []
	retry: []
}>()

const selectedInstanceId = defineModel<string>({ required: true })
const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const search = ref('')
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'app.settings.synced-options.choose-sync-source.title',
		defaultMessage: 'Choose a sync source',
	},
	search: {
		id: 'app.settings.synced-options.choose-sync-source.search-placeholder',
		defaultMessage: 'Search instance',
	},
	empty: {
		id: 'app.settings.synced-options.choose-sync-source.no-instances-found',
		defaultMessage: 'No instances found',
	},
	sync: {
		id: 'app.settings.synced-options.choose-sync-source.sync',
		defaultMessage: 'Sync',
	},
	loadError: {
		id: 'app.settings.synced-options.choose-sync-source.load-error',
		defaultMessage: 'Could not load sync sources. Please try again.',
	},
	retry: {
		id: 'app.settings.synced-options.choose-sync-source.retry',
		defaultMessage: 'Try again',
	},
})

const filteredSources = computed(() => {
	const query = search.value.trim().toLocaleLowerCase()
	return props.sources.filter((source) => source.name.toLocaleLowerCase().includes(query))
})
const canConfirm = computed(
	() =>
		!props.loading &&
		!props.pending &&
		!props.error &&
		props.sources.some((source) => source.id === selectedInstanceId.value && source.eligible),
)

function show() {
	search.value = ''
	modal.value?.show()
}

function hide() {
	return modal.value?.hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title)"
		no-padding
		actions-divider
		max-width="560px"
		width="560px"
		:disable-close="pending"
		:on-after-hide="() => emit('close')"
	>
		<p class="m-0 border-0 border-b border-solid border-surface-5 p-6 text-primary">
			{{ description }}
		</p>

		<div class="flex h-[400px] flex-col gap-3 overflow-y-auto bg-surface-2 px-6 py-4">
			<Input
				v-model="search"
				:icon="SearchIcon"
				type="search"
				autocomplete="off"
				:placeholder="formatMessage(messages.search)"
				:aria-label="formatMessage(messages.search)"
				class="shrink-0"
			/>

			<div v-if="loading" class="flex flex-1 items-center justify-center" aria-busy="true">
				<SpinnerIcon class="size-5 animate-spin text-secondary" />
			</div>
			<div v-else-if="error" class="flex flex-1 flex-col items-center justify-center gap-3">
				<p role="alert" class="m-0 text-center text-primary">
					{{ formatMessage(messages.loadError) }}
				</p>
				<Button @click="emit('retry')">{{ formatMessage(messages.retry) }}</Button>
			</div>
			<div
				v-else-if="filteredSources.length === 0"
				class="flex flex-1 items-center justify-center text-secondary"
			>
				{{ formatMessage(messages.empty) }}
			</div>
			<div
				v-else
				role="radiogroup"
				:aria-label="formatMessage(messages.title)"
				class="flex flex-col gap-1"
			>
				<CheckCircleButton
					v-for="source in filteredSources"
					:key="source.id"
					:checked="selectedInstanceId === source.id"
					:disabled="pending || !source.eligible"
					class="min-h-10"
					@click="selectedInstanceId = source.id"
				>
					<span class="size-5 shrink-0 overflow-hidden rounded-[6px]">
						<Avatar
							:src="getInstanceIconUrl(source.icon_path)"
							:alt="source.name"
							:tint-by="source.id"
							size="1.25rem"
							no-shadow
						/>
					</span>
					<span class="min-w-0 flex-1 truncate">{{ source.name }}</span>
				</CheckCircleButton>
			</div>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2 p-2">
				<Button type="outlined" :disabled="pending" @click="hide">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="!canConfirm"
					:loading="pending"
					@click="canConfirm && emit('confirm')"
				>
					<RefreshCwIcon :class="{ 'animate-spin': pending }" aria-hidden="true" />
					{{ formatMessage(messages.sync) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
