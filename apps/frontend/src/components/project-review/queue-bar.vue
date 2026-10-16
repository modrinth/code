<template>
	<footer class="shrink-0 border-0 border-t border-solid border-divider">
		<div v-if="navigationError" role="alert" class="px-3 py-2 text-xs text-red">
			{{ formatMessage(messages.queueError) }}
		</div>
		<nav
			class="flex min-h-10 flex-wrap items-center justify-between gap-x-3 gap-y-1 px-2 py-0 pr-3 text-xs text-secondary"
			:aria-label="formatMessage(messages.queueNavigation)"
			:aria-busy="busy"
		>
			<div class="flex items-center gap-3 text-xs font-medium" aria-live="polite">
				<div class="flex gap-0.5">
					<button
						v-tooltip="{ text: formatMessage(commonMessages.settingsLabel), delay: 500 }"
						type="button"
						class="queue-action"
						:aria-label="formatMessage(commonMessages.settingsLabel)"
						@click="settingsModal?.show($event)"
					>
						<SettingsIcon aria-hidden="true" />
					</button>
					<FloatingMenu
						placement="top-start"
						@open="panelMenuOpen = true"
						@close="panelMenuOpen = false"
					>
						<button
							v-tooltip="{ text: formatMessage(messages.panelLayout), delay: 500 }"
							type="button"
							class="queue-action"
							:aria-label="formatMessage(messages.panelLayout)"
							:aria-expanded="panelMenuOpen"
						>
							<LayoutPanelLeftIcon aria-hidden="true" />
						</button>
						<template #popper="{ hide }">
							<div class="flex flex-col gap-1">
								<Button
									type="quiet"
									class="justify-start"
									:disabled="isLoading || poppingOut.length > 0"
									@click="runPanelAction(resetPanelLayout, hide)"
								>
									<RotateCounterClockwiseIcon aria-hidden="true" />
									{{ formatMessage(messages.resetPanelLayout) }}
								</Button>
								<Button
									v-if="detachedTabs.length"
									type="quiet"
									class="justify-start"
									@click="runPanelAction(closeExternalPanels, hide)"
								>
									<MonitorXIcon aria-hidden="true" />
									{{ formatMessage(messages.closePopoutWindows) }}
								</Button>
							</div>
						</template>
					</FloatingMenu>
				</div>

				<span
					>{{ formatMessage(messages.complete) }}
					<span class="ml-1 tabular-nums text-primary">{{
						queue.currentQueue.completed.length
					}}</span></span
				>
				<span class="h-3 w-px bg-divider" aria-hidden="true" />
				<span
					>{{ formatMessage(messages.total) }}
					<span class="ml-1 tabular-nums text-primary">{{ queue.currentQueue.total }}</span></span
				>
			</div>
			<div class="flex items-center">
				<button type="button" class="queue-action" :disabled="busy" @click="exit">
					{{ formatMessage(messages.exit) }}
				</button>
				<span class="mx-1.5 h-3 w-px bg-divider" aria-hidden="true" />
				<button type="button" class="queue-action" :disabled="disabled || !canGoBack" @click="back">
					<LeftArrowIcon class="mb-px" />
					{{ formatMessage(messages.back) }}
				</button>
				<button
					type="button"
					class="queue-action"
					:disabled="disabled || (!inQueue && !completed)"
					@click="next"
				>
					{{
						formatMessage(
							completed ? (remaining.length ? messages.next : messages.finish) : messages.skip,
						)
					}}
					<RightArrowIcon class="mb-px" />
				</button>
			</div>
		</nav>
	</footer>
	<TabbedModal
		ref="settingsModal"
		:header="formatMessage(commonMessages.settingsLabel)"
		:tabs="settingsTabs"
		width="60rem"
	>
		<template #content="{ tab }">
			<component :is="tab.content" v-if="tab?.content" embedded />
		</template>
	</TabbedModal>
</template>

<script setup lang="ts">
import {
	KeyboardIcon,
	LayoutPanelLeftIcon,
	LeftArrowIcon,
	MonitorXIcon,
	RightArrowIcon,
	RotateCounterClockwiseIcon,
	SettingsIcon,
} from '@modrinth/assets'
import {
	Button,
	commonMessages,
	FloatingMenu,
	TabbedModal,
	type TabbedModalTab,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import ModerationKeybinds from '~/components/ui/moderation/settings/ModerationKeybinds.vue'
import ModerationSettings from '~/components/ui/moderation/settings/ModerationSettings.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import { injectProjectReviewContext } from './layout/context'
import { projectReviewMessages as messages } from './messages'
import { useReviewShortcut } from './shortcuts'

const panelMenuOpen = ref(false)
const settingsModal = ref<InstanceType<typeof TabbedModal> | null>(null)
const settingsTabs: TabbedModalTab[] = [
	{
		name: messages.generalSettings,
		icon: SettingsIcon,
		content: ModerationSettings,
	},
	{
		name: messages.keybindsSettings,
		icon: KeyboardIcon,
		content: ModerationKeybinds,
	},
]

const { queue, isLoading, navigation } = injectProjectReviewPageContext()
const { detachedTabs, closeExternalPanels, resetPanelLayout, poppingOut } =
	injectProjectReviewContext()
const {
	busy,
	error: navigationError,
	canGoBack,
	inQueue,
	completed,
	remaining,
	back,
	next,
	exit,
} = navigation
const { formatMessage } = useVIntl()
const disabled = computed(() => busy.value || isLoading.value || !queue.hydrated)

function runPanelAction(action: () => void, hide: () => void) {
	action()
	hide()
}

useReviewShortcut(
	'back',
	() => {
		void back()
	},
	() => !disabled.value && canGoBack.value,
)
useReviewShortcut(
	'next',
	() => {
		void next()
	},
	() => !disabled.value && (inQueue.value || completed.value),
)
useReviewShortcut(
	'exit',
	() => {
		void exit()
	},
	() => !busy.value,
)
</script>

<style scoped>
.queue-action {
	display: inline-flex;
	align-items: center;
	gap: 0.25rem;
	padding: 0.5rem 0.5rem;
	background: transparent;
	color: inherit;
	font: inherit;
	font-weight: 600;
}
.queue-action:hover:not(:disabled) {
	color: var(--color-contrast);
}
.queue-action:disabled {
	cursor: default;
	opacity: 0.4;
}
.queue-action :deep(svg) {
	width: 1rem;
	height: 1rem;
}
</style>
