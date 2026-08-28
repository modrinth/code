import { useVIntl } from '@modrinth/ui/i18n'

import type { Nag } from '../types/nags.ts'
import { nagDestinations } from './nag-destinations.ts'
import type { ValidationRuleMatch, ValidationRuleSeverity } from './types.ts'

function toNagStatus(severity: ValidationRuleSeverity): Nag['status'] {
	if (severity === 'error') return 'required'
	return severity
}

export function toNags(matches: ValidationRuleMatch[]): Nag[] {
	return matches.map((match) => {
		const presentation = match.rule.presentation
		const destination = presentation.nag.destination
			? nagDestinations[presentation.nag.destination]
			: undefined

		return {
			id: match.code,
			title: presentation.nag.title,
			description: () => {
				const { formatMessage } = useVIntl()
				const values = presentation.nag.formatValues
					? presentation.nag.formatValues(match.values, formatMessage)
					: match.values
				return formatMessage(match.message, values)
			},
			status: toNagStatus(match.rule.severity),
			shouldShow: () => true,
			...(destination
				? {
						link: {
							...destination,
							title: presentation.nag.linkTitle ?? destination.title,
						},
					}
				: {}),
		}
	})
}
