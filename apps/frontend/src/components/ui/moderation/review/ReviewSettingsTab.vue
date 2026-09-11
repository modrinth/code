<template>
	<div class="flex min-h-0 flex-col">
		<div
			class="mb-3 flex shrink-0 flex-wrap gap-1 border-0 border-b border-solid border-divider pb-3"
		>
			<button
				v-for="section in sections"
				:key="section.id"
				class="rounded-full px-3 py-1 text-sm"
				:class="
					section.id === activeId
						? 'bg-button-bg font-semibold text-contrast'
						: 'text-secondary hover:bg-button-bg hover:text-contrast'
				"
				@click="activeId = section.id"
			>
				{{ section.label }}
			</button>
		</div>

		<div class="min-h-0 flex-1 overflow-y-auto">
			<Suspense>
				<component :is="currentComponent" :key="activeId" />
				<template #fallback>
					<div class="flex items-center justify-center gap-2 py-12 text-secondary">
						<SpinnerIcon class="size-5 animate-spin" /> Loading settings…
					</div>
				</template>
			</Suspense>
		</div>
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { type Component, computed, defineAsyncComponent } from 'vue'

import { useModerationReviewLayout } from '~/services/moderation/review-layout'

const { projectV3, currentMember } = injectProjectPageContext()
const layout = useModerationReviewLayout()

interface SettingsSection {
	id: string
	label: string
	component: Component
	hideForServer?: boolean
	serverOnly?: boolean
	envOnly?: boolean
}

/**
 * Tags / License / Links / Disclosures / Permissions were promoted to top-level review tabs
 * (they carry checklist stages); this tab keeps the sections that don't.
 */
const ALL_SECTIONS: SettingsSection[] = [
	{
		id: 'general',
		label: 'General',
		component: defineAsyncComponent(() => import('~/pages/[type]/[project]/settings/index.vue')),
	},
	{
		id: 'description',
		label: 'Description',
		component: defineAsyncComponent(
			() => import('~/pages/[type]/[project]/settings/description.vue'),
		),
		hideForServer: true,
	},
	{
		id: 'environment',
		label: 'Environment',
		component: defineAsyncComponent(
			() => import('~/pages/[type]/[project]/settings/environment.vue'),
		),
		envOnly: true,
	},
	{
		id: 'members',
		label: 'Members',
		component: defineAsyncComponent(() => import('~/pages/[type]/[project]/settings/members.vue')),
	},
	{
		id: 'gallery',
		label: 'Gallery',
		component: defineAsyncComponent(() => import('~/pages/[type]/[project]/settings/gallery.vue')),
	},
	{
		id: 'server',
		label: 'Server',
		component: defineAsyncComponent(() => import('~/pages/[type]/[project]/settings/server.vue')),
		serverOnly: true,
	},
]

const isServer = computed(() => projectV3.value?.minecraft_server != null)
const projectTypes = computed(() => projectV3.value?.project_types ?? [])
const showEnv = computed(
	() =>
		projectTypes.value.some((t) => ['mod', 'modpack'].includes(t)) &&
		isStaff(currentMember.value?.user),
)

const sections = computed(() =>
	ALL_SECTIONS.filter((s) => {
		if (s.hideForServer && isServer.value) return false
		if (s.serverOnly && !isServer.value) return false
		if (s.envOnly && !showEnv.value) return false
		return true
	}),
)

const activeId = computed<string>({
	get: () =>
		sections.value.some((s) => s.id === layout.settingsSection.value)
			? layout.settingsSection.value
			: 'general',
	set: (value) => layout.setSettingsSection(value),
})

const currentComponent = computed<Component>(
	() => (sections.value.find((s) => s.id === activeId.value) ?? sections.value[0]).component,
)
</script>
