import type {
	EnumSettingDefinition,
	StringSettingDefinition,
	ToggleSettingDefinition
} from "../types/settings.ts";

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
	}
} as const

export default settings
