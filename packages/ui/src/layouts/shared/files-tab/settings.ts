import { defineSetting, type SettingGroup, useLocalSettings } from '#ui/components/settings'
import { defineMessages } from '#ui/composables/i18n'

/** How formatted JSON is indented: with tabs, or with the given number of spaces. */
export type FilesJsonIndent = 'tab' | '2' | '4'

/** The indentation formatted JSON uses, as `JSON.stringify` takes it. */
export function jsonIndentOf(indent: FilesJsonIndent) {
	return indent === 'tab' ? '\t' : Number(indent)
}

/** Where the folder tree's button sits: atop the tree, or in the navigation of each tab. */
export type FilesSidebarToggleLocation = 'sidebar' | 'navigation'

const messages = defineMessages({
	group: { id: 'files.settings.group', defaultMessage: 'Files' },
	groupDescription: {
		id: 'files.settings.group-description',
		defaultMessage: 'How files and folders are listed and browsed.',
	},
	coloredIcons: { id: 'files.settings.colored-icons', defaultMessage: 'Colored file icons' },
	coloredIconsDescription: {
		id: 'files.settings.colored-icons-description',
		defaultMessage: 'Tint file and folder icons by their type.',
	},
	compactFolders: { id: 'files.settings.compact-folders', defaultMessage: 'Compact folders' },
	compactFoldersDescription: {
		id: 'files.settings.compact-folders-description',
		defaultMessage:
			'Fold chains of folders that each hold just one folder into a single row of the tree.',
	},
	newTabButton: { id: 'files.settings.new-tab-button', defaultMessage: 'New tab button' },
	newTabButtonDescription: {
		id: 'files.settings.new-tab-button-description',
		defaultMessage: 'Show a button for opening a new tab beside the file tabs.',
	},
	formatJson: { id: 'files.settings.format-json', defaultMessage: 'Format JSON files' },
	formatJsonDescription: {
		id: 'files.settings.format-json-description',
		defaultMessage:
			"Open JSON files that can't be edited laid out over lines and indented, even if they're stored compacted.",
	},
	jsonIndent: { id: 'files.settings.json-indent', defaultMessage: 'JSON indentation' },
	jsonIndentDescription: {
		id: 'files.settings.json-indent-description',
		defaultMessage: 'How formatted JSON is indented.',
	},
	jsonIndentTab: { id: 'files.settings.json-indent.tab', defaultMessage: 'Tabs' },
	jsonIndentTwo: { id: 'files.settings.json-indent.two', defaultMessage: '2 spaces' },
	jsonIndentFour: { id: 'files.settings.json-indent.four', defaultMessage: '4 spaces' },
	sidebarToggle: { id: 'files.settings.sidebar-toggle', defaultMessage: 'Folder tree button' },
	sidebarToggleSidebar: {
		id: 'files.settings.sidebar-toggle.sidebar',
		defaultMessage: 'In the folder tree',
	},
	sidebarToggleNavigation: {
		id: 'files.settings.sidebar-toggle.navigation',
		defaultMessage: 'In the navigation bar',
	},
})

/**
 * Settings of the files layout, read by it and offered by hosts' settings (e.g. the archive
 * viewer's). The folder tree button's placement is picked from the files' own view settings
 * instead, as hosts with a layout of their own (e.g. a workbench) toggle the tree themselves.
 */
export const filesTabSettings = {
	coloredIcons: defineSetting.toggle({
		id: 'colored-icons',
		storageKey: 'files-colored-icons',
		title: messages.coloredIcons,
		description: messages.coloredIconsDescription,
		default: true,
	}),
	compactFolders: defineSetting.toggle({
		id: 'compact-folders',
		storageKey: 'files-compact-folders',
		title: messages.compactFolders,
		description: messages.compactFoldersDescription,
		default: false,
	}),
	newTabButton: defineSetting.toggle({
		id: 'new-tab-button',
		title: messages.newTabButton,
		description: messages.newTabButtonDescription,
		default: true,
	}),
	formatJson: defineSetting.toggle({
		id: 'format-json',
		title: messages.formatJson,
		description: messages.formatJsonDescription,
		default: true,
	}),
	jsonIndent: defineSetting.enum<FilesJsonIndent>({
		id: 'json-indent',
		title: messages.jsonIndent,
		description: messages.jsonIndentDescription,
		default: 'tab',
		display: 'chips',
		entries: [
			{ value: 'tab', label: messages.jsonIndentTab },
			{ value: '2', label: messages.jsonIndentTwo },
			{ value: '4', label: messages.jsonIndentFour },
		],
	}),
	sidebarToggle: defineSetting.enum<FilesSidebarToggleLocation>({
		id: 'sidebar-toggle-placement',
		title: messages.sidebarToggle,
		default: 'sidebar',
		entries: [
			{ value: 'sidebar', label: messages.sidebarToggleSidebar },
			{ value: 'navigation', label: messages.sidebarToggleNavigation },
		],
	}),
}

export const filesTabSettingGroups: SettingGroup[] = [
	{
		id: 'files',
		title: messages.group,
		description: messages.groupDescription,
		settings: [
			filesTabSettings.coloredIcons,
			filesTabSettings.compactFolders,
			filesTabSettings.newTabButton,
			filesTabSettings.formatJson,
			filesTabSettings.jsonIndent,
		],
	},
]

/** The files layout's settings, kept in local storage. */
export function useFilesTabSettings() {
	return useLocalSettings(Object.values(filesTabSettings), {
		prefix: 'files-setting:',
		initOnMounted: true,
	})
}
