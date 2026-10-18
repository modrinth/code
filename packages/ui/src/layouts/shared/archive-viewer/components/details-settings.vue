<template>
	<TeleportPopoutMenu
		:label="formatMessage(messages.title)"
		:tooltip="formatMessage(messages.title)"
		type="quiet"
		size="xs"
		icon-only
		placement="bottom-end"
	>
		<template #trigger>
			<SettingsIcon aria-hidden="true" />
		</template>
		<template #panel>
			<div class="flex w-64 flex-col gap-4 p-1 text-sm">
				<fieldset class="m-0 flex flex-col gap-2 border-0 p-0">
					<legend class="mb-2 p-0 font-semibold text-contrast">
						{{ formatMessage(messages.hashes) }}
					</legend>
					<Checkbox
						v-for="type in HASH_TYPES"
						:key="type"
						:label="formatMessage(hashTypeMessages[type])"
						:model-value="settings.hashTypes.value.includes(type)"
						@update:model-value="(checked) => setHashType(type, checked)"
					/>
				</fieldset>
				<fieldset class="m-0 flex flex-col gap-2 border-0 p-0">
					<legend class="mb-2 p-0 font-semibold text-contrast">
						{{ formatMessage(messages.contentCounts) }}
					</legend>
					<RadioButtons v-model="settings.contentCounts.value" :items="DETAIL_VISIBILITIES">
						<template #default="{ item }">
							{{ formatMessage(visibilityMessages[item]) }}
						</template>
					</RadioButtons>
				</fieldset>
			</div>
		</template>
	</TeleportPopoutMenu>
</template>

<script setup lang="ts">
import { SettingsIcon } from '@modrinth/assets'

import { TeleportPopoutMenu } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import RadioButtons from '#ui/components/base/RadioButtons.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { HASH_TYPES, type HashType } from '#ui/helpers/toolbox/file'

import { hashTypeMessages } from '../messages'
import { type DetailVisibility, injectArchiveDetailsSettings } from '../providers/details-settings'

/** Which details the archive viewer shows: the hashes to list and when content counts show. */
const { formatMessage } = useVIntl()
const settings = injectArchiveDetailsSettings()

const DETAIL_VISIBILITIES: DetailVisibility[] = ['always', 'details', 'never']

const messages = defineMessages({
	title: { id: 'archive-viewer.details-settings.title', defaultMessage: 'Details settings' },
	hashes: { id: 'archive-viewer.details-settings.hashes', defaultMessage: 'Hashes to list' },
	contentCounts: {
		id: 'archive-viewer.details-settings.content-counts',
		defaultMessage: 'Content counts',
	},
})

const visibilityMessages = defineMessages({
	always: { id: 'archive-viewer.details-settings.always', defaultMessage: 'Always shown' },
	details: {
		id: 'archive-viewer.details-settings.details',
		defaultMessage: 'Shown with details',
	},
	never: { id: 'archive-viewer.details-settings.never', defaultMessage: 'Never shown' },
} satisfies Record<DetailVisibility, unknown>)

/** Keeps the selection in the order hashes are listed in. */
function setHashType(type: HashType, checked: boolean) {
	const selected = new Set(settings.hashTypes.value)
	if (checked) selected.add(type)
	else selected.delete(type)
	settings.hashTypes.value = HASH_TYPES.filter((candidate) => selected.has(candidate))
}
</script>
