import type { MessageTemplate } from '@modrinth/moderation'

export type MarkdownTemplate = MessageTemplate & { kind?: 'placeholder' }

export interface MarkdownTemplateQuery {
	trigger: '/' | '%'
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
