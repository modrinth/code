export namespace Mclogs {
	export namespace Insights {
		export namespace v1 {
			export type LogEntry = {
				level: number
				time: string | null
				prefix: string
				lines: Array<{ number: number; content: string }>
			}

			export type Solution = {
				message: string
			}

			export type Problem = {
				message: string
				counter: number
				entry: LogEntry
				solutions: Solution[]
			}

			export type Information = {
				message: string
				counter: number
				label: string
				value: string
				entry: LogEntry
			}

			export type Analysis = {
				problems: Problem[]
				information: Information[]
			}

			export type InsightsResponse = {
				id: string
				name: string
				type: string
				version: string
				title: string
				analysis: Analysis
			}
		}
	}

	export namespace Logs {
		export namespace v1 {
			export type CreateResponse = {
				success: boolean
				id: string
				source: string | null
				created: number
				expires: number
				size: number
				lines: number
				errors: number
				url: string
				raw: string
				token: string
			}
		}
	}
}
