import 'floating-vue/dist/style.css'
import 'overlayscrollbars/overlayscrollbars.css'

import { VueScanPlugin } from '@taijased/vue-render-tracker'
import { VueQueryPlugin } from '@tanstack/vue-query'
import FloatingVue from 'floating-vue'
import { createPinia } from 'pinia'
import { createApp } from 'vue'

import App from '@/App.vue'
import { overlayScrollbarsDirective } from '@/directives/overlayScrollbars'
import i18nPlugin from '@/plugins/i18n'
import i18nDebugPlugin from '@/plugins/i18n-debug'
import router from '@/routes'

const vueScan = new VueScanPlugin({
	enabled: false, // Enable or disable the tracker
	showOverlay: true, // Show overlay to visualize renders
	log: false, // Log render events to the console
	playSound: false, // Play sound on each render
})

const pinia = createPinia()

let app = createApp(App)

app.use(VueQueryPlugin)
app.use(vueScan)
app.use(router)
app.use(pinia)
app.use(FloatingVue, {
	themes: {
		'ribbit-popout': {
			$extend: 'dropdown',
			placement: 'bottom-end',
			instantMove: true,
			distance: 8,
		},
		'dismissable-prompt': {
			$extend: 'dropdown',
			placement: 'bottom-start',
		},
	},
})
app.use(i18nPlugin)
app.use(i18nDebugPlugin)
app.directive('overlay-scrollbars', overlayScrollbarsDirective)

app.mount('#app')
