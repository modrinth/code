import type { Archon } from '../modules/archon/types.js'

export type ParsedSseEvent = {
	kind: 'event'
	id?: string
	event?: string
	data: string
}

export type ParsedSseRetry = {
	kind: 'retry'
	retry: number
}

export type ParsedSseItem = ParsedSseEvent | ParsedSseRetry

export class SseParser {
	private buffer = ''
	private eventName = ''
	private data = ''
	private id: string | undefined

	feed(chunk: string): ParsedSseItem[] {
		this.buffer += chunk
		const items: ParsedSseItem[] = []

		while (true) {
			const lineEnd = this.findLineEnd()
			if (!lineEnd) break

			const { line, length } = lineEnd
			this.buffer = this.buffer.slice(length)
			this.processLine(line, items)
		}

		return items
	}

	end(): ParsedSseItem[] {
		const items: ParsedSseItem[] = []

		if (this.buffer.length > 0) {
			this.processLine(this.buffer.endsWith('\r') ? this.buffer.slice(0, -1) : this.buffer, items)
			this.buffer = ''
		}

		this.dispatch(items)
		return items
	}

	private findLineEnd(): { line: string; length: number } | null {
		const lf = this.buffer.indexOf('\n')
		const cr = this.buffer.indexOf('\r')

		if (lf === -1 && cr === -1) return null

		if (cr !== -1 && (lf === -1 || cr < lf)) {
			if (cr === this.buffer.length - 1) return null
			const length = this.buffer[cr + 1] === '\n' ? cr + 2 : cr + 1
			return {
				line: this.buffer.slice(0, cr),
				length,
			}
		}

		return {
			line: this.buffer.slice(0, lf),
			length: lf + 1,
		}
	}

	private processLine(line: string, items: ParsedSseItem[]): void {
		if (line === '') {
			this.dispatch(items)
			return
		}

		if (line.startsWith(':')) return

		const colon = line.indexOf(':')
		const field = colon === -1 ? line : line.slice(0, colon)
		let value = colon === -1 ? '' : line.slice(colon + 1)
		if (value.startsWith(' ')) value = value.slice(1)

		switch (field) {
			case 'event':
				this.eventName = value
				break
			case 'data':
				this.data += `${value}\n`
				break
			case 'id':
				this.id = value
				break
			case 'retry': {
				const retry = Number(value)
				if (Number.isInteger(retry) && retry >= 0) {
					items.push({ kind: 'retry', retry })
				}
				break
			}
		}
	}

	private dispatch(items: ParsedSseItem[]): void {
		if (!this.data) {
			this.eventName = ''
			this.id = undefined
			return
		}

		items.push({
			kind: 'event',
			id: this.id,
			event: this.eventName || undefined,
			data: this.data.endsWith('\n') ? this.data.slice(0, -1) : this.data,
		})

		this.eventName = ''
		this.data = ''
		this.id = undefined
	}
}

export function parseSyncEventData(data: string): Archon.Sync.v1.SyncEvent | null {
	let parsed: unknown

	try {
		parsed = JSON.parse(data)
	} catch {
		return null
	}

	if (!parsed || typeof parsed !== 'object') return null
	const event = parsed as { type?: unknown }
	if (typeof event.type !== 'string') return null

	return parsed as Archon.Sync.v1.SyncEvent
}
