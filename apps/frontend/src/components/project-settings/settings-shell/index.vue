<script setup lang="ts">
import { injectProjectPageContext } from '@modrinth/ui'
import { ref } from 'vue'

import ProjectModerationDrawer from '~/components/ui/moderation/ProjectModerationDrawer.vue'
import { provideProjectSettingsNavigation } from '~/composables/project-settings-navigation'

import SettingsSidebar from './sidebar.vue'

const { currentMember, projectV3 } = injectProjectPageContext()
const settingsRoot = ref<HTMLElement | null>(null)
provideProjectSettingsNavigation(settingsRoot)

const drawerPinned = ref(false)
</script>

<template>
	<div
		ref="settingsRoot"
		class="project-settings mb-8 flex w-full flex-col gap-4"
		:class="{ 'settings-with-review': drawerPinned }"
	>
		<ProjectModerationDrawer v-if="currentMember && projectV3" @pinned="drawerPinned = $event" />
		<div class="grid gap-6 lg:grid-cols-[1fr_3.25fr]">
			<SettingsSidebar />
			<div class="min-w-0">
				<slot />
			</div>
		</div>
	</div>
</template>

<style scoped>
.project-settings :deep(.settings-section-title) {
	margin: 0 0 12px;
	font-size: 20px;
	line-height: 28px;
	font-weight: 600;
	color: var(--color-contrast);
}

.project-settings :deep(.settings-content-card) {
	margin-top: 0;
	padding: 24px;
	border: 1px solid var(--surface-5);
	border-radius: 16px;
	background: var(--surface-3);
}

@media (max-width: 640px) {
	.project-settings :deep(.settings-content-card) {
		padding: 16px;
	}
}

.project-settings {
	box-sizing: border-box;
	transition: padding-right 240ms cubic-bezier(0.16, 1, 0.3, 1);
}

@media (prefers-reduced-motion: reduce) {
	.project-settings {
		transition: none;
	}
}

@media (min-width: 1280px) {
	.settings-with-review {
		padding-right: var(--project-settings-panel-offset, 0px);
	}
}
</style>
