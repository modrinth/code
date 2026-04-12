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

type HighlightClass = 'hl-error-primary' | 'hl-error-wrap' | 'hl-warn-primary' | 'hl-warn-wrap'

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
	private colors: HighlightColors
	private disposables: IDisposable[] = []
	private styleElement: HTMLStyleElement | null = null

	constructor(colors: HighlightColors) {
		this.colors = colors
	}

	activate(terminal: Terminal): void {
		this.terminal = terminal
		this.injectStylesheet()
		this.disposables.push(terminal.onResize(() => this.rebuildAllDecorations()))
	}

	dispose(): void {
		for (const d of this.disposables) d.dispose()
		this.disposables = []
		this.clearAll()
		this.styleElement?.remove()
		this.styleElement = null
		this.terminal = null
	}

	applyFromLine(startLine: number, levels: Array<LogLevel | null>): void {
		const term = this.terminal
		if (!term) return

		const buffer = term.buffer.active
		let levelIdx = 0

		for (let line = startLine; line < buffer.length && levelIdx < levels.length; line++) {
			const bufLine = buffer.getLine(line)
			if (!bufLine || bufLine.isWrapped) continue

			const level = levels[levelIdx++]
			if (level === 'error' || level === 'warn') {
				this.decorateLogicalLine(line, level)
			}
		}
	}

	clearAll(): void {
		for (const tl of this.tracked) {
			tl.primary?.dispose()
			for (const w of tl.wraps) w.dispose()
			tl.marker.dispose()
		}
		this.tracked = []
	}

	updateColors(colors: HighlightColors): void {
		this.colors = colors
		this.updateStylesheet()
		this.rebuildAllDecorations()
	}

	private injectStylesheet(): void {
		const el = this.terminal?.element
		if (!el) return
		this.styleElement = document.createElement('style')
		this.updateStylesheet()
		el.appendChild(this.styleElement)
	}

	private updateStylesheet(): void {
		if (!this.styleElement) return
		this.styleElement.textContent = [
			`.hl-error-primary { background-color: ${this.colors.errorPrimary} !important; }`,
			`.hl-error-wrap { background-color: ${this.colors.errorWrap} !important; }`,
			`.hl-warn-primary { background-color: ${this.colors.warnPrimary} !important; }`,
			`.hl-warn-wrap { background-color: ${this.colors.warnWrap} !important; }`,
		].join('\n')
	}

	private classForDecoration(level: 'error' | 'warn', isEntryStart: boolean): HighlightClass {
		if (level === 'error') return isEntryStart ? 'hl-error-primary' : 'hl-error-wrap'
		return isEntryStart ? 'hl-warn-primary' : 'hl-warn-wrap'
	}

	private tagElement(dec: IDecoration | undefined, cls: HighlightClass): void {
		if (!dec) return
		const disposable = dec.onRender((el) => {
			el.classList.add(cls)
			disposable.dispose()
		})
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

		const primary = term.registerDecoration({
			marker,
			backgroundColor: bgColor,
			width: term.cols,
			layer: 'bottom',
		})
		this.tagElement(primary, this.classForDecoration(level, isEntryStart))
		const wraps = this.createWrapDecorations(bufferLine, level)

		this.tracked.push({ marker, level, isEntryStart, primary, wraps })
	}

	private createWrapDecorations(primaryLine: number, level: 'error' | 'warn'): IDecoration[] {
		const term = this.terminal
		if (!term) return []

		const buffer = term.buffer.active
		const decorations: IDecoration[] = []
		const cursorAbsolute = buffer.baseY + buffer.cursorY
		const cls = this.classForDecoration(level, false)
		const color = level === 'error' ? this.colors.errorWrap : this.colors.warnWrap

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
			if (dec) {
				this.tagElement(dec, cls)
				decorations.push(dec)
			}
		}

		return decorations
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

			const cls = this.classForDecoration(tl.level, tl.isEntryStart)
			const bgColor = tl.isEntryStart
				? tl.level === 'error'
					? this.colors.errorPrimary
					: this.colors.warnPrimary
				: tl.level === 'error'
					? this.colors.errorWrap
					: this.colors.warnWrap
			tl.primary = term.registerDecoration({
				marker: tl.marker,
				backgroundColor: bgColor,
				width: term.cols,
				layer: 'bottom',
			})
			this.tagElement(tl.primary, cls)
			tl.wraps = this.createWrapDecorations(tl.marker.line, tl.level)
		}

		this.tracked = this.tracked.filter((tl) => tl.marker.line !== -1)
	}
}
