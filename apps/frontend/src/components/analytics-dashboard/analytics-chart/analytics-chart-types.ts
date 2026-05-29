export type AnalyticsChartRangeBounds = {
	start: Date
	end: Date
}

export type AnalyticsChartHoverState = {
	visible: boolean
	x: number
	y: number
	sliceIndex: number | null
}

export type AnalyticsChartLegendEntry = {
	id: string
	name: string
	projectName?: string
	color: string
	totalValue: number
	hidden: boolean
	isPreviousPeriod?: boolean
}
