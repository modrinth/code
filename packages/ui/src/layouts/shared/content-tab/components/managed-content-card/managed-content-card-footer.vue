<script setup lang="ts">
import {
	ArrowLeftRightIcon,
	ClockIcon,
	DownloadIcon,
	LockIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import BulletDivider from '#ui/components/base/BulletDivider.vue'
import { Button } from '#ui/components/base/buttons'
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { useRelativeTime } from '#ui/composables/how-ago'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { ManagedContentCardData } from '../../types'

const props = withDefaults(
	defineProps<{
		data: ManagedContentCardData
		showPrimaryAction?: boolean
		disabled?: boolean
		disabledText?: string
	}>(),
	{
		showPrimaryAction: false,
		disabled: false,
		disabledText: undefined,
	},
)

const emit = defineEmits<{
	'primary-action': [event: MouseEvent]
}>()

const { formatMessage } = useVIntl()
const formatTimeAgo = useRelativeTime()
const formatDateTime = useFormatDateTime({
	dateStyle: 'long',
	timeStyle: 'short',
})

const messages = defineMessages({
	providedBy: {
		id: 'content.managed-card.provided-by',
		defaultMessage: 'Provided by',
	},
	server: {
		id: 'content.managed-card.server-suffix',
		defaultMessage: 'server',
	},
	updated: {
		id: 'content.managed-card.updated',
		defaultMessage: 'Updated {time}',
	},
	synced: {
		id: 'content.managed-card.synced',
		defaultMessage: 'Synced {time}',
	},
	switchVersion: {
		id: 'content.managed-card.switch-version',
		defaultMessage: 'Switch version',
	},
	updateAvailable: {
		id: 'content.managed-card.update-available',
		defaultMessage: 'Update available',
	},
	updating: {
		id: 'content.managed-card.updating',
		defaultMessage: 'Updating...',
	},
})

const timestamp = computed(() => {
	if (props.data.kind === 'modpack') {
		return props.data.updatedAt
	}
	return props.data.syncedAt
})

const timestampLabel = computed(() => {
	if (!timestamp.value) return null
	return formatMessage(props.data.kind === 'modpack' ? messages.updated : messages.synced, {
		time: formatTimeAgo(timestamp.value),
	})
})

const timestampTooltip = computed(() =>
	timestamp.value ? formatDateTime(timestamp.value) : undefined,
)

const showAction = computed(
	() =>
		props.showPrimaryAction &&
		(props.data.kind === 'modpack'
			? !!props.data.versionNumber
			: props.data.updateAvailable || props.disabled),
)
</script>

<template>
	<div
		class="flex min-h-14 flex-col items-start justify-between gap-x-4 gap-y-3 bg-surface-2 px-[18px] py-4 !font-semibold text-secondary @[700px]:flex-row @[700px]:items-center"
	>
		<div class="flex min-w-0 flex-wrap items-center gap-2">
			<div class="flex min-w-0 flex-wrap items-center gap-1.5 @[700px]:flex-nowrap">
				<LockIcon aria-hidden="true" class="size-5 shrink-0" />
				<div class="flex min-w-0 flex-wrap items-center gap-[5px] @[700px]:flex-nowrap">
					<span class="whitespace-nowrap">
						{{ formatMessage(messages.providedBy) }}
					</span>
					<AutoLink
						:to="data.manager.link"
						class="flex min-w-0 items-center gap-1.5 text-secondary"
						:class="data.manager.link ? 'hover:underline' : ''"
					>
						<Avatar
							:src="data.manager.iconUrl"
							:alt="data.manager.name"
							size="1.5rem"
							:circle="data.kind === 'shared-instance'"
							:tint-by="data.manager.name"
							no-shadow
							:class="data.kind === 'shared-instance' ? '' : '!rounded-[6px]'"
						/>
						<span class="break-words @[700px]:truncate @[700px]:whitespace-nowrap">
							{{ data.manager.name }}
						</span>
					</AutoLink>
					<span v-if="data.kind === 'server'" class="whitespace-nowrap">
						{{ formatMessage(messages.server) }}
					</span>
				</div>
			</div>

			<template v-if="data.kind === 'modpack' && data.versionNumber">
				<BulletDivider class="!mx-0" />
				<AutoLink
					:to="data.versionLink"
					class="whitespace-nowrap text-secondary !decoration-secondary"
					:class="data.versionLink ? 'hover:underline' : ''"
				>
					{{ data.versionNumber }}
				</AutoLink>
			</template>

			<template v-if="timestampLabel">
				<BulletDivider class="!mx-0" />
				<div
					v-tooltip="timestampTooltip"
					class="flex items-center whitespace-nowrap"
					:class="[data.kind === 'modpack' ? 'gap-2' : 'gap-1.5', 'cursor-help']"
				>
					<ClockIcon aria-hidden="true" class="size-5 shrink-0" />
					<span>{{ timestampLabel }}</span>
				</div>
			</template>
		</div>

		<Button
			v-if="showAction"
			type="quiet"
			:color="data.kind === 'modpack' ? undefined : 'green'"
			:disabled="disabled"
			class=""
			@click="emit('primary-action', $event)"
		>
			<SpinnerIcon v-if="disabled" aria-hidden="true" class="animate-spin" />
			<ArrowLeftRightIcon v-else-if="data.kind === 'modpack'" aria-hidden="true" />
			<DownloadIcon v-else aria-hidden="true" />
			<span>
				{{
					disabled
						? (disabledText ?? formatMessage(messages.updating))
						: data.kind === 'modpack'
							? formatMessage(messages.switchVersion)
							: formatMessage(messages.updateAvailable)
				}}
			</span>
		</Button>
	</div>
</template>
