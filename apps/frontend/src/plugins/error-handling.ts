export default defineNuxtPlugin((nuxtApp) => {
	nuxtApp.hook('app:error', (error: any) => {
		console.error('An error occurred:', error)
	})
})
