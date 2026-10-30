import keybinds from '../data/keybinds.ts'
import {
	type BaseKeybindListener,
	type KeybindDefinition,
	type KeybindListener,
	keybindsOverlap,
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
	switch (ctx.scope) {
		case 'review-panel-toggle':
			return keybind.scope === 'review-panel-toggle' ? bindKeybind(keybind, ctx) : undefined
		case 'project':
			return keybind.scope === 'project' ? bindKeybind(keybind, ctx) : undefined
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

	getConflicts(): Map<string, Set<string>> {
		const bindings = [...this]
		const conflicts = new Map<string, Set<string>>()
		for (const [index, [id, binding]] of bindings.entries()) {
			for (const [otherId, other] of bindings.slice(index)) {
				const sameReviewPage =
					['project-review', 'review-actions', 'review-panel-toggle'].includes(binding.scope) &&
					['project-review', 'review-actions', 'review-panel-toggle'].includes(other.scope)
				if (
					binding.scope !== other.scope &&
					binding.scope !== 'global' &&
					other.scope !== 'global' &&
					!sameReviewPage
				)
					continue
				const overlaps = binding.keybind.some((definition, definitionIndex) =>
					other.keybind.some(
						(otherDefinition, otherIndex) =>
							(id !== otherId || otherIndex > definitionIndex) &&
							keybindsOverlap(definition, otherDefinition),
					),
				)
				if (!overlaps) continue
				if (!conflicts.has(id)) conflicts.set(id, new Set())
				if (!conflicts.has(otherId)) conflicts.set(otherId, new Set())
				conflicts.get(id)!.add(otherId)
				conflicts.get(otherId)!.add(id)
			}
		}
		return conflicts
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
					ctx.scope !== 'review-panel-toggle' &&
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
