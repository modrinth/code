import { installTooltipDirective } from '@modrinth/ui'

export default defineNuxtPlugin((nuxtApp) => {
	installTooltipDirective(nuxtApp.vueApp)
})
