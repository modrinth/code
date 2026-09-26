import type {
	Issue,
	IssueConfig,
	IssueMarkdown,
	IssueMarkdownConfig,
	IssueSelect,
	IssueSelectConfig,
	IssueText,
	IssueTextConfig,
	IssueToggle,
	IssueToggleConfig,
	PanelConfig,
	PanelNode,
	PanelSectionConfig,
} from './types'

class ContentBuilder {
	children: PanelNode[] = []

	content(...children: PanelNode[]): this {
		this.children = children
		return this
	}
}

export function issue(config: IssueConfig): Issue {
	return { ...config }
}

export function panel(config: PanelConfig) {
	return Object.assign(new ContentBuilder(), config)
}

export function section(config: PanelSectionConfig = {}) {
	return Object.assign(new ContentBuilder(), config, { type: 'section' as const })
}

export function toggle(config: IssueToggleConfig): IssueToggle {
	return { ...config, type: 'toggle' }
}

export function markdown(config: IssueMarkdownConfig): IssueMarkdown {
	return { ...config, type: 'markdown' }
}

export function text(config: IssueTextConfig): IssueText {
	return { ...config, type: 'text' }
}

export function select(config: IssueSelectConfig): IssueSelect {
	return { ...config, type: 'select' }
}
