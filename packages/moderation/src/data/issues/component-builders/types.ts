import type { Labrinth } from '@modrinth/api-client'
import type { FunctionalComponent, SVGAttributes } from 'vue'

import type { ModerationStatus } from '../../../types/node/state'

export interface ReviewContext {
	ProjectV3: Labrinth.Projects.v3.Project
	wasReviewed: boolean
	permissions: {
		groups: readonly Labrinth.Attribution.Internal.AttributionGroup[]
		unresolvedCount: number
		loaded: boolean
		loading: boolean
		error: unknown
	}
	selected: {
		/** Issue IDs selected directly by toggles without an explicit ID across the current project. */
		issueIds: readonly string[]
		/** Selected toggle IDs for the current issue; panel and section callbacks receive all selected toggle IDs. */
		toggleIds: readonly string[]
	}
	/**
	 * Returns a markdown value for the current issue, or an empty string if unavailable.
	 * Panel and section callbacks must supply an issue ID to read stored input values.
	 * Only visible, enabled controls contribute when resolving messages and corrections.
	 * UI callbacks read stored values; message and correction callbacks also resolve defaults.
	 */
	getMarkdownValue(id: string, issueId?: string): string
	getTextValue(id: string, issueId?: string): string
	getSelectValue(id: string, issueId?: string): string
	getSelectValues(id: string, issueId?: string): readonly string[]
}

export type WithContext<T> = T | ((ctx: ReviewContext) => T)

export interface Issue {
	id: string
	title: string
	category: string
	message: WithContext<string>
	suggestedStatus?: WithContext<ModerationStatus | undefined>
	corrections?: WithContext<IssueCorrections>
	/** Marks the issue that requests applying the aggregated corrections. */
	applyCorrections?: boolean
}

export interface IssueCorrections {
	project?: Labrinth.Projects.v3.EditProjectRequest
	versions?: Record<string, Labrinth.Versions.v3.ModifyVersionRequest>
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

export type IssueControl = IssueToggle | IssueMarkdown | IssueText | IssueSelect

export interface Panel {
	icon: FunctionalComponent<SVGAttributes>
	shown?: WithContext<boolean>
	title?: WithContext<string>
	hint: WithContext<string>
	guidanceUrl: string
	children: PanelNode[]
}

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
