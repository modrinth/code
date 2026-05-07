<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		fade="warning"
		max-width="520px"
		:on-hide="handleHide"
	>
		<div v-if="request" class="flex flex-col gap-5">
			<Admonition type="warning" :header="formatMessage(messages.admonitionHeader)">
				{{
					formatMessage(messages.admonitionBody, {
						project: request.projectName,
						target: targetTypeLabel,
					})
				}}
			</Admonition>

			<p class="m-0 text-primary">
				{{
					formatMessage(messages.info, {
						selected: formatOption(request.selected),
						target: formatOption(request.target),
						targetType: targetTypeLabel,
					})
				}}
			</p>

			<div class="grid gap-2">
				<div class="rounded-xl bg-surface-2 p-3">
					<span class="text-sm font-semibold text-secondary">
						{{ formatMessage(messages.filteredVersionLabel) }}
					</span>
					<p class="m-0 mt-1 font-semibold text-contrast">
						{{ formatOption(request.selected) }}
					</p>
				</div>
				<div class="rounded-xl bg-surface-2 p-3">
					<span class="text-sm font-semibold text-secondary">
						{{
							formatMessage(messages.targetVersionLabel, {
								target: targetTypeLabel,
							})
						}}
					</span>
					<p class="m-0 mt-1 font-semibold text-contrast">
						{{ formatOption(request.target) }}
					</p>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex flex-wrap justify-end gap-2">
				<ButtonStyled type="outlined">
					<button @click="choose(null)">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button @click="choose('selected')">
						<DownloadIcon />
						{{ formatMessage(messages.installFilteredButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button @click="choose('target')">
						<DownloadIcon />
						{{
							formatMessage(messages.installTargetButton, {
								target: targetTypeLabel,
							})
						}}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { DownloadIcon, XIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'
import { formatLoaderLabel } from '#ui/utils/loaders'

export interface BrowseInstallVersionChoiceOption {
	gameVersion?: string
	loader?: string
}

export interface BrowseInstallVersionChoiceRequest {
	projectName: string
	targetType: 'server' | 'instance'
	selected: BrowseInstallVersionChoiceOption
	target: BrowseInstallVersionChoiceOption
}

export type BrowseInstallVersionChoice = 'selected' | 'target'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'browse.install-version-choice.header',
		defaultMessage: 'Choose version to install',
	},
	admonitionHeader: {
		id: 'browse.install-version-choice.admonition-header',
		defaultMessage: 'Version mismatch',
	},
	admonitionBody: {
		id: 'browse.install-version-choice.admonition-body',
		defaultMessage:
			'{project} has a version compatible with your {target}, but your current filters point to a different version.',
	},
	info: {
		id: 'browse.install-version-choice.info',
		defaultMessage:
			'Install the filtered version ({selected}) if you intentionally want that version, or use the {target} version ({target}) for compatibility.',
	},
	filteredVersionLabel: {
		id: 'browse.install-version-choice.filtered-version-label',
		defaultMessage: 'Current filters',
	},
	targetVersionLabel: {
		id: 'browse.install-version-choice.target-version-label',
		defaultMessage: '{target, select, server {Server} other {Instance}} version',
	},
	installFilteredButton: {
		id: 'browse.install-version-choice.install-filtered-button',
		defaultMessage: 'Install filtered version',
	},
	installTargetButton: {
		id: 'browse.install-version-choice.install-target-button',
		defaultMessage: 'Use {target, select, server {server} other {instance}} version',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const request = ref<BrowseInstallVersionChoiceRequest | null>(null)
const resolveChoice = ref<((choice: BrowseInstallVersionChoice | null) => void) | null>(null)

const targetTypeLabel = computed(() => request.value?.targetType ?? 'server')

function formatOption(option: BrowseInstallVersionChoiceOption) {
	return [
		option.gameVersion ? `MC ${option.gameVersion}` : null,
		option.loader ? formatLoaderLabel(option.loader) : null,
	]
		.filter(Boolean)
		.join(' / ')
}

function show(nextRequest: BrowseInstallVersionChoiceRequest) {
	request.value = nextRequest
	modal.value?.show()

	return new Promise<BrowseInstallVersionChoice | null>((resolve) => {
		resolveChoice.value = resolve
	})
}

function choose(choice: BrowseInstallVersionChoice | null) {
	resolveChoice.value?.(choice)
	resolveChoice.value = null
	modal.value?.hide()
}

function handleHide() {
	if (resolveChoice.value) {
		resolveChoice.value(null)
		resolveChoice.value = null
	}
}

defineExpose({
	show,
})
</script>
