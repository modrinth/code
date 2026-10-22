import { moderationSettings } from '@modrinth/moderation'
import { defineMessages, type SettingGroup } from '@modrinth/ui'

const messages = defineMessages({
	workflow: {
		id: 'moderation.settings.workflow',
		defaultMessage: 'Review workflow and queue',
	},
	panels: {
		id: 'moderation.settings.panels',
		defaultMessage: 'Panels',
	},
	appearance: {
		id: 'moderation.settings.appearance',
		defaultMessage: 'Appearance',
	},
	shortcuts: {
		id: 'moderation.settings.shortcuts',
		defaultMessage: 'Editors and shortcut hints',
	},
	projectPages: {
		id: 'moderation.settings.project-pages',
		defaultMessage: 'Public pages and hosts',
	},
	debug: {
		id: 'moderation.settings.debug',
		defaultMessage: 'Debug settings',
	},
})

const general = moderationSettings.General

/** The moderation settings by heading, for the settings page and the review's settings modal. */
export const moderationSettingGroups: SettingGroup[] = [
	{
		id: 'workflow',
		title: messages.workflow,
		settings: [
			general.SendIssuesAsThreadMessage,
			general.AutoGoNextOnReviewOutcome,
			general.IncludeNonProcessingProjects,
		],
	},
	{
		id: 'panels',
		title: messages.panels,
		settings: [general.LinkInfoPlacement, general.ArchiveIntegration, general.MergeRulings],
	},
	{
		id: 'appearance',
		title: messages.appearance,
		settings: [
			general.GalleryColumns,
			general.ExpandVersionDetailsByDefault,
			general.ExpandVersionDependenciesByDefault,
			general.ExpandVersionChangelogByDefault,
			general.EnableDelayedHoverMessagePreviews,
			general.ButtonSize,
			general.PrivateMessageHighlight,
			general.SectionHeadersVisible,
			general.SectionReducedGap
		],
	},
	{
		id: 'shortcuts',
		title: messages.shortcuts,
		settings: [
			general.MarkdownFormattingButtonsVisibility,
			general.ShowShortcutKeybindHints,
			general.ShowToggleIssueButtonShortcutHint,
		],
	},
	{
		id: 'project-pages',
		title: messages.projectPages,
		settings: [
			general.EnableProjectKeybinds,
			general.ArchiveButtonInVersions,
			general.AlternativeHostname,
		],
	},
	{
		id: 'debug',
		title: messages.debug,
		settings: [general.ShowAllIssueControls],
	},
]
