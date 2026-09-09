import { setting } from '../types/settings.ts'

const settings = {
	General: {
		ChecklistPosition: setting.asEnum({
			type: 'enum',
			id: 'checklist-position',
			title: 'Checklist position',
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
			title: 'Enable project keybinds',
			description: 'Weather certain keybinds should work without the checklist visible.',
			default: false,
		}),
		PrivateMessageHighlight: setting.asToggle({
			type: 'toggle',
			id: 'private-message-highlight',
			title: 'Highlight private messages',
			description: 'Whether private messages should be highlighted in the chat.',
			default: true,
		}),
		SlicerButtonInVersions: setting.asToggle({
			type: 'toggle',
			id: 'slicer-button-in-versions',
			title: 'Show Slicer button in versions table and page',
			description: 'Whether the slicer button should be shown in the versions table and page.',
			default: false,
		}),
		AdjustPageAlignment: setting.asEnum({
			type: 'enum',
			id: 'adjust-page-alignment',
			title: 'Adjust page alignment',
			description:
				'Whether the main page elements should be centered or offset opposite to the Checklist Position.',
			entries: [
				{ value: 'never', label: 'Never' },
				{ value: 'checklist-present', label: 'Checklist Needed' },
				{ value: 'always', label: 'Always' },
			],
			default: 'never',
		}),
		AlternativeHostname: setting.asString({
			type: 'string',
			id: 'alternative-hostname',
			title: 'Alternative hostname',
			description:
				'When Open production/staging is used on an official host, open this hostname instead. Example: localhost:3000',
			default: null,
		}),
	},
} as const

export default settings
