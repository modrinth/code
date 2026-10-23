<template>
	<SettingsPages
		:pages="settingsPages"
		:store="moderationSettings"
		storage-key="moderation.project-review.settings-page"
	>
		<template v-if="!floating" #actions>
			<IconButton
				v-tooltip="'Open as floating window'"
				type="quiet"
				size="sm"
				label="Open as floating window"
				@click="settingsAsWindow"
			>
				<WindowIcon aria-hidden="true" />
			</IconButton>
		</template>
		<template #setting-actions="{ setting }">
			<ModerationSettingActions :setting="setting" :store="moderationSettings" />
		</template>
		<template v-if="!isLocal" #footer>
			<p class="m-0 shrink-0 break-words text-xs text-secondary">
				Based on
				<a
					v-if="config.public.owner && config.public.branch"
					class="hover:underline"
					target="_blank"
					rel="noopener"
					:href="`https://github.com/${config.public.owner}/code/tree/${config.public.branch}`"
				>
					{{ config.public.owner }}/{{ config.public.branch }}
				</a>
				@
				<span v-if="config.public.hash === 'unknown'">unknown</span>
				<AutoLink
					v-else
					class="text-link"
					target="_blank"
					:to="`https://github.com/${config.public.owner}/code/commit/${config.public.hash}`"
				>
					{{ config.public.hash }}
				</AutoLink>
			</p>
		</template>
	</SettingsPages>
</template>

<script setup lang="ts">
import {
	FolderArchiveIcon,
	KeyboardIcon,
	LayoutPanelLeftIcon,
	PaletteIcon,
	SettingsIcon,
	WindowIcon,
} from '@modrinth/assets'
import {
	archiveSettingGroups,
	AutoLink,
	IconButton,
	injectArchiveSettings,
	injectWorkbench,
	type SettingsPage,
	SettingsPages,
	useFilesWorkspaceSettings,
	workbenchSettingGroup,
	workbenchSettingsStore,
} from '@modrinth/ui'

import { moderationSettingGroups } from '~/components/ui/moderation/settings/groups'
import ModerationAppearance from '~/components/ui/moderation/settings/ModerationAppearance.vue'
import ModerationKeybinds from '~/components/ui/moderation/settings/ModerationKeybinds.vue'
import ModerationSettingActions from '~/components/ui/moderation/settings/ModerationSettingActions.vue'
import { useModerationSettings } from '~/composables/moderation'

import { injectProjectReviewContext } from './layout/context'

/**
 * The review's settings, along with those of its archive and layout, so there's a single place to
 * change them while watching them apply: in a panel of their own, or `floating` in a window above
 * the review.
 */
defineProps<{ floating?: boolean }>()

const { settingsAsWindow } = injectProjectReviewContext()
const config = useRuntimeConfig()
const { hostname } = useRequestURL()
const isLocal =
	import.meta.dev ||
	hostname === 'localhost' ||
	hostname.endsWith('.localhost') ||
	hostname.startsWith('127.') ||
	hostname === '[::1]'

const moderationSettings = useModerationSettings()
const workbench = injectWorkbench()
const archiveStore = injectArchiveSettings()
/** The files settings as the review browses files, whichever way its archive is laid out. */
const filesSettings = useFilesWorkspaceSettings('project-review', { browsing: 'sidebar' })

const settingsPages: SettingsPage[] = [
	{
		id: 'general',
		name: {
			id: 'moderation.project-review.generalSettings',
			defaultMessage: 'General',
		},
		icon: SettingsIcon,
		groups: moderationSettingGroups,
	},
	{
		id: 'appearance',
		name: {
			id: 'moderation.project-review.appearanceSettings',
			defaultMessage: 'Appearance',
		},
		icon: PaletteIcon,
		content: ModerationAppearance,
		contentProps: { embedded: true },
	},
	{
		id: 'archive',
		name: {
			id: 'moderation.project-review.archive-settings',
			defaultMessage: 'Archive',
		},
		icon: FolderArchiveIcon,
		groups: archiveSettingGroups.map((group) => ({ ...group, store: archiveStore })),
	},
	{
		id: 'layout',
		name: {
			id: 'moderation.project-review.layout-settings',
			defaultMessage: 'Layout and files',
		},
		icon: LayoutPanelLeftIcon,
		groups: [
			{ ...workbenchSettingGroup(workbench), store: workbenchSettingsStore(workbench) },
			...filesSettings.groups.map((group) => ({ ...group, store: filesSettings.store })),
		],
	},
	{
		id: 'keybinds',
		name: {
			id: 'moderation.project-review.keybindsSettings',
			defaultMessage: 'Keybinds',
		},
		icon: KeyboardIcon,
		content: ModerationKeybinds,
		contentProps: { embedded: true },
	},
]
</script>
