<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="500px" :on-hide="reset">
		<div class="flex flex-col gap-4 pb-1">
			<template v-if="!preview">
				<span class="text-primary">{{ formatMessage(messages.description) }}</span>
				<label for="relink-shared-instance-input" class="m-0 font-semibold text-contrast">
					{{ formatMessage(messages.inputLabel) }}
				</label>
				<Input
					id="relink-shared-instance-input"
					v-model="input"
					autocomplete="off"
					:spellcheck="false"
					:placeholder="formatMessage(messages.inputPlaceholder)"
					:disabled="previewMutation.isPending.value"
					wrapper-class="w-full"
					@keydown.enter="findSharedInstance"
				/>
			</template>
			<template v-else>
				<div class="flex flex-col gap-1 rounded-2xl bg-button-bg p-4">
					<span class="text-lg font-semibold text-contrast">{{ preview.name }}</span>
					<span class="text-secondary">
						{{
							formatMessage(messages.versionSummary, {
								loader: formatLoaderLabel(preview.loader),
								gameVersion: preview.gameVersion,
								version: preview.version,
							})
						}}
					</span>
				</div>
				<Admonition
					v-if="blockerMessage"
					type="critical"
					:header="formatMessage(messages.blockedHeader)"
				>
					{{ blockerMessage }}
				</Admonition>
				<template v-else>
					<Admonition
						v-if="preview.gameVersionMismatch || preview.loaderMismatch"
						type="warning"
						:header="formatMessage(messages.mismatchHeader)"
					>
						{{
							formatMessage(messages.mismatchBody, {
								localLoader: formatLoaderLabel(instance.loader),
								localGameVersion: instance.game_version,
								loader: formatLoaderLabel(preview.loader),
								gameVersion: preview.gameVersion,
							})
						}}
					</Admonition>
					<span class="text-primary">{{ formatMessage(messages.confirmBody) }}</span>
				</template>
			</template>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<template v-if="!preview">
					<Button type="outlined" class="!border" @click="modal?.hide()">
						<XIcon />{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="!input.trim() || previewMutation.isPending.value"
						@click="findSharedInstance"
					>
						<SpinnerIcon v-if="previewMutation.isPending.value" class="animate-spin" />
						<SearchIcon v-else />
						{{ formatMessage(messages.findButton) }}
					</Button>
				</template>
				<template v-else>
					<Button
						type="outlined"
						class="!border"
						:disabled="relinkMutation.isPending.value"
						@click="preview = null"
					>
						<LeftArrowIcon />{{ formatMessage(commonMessages.backButton) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="!!preview.blocker || relinkMutation.isPending.value"
						@click="confirmRelink"
					>
						<SpinnerIcon v-if="relinkMutation.isPending.value" class="animate-spin" />
						<LinkIcon v-else />
						{{ formatMessage(messages.relinkButton) }}
					</Button>
				</template>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { LeftArrowIcon, LinkIcon, SearchIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	Button,
	commonMessages,
	defineMessages,
	formatLoaderLabel,
	Input,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'
import {
	getSharedInstanceRelinkPreview,
	relinkSharedInstance,
	type SharedInstanceRelinkPreview,
} from '@/platform/app-lib/shared-instances/commands'

import { injectInstanceSettings } from '../instance-settings-context.ts'

const { instance } = injectInstanceSettings()
const { formatMessage } = useVIntl()
const { notifySharedInstanceError } = useSharedInstanceErrors()
const queryClient = useQueryClient()

const modal = ref<InstanceType<typeof NewModal>>()
const input = ref('')
const preview = ref<SharedInstanceRelinkPreview | null>(null)
/** Increments each time the modal resets, so lookups from an earlier opening are ignored. */
let session = 0

const previewMutation = useMutation({
	mutationFn: (variables: { instanceId: string; input: string; session: number }) =>
		getSharedInstanceRelinkPreview(variables.instanceId, variables.input),
	onSuccess: (result, variables) => {
		if (variables.session !== session) return
		preview.value = result
	},
	onError: (error, variables) => {
		if (variables.session !== session) return
		notifySharedInstanceError(error)
	},
})

const relinkMutation = useMutation({
	mutationFn: (variables: { instanceId: string; sharedInstanceId: string }) =>
		relinkSharedInstance(variables.instanceId, variables.sharedInstanceId),
	onSuccess: async () => {
		modal.value?.hide()
		await queryClient.invalidateQueries({ queryKey: ['instances'] })
	},
	onError: notifySharedInstanceError,
})

const blockerMessage = computed(() => {
	const blocker = preview.value?.blocker
	switch (blocker?.type) {
		case 'not_owner':
			return formatMessage(messages.notOwner)
		case 'server_managed':
			return formatMessage(messages.serverManaged)
		case 'already_linked':
			return formatMessage(messages.alreadyLinked, { name: blocker.instanceName })
		default:
			return null
	}
})

function findSharedInstance() {
	const value = input.value.trim()
	if (!value || previewMutation.isPending.value) return
	previewMutation.mutate({ instanceId: instance.value.id, input: value, session })
}

function confirmRelink() {
	if (!preview.value || preview.value.blocker || relinkMutation.isPending.value) return
	relinkMutation.mutate({
		instanceId: instance.value.id,
		sharedInstanceId: preview.value.sharedInstanceId,
	})
}

function reset() {
	session++
	previewMutation.reset()
	input.value = ''
	preview.value = null
}

function show(event?: MouseEvent) {
	reset()
	modal.value?.show(event)
}

defineExpose({ show })

const messages = defineMessages({
	header: {
		id: 'installation-settings.relink-shared-instance.modal.header',
		defaultMessage: 'Relink shared instance',
	},
	description: {
		id: 'installation-settings.relink-shared-instance.modal.description',
		defaultMessage: 'Reconnect this instance to a shared instance you own.',
	},
	inputLabel: {
		id: 'installation-settings.relink-shared-instance.modal.input-label',
		defaultMessage: 'Invite link or shared instance ID',
	},
	inputPlaceholder: {
		id: 'installation-settings.relink-shared-instance.modal.input-placeholder',
		defaultMessage: 'https://modrinth.com/share/...',
	},
	findButton: {
		id: 'installation-settings.relink-shared-instance.modal.find-button',
		defaultMessage: 'Find shared instance',
	},
	versionSummary: {
		id: 'installation-settings.relink-shared-instance.modal.version-summary',
		defaultMessage: '{loader} {gameVersion} · Version {version}',
	},
	blockedHeader: {
		id: 'installation-settings.relink-shared-instance.modal.blocked-header',
		defaultMessage: "This instance can't be relinked",
	},
	notOwner: {
		id: 'installation-settings.relink-shared-instance.modal.not-owner',
		defaultMessage:
			'Only the owner can relink a shared instance. To get its updates as a member, install it again from your library.',
	},
	serverManaged: {
		id: 'installation-settings.relink-shared-instance.modal.server-managed',
		defaultMessage: "This shared instance is managed by a server, so it can't be relinked.",
	},
	alreadyLinked: {
		id: 'installation-settings.relink-shared-instance.modal.already-linked',
		defaultMessage: 'Your instance "{name}" is already linked to this shared instance.',
	},
	mismatchHeader: {
		id: 'installation-settings.relink-shared-instance.modal.mismatch-header',
		defaultMessage: 'Different game or loader version',
	},
	mismatchBody: {
		id: 'installation-settings.relink-shared-instance.modal.mismatch-body',
		defaultMessage:
			'This instance uses {localLoader} {localGameVersion}, but the shared instance uses {loader} {gameVersion}. Your next publish will change the shared instance to match this instance.',
	},
	confirmBody: {
		id: 'installation-settings.relink-shared-instance.modal.confirm-body',
		defaultMessage:
			'This instance becomes the source of the shared instance again. Members get your changes when you publish an update.',
	},
	relinkButton: {
		id: 'installation-settings.relink-shared-instance.modal.relink-button',
		defaultMessage: 'Relink',
	},
})
</script>
