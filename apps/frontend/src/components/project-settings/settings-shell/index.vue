<script setup lang="ts">
import { injectProjectPageContext } from '@modrinth/ui'
import { ref, useTemplateRef } from 'vue'

import ProjectModerationDrawer from '~/components/ui/project-settings/project-moderation-drawer/index.vue'
import SidebarPageLayout from '~/components/ui/SidebarPageLayout.vue'
import { provideProjectSettingsNavigation } from '~/composables/project-settings-navigation'

import SettingsSidebar from './sidebar.vue'

const { currentMember, projectV3 } = injectProjectPageContext()
const settingsRoot = ref<HTMLElement | null>(null)
const moderationDrawer = useTemplateRef('moderationDrawer')
provideProjectSettingsNavigation(settingsRoot, () => moderationDrawer.value?.show('moderation'))

const drawerPinned = ref(false)
</script>

<template>
	<div
		ref="settingsRoot"
		class="project-settings mb-8 flex w-full flex-col gap-4"
		:class="{ 'settings-with-review': drawerPinned }"
	>
		<ProjectModerationDrawer
			v-if="currentMember && projectV3"
			ref="moderationDrawer"
			@pinned="drawerPinned = $event"
		/>
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
	@apply relative;
}

.project-settings :deep(.project-field-highlight-outline) {
	@apply pointer-events-none absolute -inset-3 rounded-2xl border-2 border-solid border-orange;
}

.project-settings :deep(.settings-section-title) {
	@apply text-2xl font-semibold text-contrast;
}

.project-settings :deep(.settings-content-card) {
	@apply mt-0 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-6;
}

@media (max-width: 640px) {
	.project-settings :deep(.settings-content-card) {
		@apply p-4;
	}
}
</style>
