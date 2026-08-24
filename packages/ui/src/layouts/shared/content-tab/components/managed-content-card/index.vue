<script setup lang="ts">
import { EyeIcon, WrenchIcon } from '@modrinth/assets'
import { computed } from 'vue'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import { Button, IconButton } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { ManagedContentCardData } from '../../types'
import ManagedContentCardFooter from './managed-content-card-footer.vue'
import ManagedContentCardSummary from './managed-content-card-summary.vue'

const props = withDefaults(
	defineProps<{
		data: ManagedContentCardData
		disabled?: boolean
		disabledText?: string
		showViewContent?: boolean
		showSettings?: boolean
		showPrimaryAction?: boolean
	}>(),
	{
		disabled: false,
		disabledText: undefined,
		showViewContent: false,
		showSettings: false,
		showPrimaryAction: false,
	},
)

const emit = defineEmits<{
	'view-content': []
	settings: []
	'primary-action': [event: MouseEvent]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	modpackContent: {
		id: 'content.managed-card.modpack-content',
		defaultMessage: 'Modpack content',
	},
	sharedContent: {
		id: 'content.managed-card.shared-content',
		defaultMessage: 'Shared content',
	},
	viewContent: {
		id: 'content.managed-card.view-content',
		defaultMessage: 'View content',
	},
	settings: {
		id: 'content.managed-card.settings',
		defaultMessage: 'Installation settings',
	},
})

const title = computed(() =>
	formatMessage(props.data.kind === 'modpack' ? messages.modpackContent : messages.sharedContent),
)
</script>

<template>
	<section
		class="@container overflow-hidden rounded-[20px] border border-solid border-surface-4 shadow-[0_1px_1px_rgba(0,0,0,0.12)]"
		:aria-busy="disabled || undefined"
	>
		<div
			class="flex min-h-[94px] flex-col items-stretch justify-between gap-4 bg-surface-3 px-[18px] py-4 @[700px]:flex-row @[700px]:items-center"
		>
			<div class="flex min-w-0 flex-1 items-center gap-4 pl-0.5">
				<AutoLink v-if="data.kind === 'modpack'" :to="data.manager.link" class="shrink-0">
					<Avatar
						:src="data.manager.iconUrl"
						:alt="data.manager.name"
						size="4rem"
						:tint-by="data.manager.name"
						no-shadow
					/>
				</AutoLink>
				<div class="flex min-w-0 flex-1 flex-col gap-1.5">
					<h2 class="m-0 text-2xl font-semibold leading-8 text-contrast">{{ title }}</h2>
					<ManagedContentCardSummary :summary="data.summary" :installing="data.installing" />
				</div>
			</div>

			<div
				v-if="showViewContent || showSettings"
				class="flex shrink-0 items-center gap-2 self-start"
			>
				<Button
					v-if="showViewContent"
					type="base"
					size="lg"
					:disabled="disabled"
					@click="emit('view-content')"
				>
					<EyeIcon aria-hidden="true" />
					{{ formatMessage(messages.viewContent) }}
				</Button>
				<IconButton
					v-if="showSettings"
					type="outlined"
					size="lg"
					:disabled="disabled"
					:label="formatMessage(messages.settings)"
					@click="emit('settings')"
				>
					<WrenchIcon aria-hidden="true" />
				</IconButton>
			</div>
		</div>

		<div class="h-px bg-surface-4" />

		<ManagedContentCardFooter
			:data="data"
			:disabled="disabled"
			:disabled-text="disabledText"
			:show-primary-action="showPrimaryAction"
			@primary-action="emit('primary-action', $event)"
		/>
	</section>
</template>
