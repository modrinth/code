<template>
	<div class="flex w-[500px] max-w-full flex-col gap-6">
		<p class="m-0 text-base leading-6 text-primary">
			{{ formatMessage(messages.description) }}
		</p>
		<div class="flex flex-col gap-3">
			<h3 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.whatWillHappen) }}
			</h3>
			<ol class="m-0 flex list-none flex-col p-0">
				<li v-for="(item, index) in steps" :key="item.key" class="flex gap-3">
					<div class="flex shrink-0 flex-col items-center">
						<div
							class="flex size-10 items-center justify-center rounded-full border border-solid border-surface-5 bg-surface-4 text-secondary"
						>
							<component :is="item.icon" class="size-5" aria-hidden="true" />
						</div>
						<div v-if="index < steps.length - 1" class="my-1 w-px flex-1 bg-surface-5" />
					</div>
					<div class="flex flex-col gap-1 pt-2" :class="{ 'pb-6': index < steps.length - 1 }">
						<span class="text-base font-semibold text-contrast">
							{{ index + 1 }}. {{ item.title }}
						</span>
						<span class="text-base leading-6 text-secondary">{{ item.description }}</span>
					</div>
				</li>
			</ol>
		</div>
		<Checkbox v-model="acknowledged" :label="formatMessage(messages.acknowledge)" />
		<div class="flex flex-wrap items-center justify-end gap-2">
			<Button @click="emit('back')">
				<XIcon v-if="cancel" aria-hidden="true" />
				<ChevronLeftIcon v-else aria-hidden="true" />
				{{ formatMessage(cancel ? commonMessages.cancelButton : commonMessages.backButton) }}
			</Button>
			<Button
				type="colored"
				color="brand"
				:disabled="!acknowledged || disabled"
				@click="emit('confirm')"
			>
				<RotateCounterClockwiseIcon aria-hidden="true" />
				{{ formatMessage(messages.confirm) }}
			</Button>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	ChevronLeftIcon,
	DatabaseBackupIcon,
	PowerIcon,
	RotateCounterClockwiseIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'

import { Button } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import { serverPlayMigrationMessages } from './messages'

const props = defineProps<{
	modpackName?: string | null
	cancel?: boolean
	disabled?: boolean
}>()

const emit = defineEmits<{
	back: []
	confirm: []
}>()

const { formatMessage } = useVIntl()
const acknowledged = ref(false)

const messages = defineMessages({
	description: {
		id: 'servers.play-migration.description',
		defaultMessage:
			'Since your server’s content was installed before this update, we’ll need to make a few changes to get it ready to share with friends!',
	},
	whatWillHappen: {
		id: 'servers.play-migration.what-will-happen',
		defaultMessage: 'What will happen',
	},
	backupTitle: {
		id: 'servers.play-migration.backup.title',
		defaultMessage: 'Backup your server',
	},
	backupDescription: {
		id: 'servers.play-migration.backup.description',
		defaultMessage: 'We’ll make a backup you can restore if anything goes wrong called “{name}”.',
	},
	reinstallTitle: {
		id: 'servers.play-migration.reinstall.title',
		defaultMessage: 'Reinstall your content',
	},
	reinstallModpackDescription: {
		id: 'servers.play-migration.reinstall.modpack-description',
		defaultMessage:
			'We’ll reinstall {modpack} and enable content for the server, players, or both based on each project’s environment.',
	},
	reinstallCustomDescription: {
		id: 'servers.play-migration.reinstall.custom-description',
		defaultMessage:
			'We’ll reinstall your content and enable it for the server, players, or both based on each project’s environment.',
	},
	restartTitle: {
		id: 'servers.play-migration.restart.title',
		defaultMessage: 'Restart your server',
	},
	restartDescription: {
		id: 'servers.play-migration.restart.description',
		defaultMessage: 'We’ll restart your server which may take a few minutes.',
	},
	acknowledge: {
		id: 'servers.play-migration.acknowledge',
		defaultMessage: 'I understand my server will be reinstalled and restarted.',
	},
	confirm: {
		id: 'servers.play-migration.confirm',
		defaultMessage: 'Backup & reinstall',
	},
})

const steps = computed(() => [
	{
		key: 'backup',
		icon: DatabaseBackupIcon,
		title: formatMessage(messages.backupTitle),
		description: formatMessage(messages.backupDescription, {
			name: formatMessage(serverPlayMigrationMessages.backupName),
		}),
	},
	{
		key: 'reinstall',
		icon: RotateCounterClockwiseIcon,
		title: formatMessage(messages.reinstallTitle),
		description: props.modpackName
			? formatMessage(messages.reinstallModpackDescription, { modpack: props.modpackName })
			: formatMessage(messages.reinstallCustomDescription),
	},
	{
		key: 'restart',
		icon: PowerIcon,
		title: formatMessage(messages.restartTitle),
		description: formatMessage(messages.restartDescription),
	},
])
</script>
