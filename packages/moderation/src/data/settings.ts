import { setting } from '../types/settings.ts'

const settings = {
	General: {
		AutoGoNextOnReviewOutcome: setting.asToggle({
			type: 'toggle',
			id: 'auto-go-next-on-review-outcome',
			title: 'Automatically go to the next project after review',
			description: 'Move to the next project after successfully submitting a review outcome.',
			default: true,
		}),
		HideMarkdownFormattingButtons: setting.asToggle({
			type: 'toggle',
			id: 'hide-markdown-formatting-buttons',
			title: 'Hide Markdown formatting buttons',
			description: 'Hide the formatting buttons above all Markdown editors in project review.',
			default: true,
		}),
		ShowShortcutKeybindHints: setting.asToggle({
			type: 'toggle',
			id: 'show-shortcut-keybind-hints',
			title: 'Show shortcut keybind hints',
			description: 'Show keyboard shortcut hints on review tabs, Reply, and Private note.',
			default: true,
		}),
		ShowToggleIssueButtonShortcutHint: setting.asToggle({
			type: 'toggle',
			id: 'show-toggle-issue-button-shortcut-hint',
			title: 'Show toggle issue button shortcut hints',
			description: 'Show number key shortcut hints on issue toggle buttons.',
			default: true,
		}),
		ShowFloatingPanelFieldHoverHighlight: setting.asToggle({
			type: 'toggle',
			id: 'show-floating-panel-field-hover-highlight',
			title: 'Highlight fields for floating review panels',
			description:
				'Highlight the reviewed field when hovering over it or its floating review panel.',
			default: true,
		}),
		ShowInlinePanelHoverHighlight: setting.asToggle({
			type: 'toggle',
			id: 'show-inline-panel-hover-highlight',
			title: 'Highlight inline review panels',
			description: 'Highlight inline review panels when hovering over or interacting with them.',
			default: true,
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
		AlternativeHostname: setting.asString({
			type: 'string',
			id: 'alternative-hostname',
			title: 'Alternative hostname',
			description:
				'When Open production/staging is used on an official host, open this hostname instead. Example: localhost:3000',
			default: '',
		}),
		EnableProjectKeybinds: setting.asToggle({
			type: 'toggle',
			id: 'checklist-always-allow-keybinds',
			title: 'Enable project page shortcuts',
			description: 'Use moderation copy shortcuts on public project pages.',
			default: false,
		}),
		IncludeNonProcessingProjects: setting.asToggle({
			type: 'toggle',
			id: 'checklist-ignore-conditions',
			title: 'Include projects no longer awaiting review',
			description: 'Allow queue navigation to include projects that are no longer awaiting review.',
			default: false,
		}),
	},
} as const

export default settings
