import { $fetch } from 'ofetch'

import { AbstractModule } from '../../core/abstract-module'
import type { ISO3166 } from './types'

export type { ISO3166 } from './types'

const ISO3166_REPO = 'https://raw.githubusercontent.com/ipregistry/iso3166/master'

/**
 * Parse CSV string into array of objects
 * @param csv - CSV string to parse
 * @returns Array of objects with header keys mapped to row values
 */
function parseCSV(csv: string): Record<string, string>[] {
	const lines = csv
		.trim()
		.split('\n')
		.filter((line) => line.trim() !== '')

	if (lines.length === 0) return []

	const headerLine = lines[0]
	const headers = (headerLine.startsWith('#') ? headerLine.slice(1) : headerLine).split(',')

	return lines.slice(1).map((line) => {
		const values = line.split(',')
		const row: Record<string, string> = {}

		headers.forEach((header, index) => {
			row[header] = values[index] || ''
		})

		return row
	})
}

/**
 * Module for fetching ISO 3166 country and subdivision data
 * Data from https://github.com/ipregistry/iso3166 (Licensed under CC BY-SA 4.0)
 * @platform Not for use in Tauri or Nuxt environments, only node.
 */
export class ISO3166Module extends AbstractModule {
	public getModuleID(): string {
		return 'iso3166_data'
	}

	/**
	 * Build ISO 3166 country and subdivision data from the ipregistry repository
	 *
	 * @returns Promise resolving to countries and subdivisions data
	 *
	 * @example
	 * ```typescript
	 * const data = await client.iso3166.data.build()
	 * console.log(data.countries) // Array of country data
	 * console.log(data.subdivisions['US']) // Array of US state data
	 * ```
	 */
	public async build(): Promise<ISO3166.State> {
		try {
			const [countriesCSV, subdivisionsCSV] = await Promise.all([
				$fetch<string>(`${ISO3166_REPO}/countries.csv`, {
					// @ts-expect-error supports text
					responseType: 'text',
				}),
				$fetch<string>(`${ISO3166_REPO}/subdivisions.csv`, {
					// @ts-expect-error supports text
					responseType: 'text',
				}),
			])

			const countriesData = parseCSV(countriesCSV)
			const subdivisionsData = parseCSV(subdivisionsCSV)

			const countries: ISO3166.Country[] = countriesData.map((c) => ({
				alpha2: c.country_code_alpha2,
				alpha3: c.country_code_alpha3,
				numeric: c.numeric_code,
				nameShort: c.name_short,
				nameLong: c.name_long,
			}))

			// Group subdivisions by country code
			const subdivisions: Record<string, ISO3166.Subdivision[]> = subdivisionsData.reduce(
				(acc, sub) => {
					const countryCode = sub.country_code_alpha2

					if (!countryCode || typeof countryCode !== 'string' || countryCode.trim() === '') {
						return acc
					}

					if (!acc[countryCode]) acc[countryCode] = []

					acc[countryCode].push({
						code: sub['subdivision_code_iso3166-2'],
						name: sub.subdivision_name,
						localVariant: sub.localVariant || null,
						category: sub.category,
						parent: sub.parent_subdivision || null,
						language: sub.language_code,
					})

					return acc
				},
				{} as Record<string, ISO3166.Subdivision[]>,
			)

			return {
				countries,
				subdivisions,
			}
		} catch (err) {
			console.error('Error fetching ISO3166 data:', err)
			return {
				countries: [],
				subdivisions: {},
			}
		}
	}
}
