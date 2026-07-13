<template>
	<div class="flex flex-col gap-1">
		<div class="flex justify-between items-center">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.sharedInstance) }}</span>
			<ButtonStyled type="transparent">
				<button @click="emit('viewContents')">
					<EyeIcon />
					{{ formatMessage(messages.viewContents) }}
				</button>
			</ButtonStyled>
		</div>
		<div class="flex items-center gap-3 rounded-xl bg-surface-2 p-3">
			<Avatar :src="preview.iconUrl" :alt="preview.name" size="48px" />
			<div class="flex flex-col gap-0.5">
				<span class="font-semibold text-contrast">{{ preview.name }}</span>
				<span class="text-sm text-secondary">
					{{ loaderDisplay }} {{ preview.gameVersion }}
					<template v-if="preview.modCount">
						<BulletDivider />
						{{ formatMessage(messages.modCount, { count: preview.modCount }) }}
					</template>
				</span>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { EyeIcon } from '@modrinth/assets'
import {
	Avatar,
	BulletDivider,
	ButtonStyled,
	defineMessages,
	formatLoader,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import type { SharedInstanceInstallPreview } from '@/helpers/install'

const props = defineProps<{ preview: SharedInstanceInstallPreview }>()
const emit = defineEmits<{ viewContents: [] }>()
const { formatMessage } = useVIntl()
const loaderDisplay = computed(() =>
	props.preview.loader ? formatLoader(formatMessage, props.preview.loader) : '',
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
