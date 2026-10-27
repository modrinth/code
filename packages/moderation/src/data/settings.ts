import { setting } from '../types/settings.ts'

const settings = {
	General: {
		GalleryColumns: setting.asEnum({
			type: 'enum',
			id: 'gallery-columns',
			title: 'Gallery columns',
			description:
				'Number of gallery columns to display in project review, regardless of panel width.',
			default: '3',
			entries: [
				{ label: '1', value: '1' },
				{ label: '2', value: '2' },
				{ label: '3', value: '3' },
				{ label: '4', value: '4' },
				{ label: '5', value: '5' },
				{ label: '6', value: '6' },
			],
		}),
		ShowAllIssueControls: setting.asToggle({
			type: 'toggle',
			id: 'show-all-issue-controls',
			title: 'Always show issue controls',
			description:
				'Show all issue controls in the issues menu regardless of visibility conditions.',
			default: false,
		}),
		SendIssuesAsThreadMessage: setting.asToggle({
			type: 'toggle',
			id: 'send-issues-as-thread-message',
			title: 'Send issues as thread message',
			description: 'Send selected issues as a thread message instead of creating tracked issues.',
			default: false,
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
		EnableDelayedHoverMessagePreviews: setting.asToggle({
			type: 'toggle',
			id: 'enable-delayed-hover-message-previews',
			title: 'Enable delayed hover message previews',
			description: 'Show issue card previews when hovering over issue toggle buttons.',
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
			title: 'Highlight review sections',
			description:
				'Highlight the review section when hovering over it or its floating review panel.',
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
		ReviewHighlightColor: setting.asColor({
			type: 'color',
			id: 'review-highlight-color',
			title: 'Color',
			description: 'Color for reviewed fields, inline review panels, and navigation flashes.',
			default: '',
		}),
		ReviewHighlightOpacity: setting.asNumber({
			type: 'number',
			id: 'review-highlight-opacity',
			title: 'Opacity',
			description:
				'Background opacity for review highlights. Navigation flashes use twice this opacity, up to 100%.',
			default: 3,
			min: 0,
			max: 100,
			step: 1,
			unit: '%',
		}),
		IssueToggleColor: setting.asColor({
			type: 'color',
			id: 'issue-toggle-color',
			title: 'Selected color',
			description:
				'Color for selected and partially selected issue buttons, with a 25% background tint.',
			default: '',
		}),
		FloatingReviewPanelBorderColor: setting.asColor({
			type: 'color',
			id: 'floating-review-panel-border-color',
			title: 'Border color',
			description: 'Border color for floating review panels.',
			default: '',
		}),
		FloatingReviewPanelBackground: setting.asColor({
			type: 'color',
			id: 'floating-review-panel-background',
			title: 'Background',
			description: 'Background tint color for floating review panels.',
			default: '',
		}),
		FloatingReviewPanelBackgroundOpacity: setting.asNumber({
			type: 'number',
			id: 'floating-review-panel-background-opacity',
			title: 'Opacity',
			description: 'Background tint opacity for floating review panels.',
			default: 3,
			min: 0,
			max: 100,
			step: 1,
			unit: '%',
		}),
		FloatingReviewPanelBorderWidth: setting.asNumber({
			type: 'number',
			id: 'floating-review-panel-border-width',
			title: 'Border width',
			description: 'Border width for floating review panels. Set to 0 to hide the border.',
			default: 1,
			min: 0,
			max: 8,
			step: 1,
			unit: 'px',
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
		ExpandVersionDetailsByDefault: setting.asToggle({
			type: 'toggle',
			id: 'expand-version-details-by-default',
			title: 'Expand version details by default',
			description: 'Expand the latest version’s Details section automatically in project review.',
			default: true,
		}),
		ExpandVersionDependenciesByDefault: setting.asToggle({
			type: 'toggle',
			id: 'expand-version-dependencies-by-default',
			title: 'Expand version dependencies by default',
			description:
				'Expand dependencies automatically when viewing version details in project review.',
			default: true,
		}),
		ExpandVersionChangelogByDefault: setting.asToggle({
			type: 'toggle',
			id: 'expand-version-changelog-by-default',
			title: 'Expand version changelog by default',
			description:
				'Expand the changelog automatically when viewing version details in project review.',
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
