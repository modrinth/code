import type { Labrinth } from '@modrinth/api-client'
import type { FunctionalComponent, SVGAttributes } from 'vue'

import type { ModerationStatus } from '../../../types/node/state'
import type { IssuePriority } from './priority'

export interface ReviewContext {
	projectV3: Labrinth.Projects.v3.Project
	projectV2: Labrinth.Projects.v2.Project | undefined
	disclosures: readonly Labrinth.Projects.v3.ProjectDisclosureData[]
	members: readonly Labrinth.Projects.v3.TeamMember[]
	organization: Labrinth.Organizations.v3.Organization | null
	organizationMembers: readonly Labrinth.Projects.v3.TeamMember[]
	wasReviewed: boolean
	permissions: {
		groups: readonly Labrinth.Attribution.Internal.AttributionGroup[]
		unresolvedCount: number
		loaded: boolean
		loading: boolean
		error: unknown
	}
	selected: {
		/** Issue IDs selected directly or through explicit toggle IDs across the current project. */
		issueIds: readonly string[]
		/** Selected toggle IDs for the current issue; panel and section callbacks receive all selected toggle IDs. */
		toggleIds: readonly string[]
		/** Message and facet callbacks receive selected items for the current issue, grouped by their registry key. */
		items: Readonly<Record<string, readonly { key: string; context: ReviewContext }[]>>
	}
	/**
	 * Returns a markdown value for the current issue, or an empty string if unavailable.
	 * Panel and section callbacks must supply an issue ID to read stored input values.
	 * Only visible, enabled controls contribute when resolving messages.
	 * UI callbacks read stored values; message callbacks also resolve defaults.
	 */
	getMarkdownValue(id: string, issueId?: string): string
	getTextValue(id: string, issueId?: string): string
	getSelectValue(id: string, issueId?: string): string
	getSelectValues(id: string, issueId?: string): readonly string[]
}

export type WithContext<T> = T | ((ctx: ReviewContext) => T)

export type IssueFacet = (ctx: ReviewContext) => Labrinth.Threads.v3.ThreadIssueTarget

export interface Issue {
	id: string
	title: string
	category: string
	/** Orders selected issues; omitted priorities use the default group. */
	priority?: IssuePriority | number
	message: WithContext<string>
	/** Targets to change, resolved from current context. Omitted or empty facets require checkbox acknowledgment. */
	facets?: WithContext<readonly IssueFacet[]>
	/** Additional navigation destinations and field placement, independent of facet completion. Locations sharing a field with a facet reuse its button; a custom location label overrides the button label. */
	locations?: WithContext<readonly Labrinth.Threads.v3.ThreadIssueLocation[]>
	suggestedStatus?: WithContext<ModerationStatus | undefined>
}

export type IssueConfig = Issue

interface IssueControlOptions {
	issue: Issue
	label: WithContext<string>
	shown?: WithContext<boolean>
	tooltip?: WithContext<string>
	disabled?: WithContext<boolean>
}

export interface IssueToggle extends IssueControlOptions {
	type: 'toggle'
	/** Optional label for this toggle in the issue list. */
	issueListLabel?: WithContext<string>
	/** Optional issue-list row for this toggle. Defaults to the toggle label. */
	issueListGroup?: WithContext<string>
	/**
	 * Identifies a selection within its issue. Reuse the ID across panels to share selection state.
	 * Without an ID, the toggle activates the issue directly and contributes no selected toggle ID.
	 */
	id?: string
}

export type IssueToggleConfig = Omit<IssueToggle, 'type'>

export interface IssueMarkdown extends IssueControlOptions {
	type: 'markdown'
	id: string
	required?: WithContext<boolean>
}

export type IssueMarkdownConfig = Omit<IssueMarkdown, 'type'>

export interface IssueText extends IssueControlOptions {
	type: 'text'
	id: string
	required?: WithContext<boolean>
	placeholder?: WithContext<string>
	initial?: WithContext<string>
}

export type IssueTextConfig = Omit<IssueText, 'type'>

export interface IssueTextarea extends IssueControlOptions {
	type: 'textarea'
	id: string
	required?: WithContext<boolean>
	placeholder?: WithContext<string>
	initial?: WithContext<string>
	rows?: number
	maxlength?: number
}

export type IssueTextareaConfig = Omit<IssueTextarea, 'type'>

export interface IssueSelectOption {
	value: string
	label: WithContext<string>
	shown?: WithContext<boolean>
	disabled?: WithContext<boolean>
}

export interface IssueSelect extends IssueControlOptions {
	type: 'select'
	id: string
	options: WithContext<readonly IssueSelectOption[]>
	multiple?: boolean
	required?: WithContext<boolean>
	placeholder?: WithContext<string>
	initial?: WithContext<string | readonly string[]>
}

export type IssueSelectConfig = Omit<IssueSelect, 'type'>

export type IssueControl = IssueToggle | IssueMarkdown | IssueText | IssueTextarea | IssueSelect

export interface ReviewPanelItem {
	key: string
	panel: Panel
}

export interface Panel {
	icon: FunctionalComponent<SVGAttributes>
	/** Parent panel whose matching controls apply to this item. */
	parent?: string
	shown?: WithContext<boolean>
	title?: WithContext<string>
	hint: WithContext<string>
	guidanceUrl: string
	children: PanelNode[]
}

/** Registers keyed panels returned by a callback. */
export interface EachPanel {
	type: 'each'
	items: (ctx: ReviewContext) => readonly ReviewPanelItem[]
}

export type PanelRegistration = Panel | EachPanel

export type PanelConfig = Omit<Panel, 'children'>

/**
 * Only panels and sections support `.content()`; toggles and markdown controls are leaves.
 * When migrating other stages, move nested control content into a section and use `shown`
 * to check `selected.issueIds` for a toggle without an ID, or `selected.toggleIds` for an explicit ID.
 * Preserve ancestor visibility by nesting conditional sections or checking every parent toggle
 * in a sibling section's `shown` callback. Keep existing issue IDs, toggle IDs, and markdown keys.
 * See description.ts for an example of migrating toggle content into conditional sections.
 */
export interface PanelSection {
	type: 'section'
	label?: WithContext<string>
	shown?: WithContext<boolean>
	children: PanelNode[]
}

export type PanelSectionConfig = Omit<PanelSection, 'type' | 'children'>

export type PanelNode = PanelSection | IssueControl
