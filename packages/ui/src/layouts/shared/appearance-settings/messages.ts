import { defineMessages } from '#ui/composables'

export const appearanceSettingsMessages = defineMessages({
	colorThemeTitle: {
		id: 'settings.display.theme.title',
		defaultMessage: 'Color theme',
	},
	colorThemeDescription: {
		id: 'settings.display.theme.description',
		defaultMessage: 'Select your preferred color theme across Modrinth.',
	},
	syncAcrossDevicesTitle: {
		id: 'settings.display.theme.sync-across-devices',
		defaultMessage: 'Sync theme across devices',
	},
	syncAcrossDevicesDescription: {
		id: 'settings.display.theme.sync-across-devices.description',
		defaultMessage:
			"Use this theme everywhere you're signed in. Turn this off to keep a separate theme on this device.",
	},
	syncAcrossDevicesSignedOutTooltip: {
		id: 'settings.display.theme.sync-across-devices.sign-in-tooltip',
		defaultMessage: 'Sign into Modrinth to sync theme',
	},
	projectListLayoutsTitle: {
		id: 'settings.display.project-list-layouts.title',
		defaultMessage: 'Project list layouts',
	},
	projectListLayoutsDescription: {
		id: 'settings.display.project-list-layouts.description',
		defaultMessage: 'Select your preferred layout for each page that displays project lists.',
	},
	mod: {
		id: 'settings.display.project-list-layouts.mod',
		defaultMessage: 'Mods page',
	},
	plugin: {
		id: 'settings.display.project-list-layouts.plugin',
		defaultMessage: 'Plugins page',
	},
	datapack: {
		id: 'settings.display.project-list-layouts.datapack',
		defaultMessage: 'Data Packs page',
	},
	shader: {
		id: 'settings.display.project-list-layouts.shader',
		defaultMessage: 'Shaders page',
	},
	resourcepack: {
		id: 'settings.display.project-list-layouts.resourcepack',
		defaultMessage: 'Resource Packs page',
	},
	modpack: {
		id: 'settings.display.project-list-layouts.modpack',
		defaultMessage: 'Modpacks page',
	},
	server: {
		id: 'settings.display.project-list-layouts.server',
		defaultMessage: 'Servers page',
	},
	user: {
		id: 'settings.display.project-list-layouts.user',
		defaultMessage: 'User profile pages',
	},
	advancedRenderingTitle: {
		id: 'settings.display.sidebar.advanced-rendering.title',
		defaultMessage: 'Advanced rendering',
	},
	advancedRenderingDescription: {
		id: 'settings.display.sidebar.advanced-rendering.description',
		defaultMessage:
			'Enables advanced rendering such as blur effects that may cause performance issues without hardware-accelerated rendering.',
	},
	externalLinksNewTabTitle: {
		id: 'settings.display.sidebar.external-links-new-tab.title',
		defaultMessage: 'Open external links in new tab',
	},
	externalLinksNewTabDescription: {
		id: 'settings.display.sidebar.external-links-new-tab.description',
		defaultMessage:
			'Make links which go outside of Modrinth open in a new tab. No matter this setting, links on the same domain and in Markdown descriptions will open in the same tab, and links on ads and edit pages will open in a new tab.',
	},
	rightAlignedFiltersSidebarTitle: {
		id: 'settings.display.sidebar.right-aligned-filters-sidebar.title',
		defaultMessage: 'Right-aligned filters sidebar on search pages',
	},
	rightAlignedFiltersSidebarDescription: {
		id: 'settings.display.sidebar.right-aligned-filters-sidebar.description',
		defaultMessage: 'Aligns the filters sidebar to the right of the search results.',
	},
	leftAlignedContentSidebarTitle: {
		id: 'settings.display.sidebar.left-aligned-content-sidebar.title',
		defaultMessage: 'Left-aligned sidebar on content pages',
	},
	leftAlignedContentSidebarDescription: {
		id: 'settings.display.sidebar.left-aligned-content-sidebar.description',
		defaultMessage: "Aligns the sidebar to the left of the page's content.",
	},
	nativeDecorationsTitle: {
		id: 'app.appearance-settings.native-decorations.title',
		defaultMessage: 'System window frame',
	},
	nativeDecorationsDescription: {
		id: 'app.appearance-settings.native-decorations.description',
		defaultMessage:
			"Use your operating system's title bar and window controls. Requires an app restart.",
	},
})
