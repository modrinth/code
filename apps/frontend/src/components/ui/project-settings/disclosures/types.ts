import type { Labrinth } from '@modrinth/api-client'

export type TelemetryConsent = Labrinth.Projects.v3.TelemetryConsent
export type AiUsage = Labrinth.Projects.v3.AiUsage
export type DerivativeSource = Labrinth.Projects.v3.DerivativeSource
export type ProjectDisclosure = Labrinth.Projects.v3.ProjectDisclosure
export type ProjectDisclosureData = Labrinth.Projects.v3.ProjectDisclosureData
export type DisclosureType = Labrinth.Projects.v3.ProjectDisclosureType
export type DisclosureOf<T extends DisclosureType> = Labrinth.Projects.v3.ProjectDisclosureOf<T>

export type NoteDisclosure = {
	enabled: boolean
	note: string
}

export type AiDisclosure = {
	enabled: boolean
	uses: AiUsage[]
	note: string
}

export type PaidFeaturesDisclosure = {
	enabled: boolean
	features: string[]
}

export type TelemetryDisclosure = {
	enabled: boolean
	consent: TelemetryConsent
	entries: string[]
}

export type DerivativeDisclosure = {
	enabled: boolean
	sources: DerivativeSource[]
}

export type DisclosureFormState = {
	ai: AiDisclosure
	advertising: NoteDisclosure
	paidFeatures: PaidFeaturesDisclosure
	telemetry: TelemetryDisclosure
	derivative: DerivativeDisclosure
	photosensitivity: NoteDisclosure
	systemInteractions: NoteDisclosure
}
