<template>
	<FloatingActionBar
		:shown="shown"
		:aria-label="formatMessage(messages.ariaLabel)"
		hide-when-modal-open
	>
		<div class="flex min-w-0 items-center gap-0.5">
			<div
				v-if="selectedCount > 0"
				class="relative h-8 shrink-0"
				:style="{ width: `${iconStackWidth}px` }"
				aria-hidden="true"
			>
				<div
					v-for="(project, index) in visibleProjects"
					:key="project.id"
					v-tooltip="project.name"
					class="absolute top-0 flex h-8 w-8 items-center justify-center overflow-hidden rounded-lg border-[1.5px] border-solid border-surface-3 bg-surface-4"
					:style="{ left: `${index * iconStackOffset}px`, zIndex: visibleProjects.length - index }"
				>
					<Avatar
						:src="project.iconUrl"
						:alt="project.name"
						:tint-by="project.id"
						size="100%"
						no-shadow
						class="selected-project-avatar"
					/>
				</div>
				<div
					v-if="overflowCount > 0"
					class="absolute top-0 flex h-8 w-8 items-center justify-center rounded-lg border-[1.5px] border-solid border-surface-3 bg-surface-4 text-xs font-bold text-contrast"
					:style="{ left: `${visibleProjects.length * iconStackOffset}px`, zIndex: 0 }"
				>
					+{{ overflowCount }}
				</div>
			</div>

			<span class="px-3 py-2 text-base font-semibold text-contrast tabular-nums">
				{{ selectedCountText }}
			</span>
			<div class="mx-0.5 h-6 w-px bg-surface-5" />
			<ButtonStyled type="transparent">
				<button
					type="button"
					class="!text-primary"
					:disabled="isInstallingSelected"
					@click="clearSelected"
				>
					<span>{{ formatMessage(commonMessages.clearButton) }}</span>
				</button>
			</ButtonStyled>
		</div>

		<div class="ml-auto shrink-0">
			<ButtonStyled color="green">
				<button type="button" :disabled="isInstallingSelected" @click="installSelected">
					<PlusIcon />
					{{ installButtonText }}
				</button>
			</ButtonStyled>
		</div>
	</FloatingActionBar>
</template>

<script setup lang="ts">
import { PlusIcon } from '@modrinth/assets'
import { computed } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import FloatingActionBar from '#ui/components/base/FloatingActionBar.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import { injectBrowseManager } from '../providers/browse-manager'
import type { BrowseInstallContext } from '../types'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	ariaLabel: {
		id: 'browse.selected-projects-floating-bar.aria-label',
		defaultMessage: 'Selected projects',
	},
	selectedCount: {
		id: 'browse.selected-projects-floating-bar.selected-count',
		defaultMessage: '{count, plural, one {# project selected} other {# projects selected}}',
	},
	installButton: {
		id: 'browse.selected-projects-floating-bar.install',
		defaultMessage: 'Install {count, plural, one {# project} other {# projects}}',
	},
})

const props = defineProps<{
	installContext?: BrowseInstallContext | null
}>()

const ctx = injectBrowseManager(null)
const installContext = computed(() => props.installContext ?? ctx?.installContext?.value ?? null)
const selectedProjects = computed(() => installContext.value?.selectedProjects ?? [])
const selectedCount = computed(() => selectedProjects.value.length)
const iconStackOffset = 24
const isInstallingSelected = computed(() => installContext.value?.isInstallingSelected ?? false)
const shown = computed(() => selectedCount.value > 0 && !isInstallingSelected.value)
const visibleProjects = computed(() => selectedProjects.value.slice(0, 3))
const overflowCount = computed(() => Math.max(0, selectedCount.value - 3))
const iconStackWidth = computed(() => {
	if (selectedCount.value === 0) return 0
	return (
		32 + (visibleProjects.value.length - 1 + (overflowCount.value > 0 ? 1 : 0)) * iconStackOffset
	)
})
const selectedCountText = computed(() =>
	formatMessage(messages.selectedCount, { count: selectedCount.value }),
)
const installButtonText = computed(() =>
	formatMessage(messages.installButton, { count: selectedCount.value }),
)

function clearSelected() {
	if (isInstallingSelected.value) return
	void (installContext.value?.clearSelected ?? installContext.value?.clearQueued)?.()
}

function installSelected() {
	if (isInstallingSelected.value) return
	void installContext.value?.installSelected?.()
}
</script>

<style scoped>
:deep(.selected-project-avatar) {
	background-color: var(--color-button-bg);
}
</style>
