import {
	type KeybindDefinition, type KeybindListener, matchesKeybind,
	type ModerationContext, normalizeKeybind
} from "../types/keybinds.ts";
import keybinds from "../data/keybinds.ts";

function normalizeKeybinds(keybind: KeybindDefinition | KeybindDefinition[] | string | string[]): KeybindDefinition[] {
	return Array.isArray(keybind) ? keybind.map(normalizeKeybind) : [normalizeKeybind(keybind)]
}

export type KeybindListenerWithDefault = KeybindListener & {
	keybind: KeybindDefinition[],
	defaultKeybind: KeybindDefinition[],
};

export class Keybinds {
	private readonly configured: { [id: string]: KeybindDefinition[] } = {}

	constructor(keybinds: { [id: string]: KeybindDefinition[] }) {
		this.configured = keybinds
	}

	* [Symbol.iterator](): IterableIterator<[string, KeybindListenerWithDefault]> {
		for (const [id, keybind] of Object.entries(keybinds)) {
			yield [id, {
				...keybind,
				keybind: this.configured[id] ?? normalizeKeybinds(keybind.keybind),
				defaultKeybind: normalizeKeybinds(keybind.keybind),
			}]
		}
	}

	set(id: string, keybind: KeybindDefinition | KeybindDefinition[] | string | string[]): void {
		this.configured[id] = normalizeKeybinds(keybind)
	}

	handle(event: KeyboardEvent, ctx: ModerationContext): boolean {
		if (
			event.target instanceof HTMLInputElement ||
			event.target instanceof HTMLTextAreaElement ||
			(event.target as HTMLElement)?.closest('.cm-editor') ||
			(event.target as HTMLElement)?.classList?.contains('cm-content') ||
			(event.target as HTMLElement)?.classList?.contains('cm-line')
		) {
			return false
		}

		for (const [id, keybind] of Object.entries(keybinds)) {
			if (ctx.scope !== keybind.scope) {
				continue
			}

			if (keybind.enabled && !keybind.enabled(ctx as any)) {
				continue
			}

			const definitions = this.configured[id] ?? normalizeKeybinds(keybind.keybind)
			const matches = definitions.some((def) => matchesKeybind(event, def))

			if (matches) {
				keybind.action(ctx as any)

				const shouldPrevent = definitions.some((def) => def.preventDefault !== false)
				if (shouldPrevent) {
					event.preventDefault()
				}

				return true
			}
		}

		return false
	}

}
