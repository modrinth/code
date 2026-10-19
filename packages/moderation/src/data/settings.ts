import { setting } from '../types/settings.ts'

const settings = {
	General: {
		SendIssuesAsThreadMessage: setting.asToggle({
			type: 'toggle',
			id: 'send-issues-as-thread-message',
			title: 'Send issues as thread message',
			description: 'Send selected issues as a thread message instead of creating tracked issues.',
			default: true,
		}),
		AutoGoNextOnReviewOutcome: setting.asToggle({
			type: 'toggle',
			id: 'auto-go-next-on-review-outcome',
			title: 'Automatically go to the next project after review',
			description:
				'Move to the next project in the queue after successfully approving, withholding, or rejecting a project.',
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
			description: 'Show keyboard shortcut hints on review tabs.',
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
			description:
				'Add a background highlight to inline review panels when hovering over or interacting with them.',
			default: true,
		}),
		LinkInfoPlacement: setting.asEnum({
			type: 'enum',
			id: 'link-info-placement',
			title: 'Link info panel position',
			description: 'Prefer showing link info above or below the link in project review.',
			default: 'above',
			entries: [
				{ label: 'Above', value: 'above' },
				{ label: 'Below', value: 'below' },
			],
		}),
		PrivateMessageHighlight: setting.asToggle({
			type: 'toggle',
			id: 'private-message-highlight',
			title: 'Highlight private messages and issue history',
			description: 'Add a background highlight to private messages and issue history in threads.',
			default: true,
		}),
		SlicerButtonInVersions: setting.asToggle({
			type: 'toggle',
			id: 'slicer-button-in-versions',
			title: 'Show Slicer button in versions table and page',
			description: 'Show the Slicer button on public version lists and version pages for staff.',
			default: false,
		}),
		AlternativeHostname: setting.asString({
			type: 'string',
			id: 'alternative-hostname',
			title: 'Alternative hostname',
			description:
				'Use this host for the "Open current page" shortcut when on production or staging. Enter a hostname or HTTP(S) URL, such as localhost:3000.',
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
