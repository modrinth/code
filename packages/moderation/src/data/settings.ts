import { setting } from '../types/settings.ts'

const settings = {
	General: {
		ChecklistPosition: setting.asEnum({
			type: 'enum',
			id: 'checklist-position',
			title: 'Checklist Position',
			description: 'Where the checklist should be displayed on the page',
			entries: [
				{ value: 'left', label: 'Left' },
				{ value: 'right', label: 'Right' },
			],
			default: 'right',
		}),
		ProjectKeybinds: setting.asToggle({
			type: 'toggle',
			id: 'project-keybinds',
			title: 'Enable Project Keybinds',
			description: 'Weather certain keybinds should work without the checklist visible.',
			default: false,
		}),
		PrivateMessageHighlight: setting.asToggle({
			type: 'toggle',
			id: 'private-message-highlight',
			title: 'Highlight Private Messages',
			description: 'Whether private messages should be highlighted in the chat.',
			default: true,
		}),
		SlicerButtonInVersions: setting.asToggle({
			type: 'toggle',
			id: 'slicer-button-in-versions',
			title: 'Show Slicer Button in Versions Table and Page',
			description: 'Whether the slicer button should be shown in the versions table and page.',
			default: false,
		}),
		AdjustPageAlignment: setting.asEnum({
			type: 'enum',
			id: 'adjust-page-alignment',
			title: 'Adjust Page Alignment',
			description:
				'Whether the main page elements should be centered or offset opposite to the Checklist Position.',
			entries: [
				{ value: 'never', label: 'Never' },
				{ value: 'checklist-present', label: 'Checklist Needed' },
				{ value: 'always', label: 'Always' },
			],
			default: 'never',
		}),
	},
} as const

export default settings
