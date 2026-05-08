<template>
	<FloatingActionBar :shown="shown" :aria-label="formatMessage(messages.ariaLabel)">
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
					class="absolute top-0 flex h-8 w-8 items-center justify-center overflow-hidden rounded-lg border-[1.5px] border-solid border-surface-3 bg-surface-4"
					:style="{ left: `${index * 26}px`, zIndex: index + 1 }"
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
					:style="{ left: `${visibleProjects.length * 26}px`, zIndex: visibleProjects.length + 1 }"
				>
					+{{ overflowCount }}
				</div>
			</div>

			<span class="px-4 py-2.5 text-base font-semibold text-contrast tabular-nums">
				{{ selectedCountText }}
			</span>
			<div class="mx-1 h-6 w-px bg-surface-5" />
			<ButtonStyled type="transparent">
				<button
					type="button"
					class="!text-primary"
					:disabled="isInstallingSelected"
					@click="clearSelected"
				>
					<span>{{ formatMessage(messages.clearButton) }}</span>
				</button>
			</ButtonStyled>
		</div>

		<div class="ml-auto shrink-0">
			<ButtonStyled color="green">
				<button type="button" :disabled="isInstallingSelected" @click="installSelected">
					<SpinnerIcon v-if="isInstallingSelected" class="animate-spin" />
					<PlusIcon v-else />
					{{ installButtonText }}
				</button>
			</ButtonStyled>
		</div>

		<div v-if="isInstallingSelected" class="absolute bottom-0 left-0 right-0 h-1">
			<div
				class="h-full rounded-l-full bg-brand transition-[width] duration-200 ease-in-out"
				:style="{ width: `${installProgressPercent}%` }"
				role="progressbar"
				:aria-valuenow="installProgress.completed"
				:aria-valuemin="0"
				:aria-valuemax="installProgress.total"
			/>
		</div>
	</FloatingActionBar>
</template>

<script setup lang="ts">
import { PlusIcon, SpinnerIcon } from '@modrinth/assets'
import { computed, onBeforeUnmount, watch } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'

import Avatar from '#ui/components/base/Avatar.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import FloatingActionBar from '#ui/components/base/FloatingActionBar.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

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
	clearButton: {
		id: 'browse.selected-projects-floating-bar.clear',
		defaultMessage: 'Clear',
	},
	installButton: {
		id: 'browse.selected-projects-floating-bar.install',
		defaultMessage: 'Install {count, plural, one {# project} other {# projects}}',
	},
	installingButton: {
		id: 'browse.selected-projects-floating-bar.installing',
		defaultMessage: 'Installing',
	},
	leaveInstallingWarning: {
		id: 'browse.selected-projects-floating-bar.leave-installing-warning',
		defaultMessage: 'Selected projects are still being installed. Are you sure you want to leave?',
	},
})

const props = defineProps<{
	installContext?: BrowseInstallContext | null
}>()

const ctx = injectBrowseManager(null)
const installContext = computed(() => props.installContext ?? ctx?.installContext?.value ?? null)
const selectedProjects = computed(() => installContext.value?.selectedProjects ?? [])
const selectedCount = computed(() => selectedProjects.value.length)
const isInstallingSelected = computed(() => installContext.value?.isInstallingSelected ?? false)
const installProgress = computed(
	() => installContext.value?.installProgress ?? { completed: 0, total: selectedCount.value },
)
const shown = computed(() => selectedCount.value > 0 || isInstallingSelected.value)
const visibleProjects = computed(() => selectedProjects.value.slice(0, 3))
const overflowCount = computed(() => Math.max(0, selectedCount.value - 3))
const iconStackWidth = computed(() => {
	if (selectedCount.value === 0) return 0
	return 32 + (visibleProjects.value.length - 1 + (overflowCount.value > 0 ? 1 : 0)) * 26
})
const selectedCountText = computed(() =>
	formatMessage(messages.selectedCount, { count: selectedCount.value }),
)
const installButtonText = computed(() => {
	if (isInstallingSelected.value) return formatMessage(messages.installingButton)
	return formatMessage(messages.installButton, { count: selectedCount.value })
})
const installProgressPercent = computed(() => {
	const total = installProgress.value.total
	if (total <= 0) return 0
	return Math.min(100, Math.max(0, (installProgress.value.completed / total) * 100))
})

function clearSelected() {
	if (isInstallingSelected.value) return
	void (installContext.value?.clearSelected ?? installContext.value?.clearQueued)?.()
}

function installSelected() {
	if (isInstallingSelected.value) return
	void installContext.value?.installSelected?.()
}

function handleBeforeUnload(e: BeforeUnloadEvent) {
	if (!isInstallingSelected.value) return
	e.preventDefault()
	return ''
}

if (typeof window !== 'undefined') {
	watch(
		isInstallingSelected,
		(installing) => {
			if (installing) {
				window.addEventListener('beforeunload', handleBeforeUnload)
			} else {
				window.removeEventListener('beforeunload', handleBeforeUnload)
			}
		},
		{ immediate: true },
	)

	onBeforeUnmount(() => {
		window.removeEventListener('beforeunload', handleBeforeUnload)
	})

	onBeforeRouteLeave(() => {
		if (isInstallingSelected.value) {
			return window.confirm(formatMessage(messages.leaveInstallingWarning))
		}
		return true
	})
}
</script>

<style scoped>
:deep(.selected-project-avatar) {
	background-color: var(--color-button-bg);
}
</style>
