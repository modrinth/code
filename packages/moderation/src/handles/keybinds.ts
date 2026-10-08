import keybinds from '../data/keybinds.ts'
import {
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
			const reviewAction = ctx.scope === 'review-actions' ? keybind.reviewAction : undefined
			if (ctx.scope !== keybind.scope && !reviewAction) {
				continue
			}

			// The scope check above guarantees ctx matches keybind's expected context shape,
			// but TS can't correlate that narrowing across these two independently-typed variables.
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			if (
				reviewAction && ctx.scope === 'review-actions'
					? !ctx.available(reviewAction)
					: keybind.enabled && !keybind.enabled(ctx as any)
			) {
				continue
			}

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

				// eslint-disable-next-line @typescript-eslint/no-explicit-any
				if (reviewAction && ctx.scope === 'review-actions') ctx.run(reviewAction)
				else keybind.action(ctx as any)

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
