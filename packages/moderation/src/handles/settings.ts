import type {SettingDefinitionBase} from "../types/settings.ts";

export class Settings {
	private readonly settings: { [id: string]: any }
	private readonly onChange: () => void

	constructor(settings: { [id: string]: any } | undefined = undefined, onChange: () => void = () => {}) {
		this.settings = settings || {}
		this.onChange = onChange
	}

	get<T>(definition: SettingDefinitionBase<T>): T {
		return this.settings[definition.id] ?? definition.default
	}

	set<T>(definition: SettingDefinitionBase<T>, value?: T): void {
		const previous = this.settings[definition.id] ?? definition.default
		this.settings[definition.id] = value
		definition.onChange?.(previous, value ?? definition.default)
		this.onChange()
	}
}
