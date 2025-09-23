import { fontStore } from '@ceereals/vue-pdf'

let didRegister = false

export function registerPdfFonts() {
	if (didRegister) return
	didRegister = true

	try {
		fontStore.register({
			family: 'Inter',
			fonts: [
				{
					src: 'https://cdn-raw.modrinth.com/fonts/inter/Inter-Regular.woff2?v=3.19',
					fontWeight: 400,
					fontStyle: 'normal',
				},
				{
					src: 'https://cdn-raw.modrinth.com/fonts/inter/Inter-Medium.woff2?v=3.19',
					fontWeight: 500,
					fontStyle: 'normal',
				},
				{
					src: 'https://cdn-raw.modrinth.com/fonts/inter/Inter-SemiBold.woff2?v=3.19',
					fontWeight: 600,
					fontStyle: 'normal',
				},
				{
					src: 'https://cdn-raw.modrinth.com/fonts/inter/Inter-Bold.woff2?v=3.19',
					fontWeight: 700,
					fontStyle: 'normal',
				},
				{
					src: 'https://cdn-raw.modrinth.com/fonts/inter/Inter-ExtraBold.woff2?v=3.19',
					fontWeight: 800,
					fontStyle: 'normal',
				},
			],
		})
	} catch (e) {
		console.warn('PDF font registration failed:', e)
	}
}
