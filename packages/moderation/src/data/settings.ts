import type { EnumSettingDefinition, ToggleSettingDefinition } from '../types/settings.ts'

const settings = {
	General: {
		ChecklistPosition: {
			type: 'enum',
			id: 'checklist-position',
			title: 'Checklist Position',
			description: 'Where the checklist should be displayed on the page',
			entries: [
				{ value: 'left', label: 'Left' },
				{ value: 'right', label: 'Right' },
			],
			default: 'right',
		} as EnumSettingDefinition,
		ProjectKeybinds: {
			type: 'toggle',
			id: 'project-keybinds',
			title: 'Enable Project Keybinds',
			description: 'Weather certain keybinds should work without the checklist visible.',
			default: false,
		} as ToggleSettingDefinition,
		PrivateMessageHighlight: {
			type: 'toggle',
			id: 'private-message-highlight',
			title: 'Highlight Private Messages',
			description: 'Whether private messages should be highlighted in the chat.',
			default: true,
		} as ToggleSettingDefinition,
		SlicerButtonInVersions: {
			type: 'toggle',
			id: 'slicer-button-in-versions',
			title: 'Show Slicer Button in Versions Table and Page',
			description: 'Whether the slicer button should be shown in the versions table and page.',
			default: false,
		} as ToggleSettingDefinition,
	},
} as const

export default settings
