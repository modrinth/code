export type IconBackground =
	| { type: 'color'; value: string }
	| { type: 'linear-top-down-gradient'; top_color: string; bottom_color: string }

/** An icon created with the icon editor: a symbol from the frontend's catalog drawn over a background. */
export type IconConfig = {
	background: IconBackground
	symbol: string
}
