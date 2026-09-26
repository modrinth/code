export namespace Minelog {
	export namespace Insights {
		export namespace v2 {
			export type Detection = {
				value: string
				text: string
				detail: string | null
				confidence: string
				lines: number[]
				others: unknown[]
			}

			export type Environment = {
				gameVersion: Detection | null
				loader: Detection | null
				java: Detection | null
				launcher: Detection | null
				conflicts: unknown[]
			}

			export type Problem = {
				id: string
				message: string
				solutions: string[]
				count: number
				line: number
			}

			export type InsightsResponse = {
				kind: string
				title: string
				software: { id: string; name: string } | null
				minecraftVersion: string | null
				environment: Environment
				mods: unknown[]
				bundledMods: number
				problems: Problem[]
			}
		}
	}

	export namespace Logs {
		export namespace v2 {
			export type CreateResponse = {
				id: string
				url: string
				raw: string
				source: string | null
				kind: string
				lines: number
				size: number
				errors: number
				warnings: number
				privacyApplied: boolean
				createdAt: string
				expiresAt: string
				deleteToken: string
				deletableUntil: string
			}
		}
	}
}
