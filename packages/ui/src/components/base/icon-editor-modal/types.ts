export type IconBackground =
	| { type: 'color'; value: string }
	| { type: 'linear-top-down-gradient'; top_color: string; bottom_color: string }

export type IconConfig = {
	background: IconBackground
	symbol: string
}
