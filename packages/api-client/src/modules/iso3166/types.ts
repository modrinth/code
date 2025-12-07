export namespace ISO3166 {
	export interface Country {
		alpha2: string
		alpha3: string
		numeric: string
		nameShort: string
		nameLong: string
	}

	export interface Subdivision {
		code: string // Full ISO 3166-2 code (e.g., "US-NY")
		name: string // Official name in local language
		localVariant: string | null // English variant if different
		category: string // STATE, PROVINCE, REGION, etc.
		parent: string | null // Parent subdivision code
		language: string // Language code
	}

	export interface State {
		countries: Country[]
		subdivisions: Record<string, Subdivision[]>
	}
}
