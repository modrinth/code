import type { ElementNode } from 'comark'
import { defineComarkPlugin } from 'comark/parse'
import { visitAsync } from 'comark/utils'

const MARKER_RE = /^!([\w-]+)$/

function splitAlertRemainder(remainder: string): {
	open?: boolean
	title?: string
	body: string
} {
	let rest = remainder
	let open: boolean | undefined
	if (rest[0] === '+' || rest[0] === '-') {
		open = rest[0] === '+'
		rest = rest.slice(1)
	}

	const newline = rest.indexOf('\n')
	const titleLine = (newline === -1 ? rest : rest.slice(0, newline)).trim()
	const body = newline === -1 ? '' : rest.slice(newline + 1)

	return { open, title: titleLine || undefined, body }
}

export const alert = defineComarkPlugin(() => ({
	name: 'alert',
	async post(state) {
		await visitAsync(
			state.tree,
			(node) => Array.isArray(node) && node[0] === 'blockquote',
			(node) => {
				const element = node as ElementNode
				const holder = element[2]?.[0] === 'p' ? (element[2] as ElementNode) : element
				if (holder[2]?.[0] !== 'span') return

				const match = typeof holder[2][2] === 'string' ? MARKER_RE.exec(holder[2][2]) : null
				if (!match) return

				const remainderNode = holder[3]
				const remainder = typeof remainderNode === 'string' ? remainderNode : ''
				const { open, title, body } = splitAlertRemainder(remainder)

				holder.splice(2, 1)
				if (typeof remainderNode === 'string') holder[2] = body

				element[1].as = 'alert'
				element[1].type = match[1].toLowerCase()
				if (title) element[1].title = title
				if (!body.trim()) element[1].noBody = true
				if (open !== undefined) {
					element[1].foldable = true
					if (open) element[1].open = true
				}
			},
		)
	},
}))
