<script setup lang="ts">
import { injectProjectPageContext } from '@modrinth/ui'
import { ref } from 'vue'

import ProjectModerationDrawer from '~/components/ui/project-settings/project-moderation-drawer/index.vue'
import SidebarPageLayout from '~/components/ui/SidebarPageLayout.vue'
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
		<SidebarPageLayout>
			<template #sidebar>
				<SettingsSidebar />
			</template>
			<slot />
		</SidebarPageLayout>
	</div>
</template>

<style scoped>
.project-settings :deep(.project-field-highlight-anchor) {
	position: relative;
}

.project-settings :deep(.project-field-highlight-outline) {
	@apply pointer-events-none absolute rounded-2xl border-2 border-solid border-orange;
	inset: -12px;
}

.project-settings :deep(.settings-section-title) {
	margin: 0 0 12px;
	@apply text-2xl font-semibold text-contrast;
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
</style>
