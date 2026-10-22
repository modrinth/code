import { isValidFor, moderationSettings, type Settings } from '@modrinth/moderation'

const general = moderationSettings.General
export const appearanceFields = {
	highlightColor: general.ReviewHighlightColor,
	highlightOpacity: general.ReviewHighlightOpacity,
	panelBorderColor: general.FloatingReviewPanelBorderColor,
	panelBackground: general.FloatingReviewPanelBackground,
	panelOpacity: general.FloatingReviewPanelBackgroundOpacity,
	panelBorderWidth: general.FloatingReviewPanelBorderWidth,
	issueColor: general.IssueToggleColor,
}
export type AppearanceValues = {
	[K in keyof typeof appearanceFields]: (typeof appearanceFields)[K]['default']
}

export function readAppearance(settings: Settings): AppearanceValues {
	return Object.fromEntries(
		Object.entries(appearanceFields).map(([key, definition]) => [
			key,
			settings.get<string | number>(definition),
		]),
	) as AppearanceValues
}

export function applyAppearance(settings: Settings, values: AppearanceValues) {
	for (const [key, definition] of Object.entries(appearanceFields)) {
		settings.set<string | number>(definition, values[key as keyof AppearanceValues])
	}
}

export const defaultAppearance = Object.fromEntries(
	Object.entries(appearanceFields).map(([key, definition]) => [key, definition.default]),
) as AppearanceValues

export function exportAppearance(settings: Settings) {
	const values = readAppearance(settings)
	return JSON.stringify(
		{
			format: 'modrinth-review-appearance',
			version: 1,
			values: Object.fromEntries(
				Object.entries(values).map(([key, value]) => [key, value === '' ? null : value]),
			),
		},
		null,
		2,
	)
}

export function parseAppearance(source: string): AppearanceValues {
	const data = JSON.parse(source)
	if (
		!data ||
		data.format !== 'modrinth-review-appearance' ||
		data.version !== 1 ||
		!data.values ||
		typeof data.values !== 'object' ||
		Array.isArray(data.values)
	)
		throw new Error('format')
	if (Object.keys(data.values).length !== Object.keys(appearanceFields).length)
		throw new Error('fields')
	const values: Record<string, string | number> = {}
	for (const [key, definition] of Object.entries(appearanceFields)) {
		const value = data.values[key] === null && definition.type === 'color' ? '' : data.values[key]
		if (!isValidFor(definition, value)) throw new Error('value')
		values[key] = typeof value === 'string' ? value.toLowerCase() : value
	}
	return values as AppearanceValues
}
