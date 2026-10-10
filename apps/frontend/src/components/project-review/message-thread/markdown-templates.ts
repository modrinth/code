export type { MessageTemplate as MarkdownTemplate } from '@modrinth/moderation'

export interface MarkdownTemplateQuery {
	from: number
	to: number
	query: string
	anchor: {
		contextElement: HTMLElement
		getBoundingClientRect: () => {
			x: number
			y: number
			left: number
			right: number
			top: number
			bottom: number
			width: number
			height: number
		}
	}
}
