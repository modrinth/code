import { setting } from '../types/settings.ts'

const settings = {
	General: {
		ChecklistPosition: setting.asEnum({
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
			id: 'project-keybinds',
			title: 'Enable project keybinds',
			description: 'Weather certain keybinds should work without the checklist visible.',
			default: false,
		}),
		InlineChecklistMenu: setting.asToggle({
			type: 'toggle',
			id: 'inline-checklist-menu',
			title: 'Right-click checklist menu',
			description:
				'In the review view, right-click a section to open its checklist buttons in a context menu.',
			default: true,
		}),
		ShowChecklistWalkthrough: setting.asToggle({
			type: 'toggle',
			id: 'show-checklist-walkthrough',
			title: 'Show the checklist walkthrough',
			description:
				'In the review view, show the collapsible stage-by-stage walkthrough helper at the bottom.',
			default: true,
		}),
		ShowFloatingChecklistInReview: setting.asToggle({
			type: 'toggle',
			id: 'show-floating-checklist-in-review',
			title: 'Keep the floating checklist in the review view',
			description:
				'Keep the original floating checklist widget available (collapsed) while the review view is active.',
			default: false,
		}),
		PrivateMessageHighlight: setting.asToggle({
			id: 'private-message-highlight',
			title: 'Highlight private messages',
			description: 'Whether private messages should be highlighted in the chat.',
			default: true,
		}),
		SlicerButtonInVersions: setting.asToggle({
			id: 'slicer-button-in-versions',
			title: 'Show Slicer button in versions table and page',
			description: 'Whether the slicer button should be shown in the versions table and page.',
			default: false,
		}),
		AdjustPageAlignment: setting.asEnum({
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
			id: 'alternative-hostname',
			title: 'Alternative hostname',
			description:
				'When Open production/staging is used on an official host, open this hostname instead. Example: localhost:3000',
			default: '',
		}),
	},
} as const

export default settings
