import { EyeOffIcon, OctagonAlertIcon, ShieldAlertIcon, VirusIcon } from '@modrinth/assets'

export const severityPresentation = {
	malware: {
		icon: VirusIcon,
		class: 'bg-highlight-red text-red',
		textClass: 'text-red',
		bgColor: 'var(--color-red-bg)',
		color: 'var(--color-red)',
	},
	severe: {
		icon: ShieldAlertIcon,
		class: 'bg-highlight-red text-red',
		textClass: 'text-red',
		bgColor: 'var(--color-red-bg)',
		color: 'var(--color-red)',
	},
	high: {
		icon: OctagonAlertIcon,
		class: 'text-red',
		textClass: 'text-red',
		bgColor: 'var(--color-red-bg)',
		color: 'var(--color-red)',
	},
	medium: {
		icon: OctagonAlertIcon,
		class: 'text-orange',
		textClass: 'text-orange',
		bgColor: 'var(--color-orange-bg)',
		color: 'var(--color-orange)',
	},
	low: {
		icon: OctagonAlertIcon,
		class: 'text-yellow',
		textClass: 'text-yellow',
		bgColor: 'var(--color-yellow-bg)',
		color: 'var(--color-yellow)',
	},
	hidden: {
		icon: EyeOffIcon,
		class: 'text-secondary',
		textClass: 'text-secondary',
		bgColor: 'var(--surface-3)',
		color: 'var(--color-secondary)',
	},
}

export function traceName(issueType: string) {
	const name = issueType.replace(/_/g, ' ').toLowerCase()
	return name.charAt(0).toUpperCase() + name.slice(1)
}
