<template>
	<div class="flex w-full flex-col gap-6">
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
							<component :is="item.icon" class="size-6" aria-hidden="true" />
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
			<Button type="outlined" size="lg" @click="emit('back')">
				<XIcon v-if="cancel" aria-hidden="true" />
				<ChevronLeftIcon v-else aria-hidden="true" />
				{{ formatMessage(cancel ? commonMessages.cancelButton : commonMessages.backButton) }}
			</Button>
			<Button
				type="colored"
				color="brand"
				size="lg"
				:disabled="!acknowledged || disabled"
				@click="emit('confirm')"
			>
				<template v-if="continues">
					{{ formatMessage(commonMessages.continueButton) }}
					<RightArrowIcon aria-hidden="true" />
				</template>
				<template v-else>
					<RotateCounterClockwiseIcon aria-hidden="true" />
					{{ formatMessage(serverPlayMigrationMessages.confirm) }}
				</template>
			</Button>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	ChevronLeftIcon,
	DatabaseBackupIcon,
	PowerIcon,
	RightArrowIcon,
	RotateCounterClockwiseIcon,
	XIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import { Button } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import { serverPlayMigrationMessages } from './messages'

const props = defineProps<{
	modpackName?: string | null
	cancel?: boolean
	disabled?: boolean
	/** Whether the pack still has to be provided, so confirming continues instead of reinstalling. */
	continues?: boolean
}>()

const emit = defineEmits<{
	back: []
	confirm: []
}>()

const acknowledged = defineModel<boolean>('acknowledged', { default: false })

const { formatMessage } = useVIntl()

const messages = defineMessages({
	description: {
		id: 'servers.play-migration.description',
		defaultMessage:
			'We need to make a few changes to your server’s content to get it ready to share with your friends!',
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
			'We’ll reinstall {modpack} and any additional content, enabling each project for the server, players, or both based on its environment.',
	},
	reinstallCustomDescription: {
		id: 'servers.play-migration.reinstall.custom-description',
		defaultMessage:
			'We’ll reinstall your server content, enabling each project for the server, players, or both based on its environment.',
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
