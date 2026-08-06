import { isValidFor, type SettingDefinitionBase } from '../types/settings.ts'
import { moderationSettings } from '../index.ts'

export class Settings {
	private readonly settings: { [id: string]: any }
	private readonly onChange: () => void

	constructor(
		settings: { [id: string]: any } | undefined = undefined,
		onChange: () => void = () => {},
	) {
		this.settings = settings || {}
		this.onChange = onChange
	}

	get<T>(definition: SettingDefinitionBase<T>): T {
		const value = this.settings[definition.id]

		return (isValidFor(definition, value) ? value : undefined) ?? definition.default
	}

	set<T>(definition: SettingDefinitionBase<T>, value?: T): void {
		const previous = this.settings[definition.id] ?? definition.default
		this.settings[definition.id] = value
		definition.onChange?.(previous, value ?? definition.default)
		this.onChange()
	}
}

export function getMarginTarget(settings: Settings): string {
	return settings.get(moderationSettings.General.AdjustPageAlignment) == 'always'
		? settings.get(moderationSettings.General.ChecklistPosition) == 'right'
			? 'r'
			: 'l'
		: 'x'
}
