import keybinds from '../data/keybinds.ts'
import {
	type BaseKeybindListener,
	type KeybindDefinition,
	type KeybindListener,
	matchesKeybind,
	type ModerationContext,
	normalizeKeybind,
} from '../types/keybinds.ts'

function normalizeKeybinds(
	keybind: KeybindDefinition | KeybindDefinition[] | string | string[],
): KeybindDefinition[] {
	return Array.isArray(keybind) ? keybind.map(normalizeKeybind) : [normalizeKeybind(keybind)]
}

function bindKeybind<T>(keybind: BaseKeybindListener<T>, ctx: T) {
	return {
		enabled: () => keybind.enabled?.(ctx) ?? true,
		run: () => keybind.action(ctx),
	}
}

function scopedKeybind(keybind: KeybindListener, ctx: ModerationContext) {
	if (ctx.scope === 'review-actions' && keybind.reviewAction) {
		const action = keybind.reviewAction
		return {
			enabled: () => ctx.available(action),
			run: () => ctx.run(action),
		}
	}
	switch (ctx.scope) {
		case 'project':
			return keybind.scope === 'project' ? bindKeybind(keybind, ctx) : undefined
		case 'checklist':
			return keybind.scope === 'checklist' ? bindKeybind(keybind, ctx) : undefined
		case 'tech-review':
			return keybind.scope === 'tech-review' ? bindKeybind(keybind, ctx) : undefined
		case 'global':
			return keybind.scope === 'global' ? bindKeybind(keybind, ctx) : undefined
		case 'project-review':
			return keybind.scope === 'project-review' ? bindKeybind(keybind, ctx) : undefined
		case 'review-conversation':
			return keybind.scope === 'review-conversation' ? bindKeybind(keybind, ctx) : undefined
		case 'review-actions':
			return keybind.scope === 'review-actions' ? bindKeybind(keybind, ctx) : undefined
		case 'review-composer':
			return keybind.scope === 'review-composer' ? bindKeybind(keybind, ctx) : undefined
	}
}

export type KeybindListenerWithDefault = KeybindListener & {
	keybind: KeybindDefinition[]
	defaultKeybind: KeybindDefinition[]
}

export class Keybinds {
	private readonly configured: { [id: string]: KeybindDefinition[] } = {}

	constructor(keybinds: { [id: string]: KeybindDefinition[] }) {
		this.configured = keybinds
	}

	*[Symbol.iterator](): IterableIterator<[string, KeybindListenerWithDefault]> {
		for (const [id, keybind] of Object.entries(keybinds)) {
			yield [
				id,
				{
					...keybind,
					keybind: this.configured[id] ?? normalizeKeybinds(keybind.keybind),
					defaultKeybind: normalizeKeybinds(keybind.keybind),
				},
			]
		}
	}

	set(id: string, keybind: KeybindDefinition | KeybindDefinition[] | string | string[]): void {
		this.configured[id] = normalizeKeybinds(keybind)
	}

	handle(event: KeyboardEvent, ctx: ModerationContext): boolean {
		if (
			ctx.scope !== 'global' &&
			ctx.scope !== 'review-composer' &&
			(event.target instanceof HTMLInputElement ||
				event.target instanceof HTMLTextAreaElement ||
				(event.target as HTMLElement)?.closest('.cm-editor') ||
				(event.target as HTMLElement)?.classList?.contains('cm-content') ||
				(event.target as HTMLElement)?.classList?.contains('cm-line'))
		) {
			return false
		}

		for (const [id, keybind] of Object.entries(keybinds)) {
			const binding = scopedKeybind(keybind, ctx)
			if (!binding?.enabled()) continue

			const definitions = this.configured[id] ?? normalizeKeybinds(keybind.keybind)
			const matches = definitions.some((def) => matchesKeybind(event, def))

			if (matches) {
				if (
					ctx.scope !== 'review-composer' &&
					ctx.scope !== 'review-actions' &&
					document.activeElement instanceof HTMLElement
				) {
					document.activeElement.blur()
				}

				binding.run()

				const shouldPrevent = definitions.some((def) => def.preventDefault !== false)
				if (shouldPrevent) {
					event.preventDefault()
				}

				event.stopPropagation()
				return true
			}
		}

		return false
	}
}
