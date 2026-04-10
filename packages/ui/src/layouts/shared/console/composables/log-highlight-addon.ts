import type { IDecoration, IDisposable, IMarker, ITerminalAddon, Terminal } from '@xterm/xterm'

import { getCssVar } from '#ui/composables/terminal'

import type { LogLevel } from '../types'

export interface HighlightColors {
	errorPrimary: string
	errorWrap: string
	warnPrimary: string
	warnWrap: string
}

interface TrackedLine {
	marker: IMarker
	level: 'error' | 'warn'
	isEntryStart: boolean
	primary: IDecoration | undefined
	wraps: IDecoration[]
}

const LOG_ENTRY_START = /^\[\d{2}:\d{2}:\d{2}\]/

function parseHex(hex: string): [number, number, number] {
	const h = hex.startsWith('#') ? hex.slice(1) : hex
	return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)]
}

function blendHex(base: string, overlay: string, alpha: number): string {
	const [br, bg, bb] = parseHex(base)
	const [or, og, ob] = parseHex(overlay)
	const r = Math.round(br + (or - br) * alpha)
	const g = Math.round(bg + (og - bg) * alpha)
	const b = Math.round(bb + (ob - bb) * alpha)
	return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`
}

export function computeHighlightColors(): HighlightColors {
	const bg = getCssVar('--surface-2', '#1d1f23')
	const red = getCssVar('--color-red', '#ff496e')
	const orange = getCssVar('--color-orange', '#ffa347')
	return {
		errorPrimary: blendHex(bg, red, 0.15),
		errorWrap: blendHex(bg, red, 0.04),
		warnPrimary: blendHex(bg, orange, 0.15),
		warnWrap: blendHex(bg, orange, 0.04),
	}
}

export class LogHighlightAddon implements ITerminalAddon {
	private terminal: Terminal | null = null
	private tracked: TrackedLine[] = []
	private pendingLevels: Array<'error' | 'warn' | null> = []
	private pendingReapply: Array<LogLevel | null> | null = null
	private lastProcessedLine = 0
	private colors: HighlightColors
	private disposables: IDisposable[] = []

	constructor(colors: HighlightColors) {
		this.colors = colors
	}

	activate(terminal: Terminal): void {
		this.terminal = terminal
		this.disposables.push(
			terminal.onWriteParsed(() => this.onWriteParsed()),
			terminal.onResize(() => this.handleResize()),
		)
	}

	dispose(): void {
		for (const d of this.disposables) d.dispose()
		this.disposables = []
		this.clearTracked()
		this.terminal = null
	}

	pushLevel(level: LogLevel | null): void {
		if (level === 'error' || level === 'warn') {
			this.pendingLevels.push(level)
		} else {
			this.pendingLevels.push(null)
		}
	}

	reapply(levels: Array<LogLevel | null>): void {
		this.clearTracked()
		this.pendingLevels = []
		this.lastProcessedLine = 0
		this.pendingReapply = levels
	}

	updateColors(colors: HighlightColors): void {
		this.colors = colors
		this.rebuildAllDecorations()
	}

	private onWriteParsed(): void {
		if (this.pendingReapply) {
			this.applyAll(this.pendingReapply)
			this.pendingReapply = null
			return
		}
		this.processPending()
	}

	private processPending(): void {
		const term = this.terminal
		if (!term || this.pendingLevels.length === 0) return

		const buffer = term.buffer.active
		const cursorAbsolute = buffer.baseY + buffer.cursorY
		let levelIdx = 0

		for (
			let line = this.lastProcessedLine;
			line <= cursorAbsolute && levelIdx < this.pendingLevels.length;
			line++
		) {
			const bufLine = buffer.getLine(line)
			if (!bufLine || bufLine.isWrapped) continue

			const level = this.pendingLevels[levelIdx++]
			if (level) {
				this.decorateLogicalLine(line, level)
			}
		}

		this.lastProcessedLine = cursorAbsolute + 1
		this.pendingLevels = this.pendingLevels.slice(levelIdx)
	}

	private applyAll(levels: Array<LogLevel | null>): void {
		const term = this.terminal
		if (!term) return

		const buffer = term.buffer.active
		let levelIdx = 0

		for (let line = 0; line < buffer.length && levelIdx < levels.length; line++) {
			const bufLine = buffer.getLine(line)
			if (!bufLine || bufLine.isWrapped) continue

			const level = levels[levelIdx++]
			if (level === 'error' || level === 'warn') {
				this.decorateLogicalLine(line, level)
			}
		}

		this.lastProcessedLine = buffer.baseY + buffer.cursorY + 1
	}

	private decorateLogicalLine(bufferLine: number, level: 'error' | 'warn'): void {
		const term = this.terminal
		if (!term) return

		const buffer = term.buffer.active
		const cursorAbsolute = buffer.baseY + buffer.cursorY
		const offset = bufferLine - cursorAbsolute
		const marker = term.registerMarker(offset)
		if (!marker) return

		const lineText = buffer.getLine(bufferLine)?.translateToString(true) ?? ''
		const isEntryStart = LOG_ENTRY_START.test(lineText)

		const bgColor = isEntryStart
			? level === 'error'
				? this.colors.errorPrimary
				: this.colors.warnPrimary
			: level === 'error'
				? this.colors.errorWrap
				: this.colors.warnWrap
		const wrapColor = level === 'error' ? this.colors.errorWrap : this.colors.warnWrap

		const primary = term.registerDecoration({
			marker,
			backgroundColor: bgColor,
			width: term.cols,
			layer: 'bottom',
		})

		const wraps = this.createWrapDecorations(bufferLine, wrapColor)

		this.tracked.push({ marker, level, isEntryStart, primary, wraps })
	}

	private createWrapDecorations(primaryLine: number, color: string): IDecoration[] {
		const term = this.terminal
		if (!term) return []

		const buffer = term.buffer.active
		const decorations: IDecoration[] = []
		const cursorAbsolute = buffer.baseY + buffer.cursorY

		for (let line = primaryLine + 1; line < buffer.length; line++) {
			const bufLine = buffer.getLine(line)
			if (!bufLine || !bufLine.isWrapped) break

			const offset = line - cursorAbsolute
			const wrapMarker = term.registerMarker(offset)
			if (!wrapMarker) continue

			const dec = term.registerDecoration({
				marker: wrapMarker,
				backgroundColor: color,
				width: term.cols,
				layer: 'bottom',
			})
			if (dec) decorations.push(dec)
		}

		return decorations
	}

	private handleResize(): void {
		this.rebuildAllDecorations()
	}

	private rebuildAllDecorations(): void {
		const term = this.terminal
		if (!term) return

		for (const tl of this.tracked) {
			tl.primary?.dispose()
			for (const w of tl.wraps) w.dispose()

			if (tl.marker.line === -1) {
				tl.primary = undefined
				tl.wraps = []
				continue
			}

			const bgColor = tl.isEntryStart
				? tl.level === 'error'
					? this.colors.errorPrimary
					: this.colors.warnPrimary
				: tl.level === 'error'
					? this.colors.errorWrap
					: this.colors.warnWrap
			const wrapColor = tl.level === 'error' ? this.colors.errorWrap : this.colors.warnWrap

			tl.primary = term.registerDecoration({
				marker: tl.marker,
				backgroundColor: bgColor,
				width: term.cols,
				layer: 'bottom',
			})

			tl.wraps = this.createWrapDecorations(tl.marker.line, wrapColor)
		}

		this.tracked = this.tracked.filter((tl) => tl.marker.line !== -1)
	}

	private clearTracked(): void {
		for (const tl of this.tracked) {
			tl.primary?.dispose()
			for (const w of tl.wraps) w.dispose()
			tl.marker.dispose()
		}
		this.tracked = []
	}
}
