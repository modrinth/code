<template>
	<div class="flex flex-col gap-2.5">
		<div class="flex items-center justify-between">
			<span class="font-semibold text-contrast">{{
				heading ?? formatMessage(messages.sharedInstance)
			}}</span>
			<ButtonStyled type="transparent">
				<button @click="emit('viewContents')">
					<EyeIcon />
					{{ formatMessage(messages.viewContents) }}
				</button>
			</ButtonStyled>
		</div>
		<div class="flex items-center gap-3 rounded-2xl bg-surface-2 p-3">
			<Avatar
				:src="preview.iconUrl"
				:alt="preview.name"
				size="56px"
				no-shadow
				class="!rounded-2xl"
			/>
			<div class="flex min-w-0 flex-col gap-0.5">
				<span class="truncate font-semibold text-contrast">{{ preview.name }}</span>
				<span class="truncate text-sm font-medium text-secondary">
					{{ loaderDisplay }} {{ preview.gameVersion }}
					<template v-if="preview.modCount">
						· {{ formatMessage(messages.modCount, { count: preview.modCount }) }}
					</template>
				</span>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { EyeIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled, defineMessages, formatLoader, useVIntl } from '@modrinth/ui'
import { computed, toRefs } from 'vue'

import type { SharedInstanceInstallPreview } from '@/helpers/install'

const props = defineProps<{ preview: SharedInstanceInstallPreview; heading?: string }>()
const { preview, heading } = toRefs(props)
const emit = defineEmits<{ viewContents: [] }>()
const { formatMessage } = useVIntl()
const loaderDisplay = computed(() =>
	preview.value.loader ? formatLoader(formatMessage, preview.value.loader) : '',
)
const messages = defineMessages({
	sharedInstance: {
		id: 'app.modal.install-to-play.shared-instance',
		defaultMessage: 'Shared instance',
	},
	viewContents: { id: 'app.modal.install-to-play.view-contents', defaultMessage: 'View contents' },
	modCount: {
		id: 'app.modal.install-to-play.mod-count',
		defaultMessage: '{count, plural, one {# mod} other {# mods}}',
	},
})
</script>
