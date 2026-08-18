import { computed, reactive, ref, watch } from 'vue'

export const THEME_OPTIONS = ['dark', 'light', 'oled', 'retro', 'system'] as const

export type ColorTheme = (typeof THEME_OPTIONS)[number]
type Theme = Exclude<ColorTheme, 'system'>

const preferred = ref<ColorTheme>('dark')
const advancedRendering = ref(true)
const syncAcrossDevices = ref(false)
const nativeThemeQuery = window.matchMedia('(prefers-color-scheme: dark)')
const native = ref<Theme>(nativeThemeQuery.matches ? 'dark' : 'light')
const active = computed<Theme>(() =>
	preferred.value === 'system' ? native.value : preferred.value,
)

nativeThemeQuery.addEventListener('change', (event) => {
	native.value = event.matches ? 'dark' : 'light'
})

watch(
	active,
	(theme) => {
		const html = document.documentElement
		for (const option of THEME_OPTIONS) {
			html.classList.remove(`${option}-mode`)
		}
		html.classList.add(`${theme}-mode`)
	},
	{ immediate: true },
)

const theme = reactive({
	preferred,
	active,
	native,
	syncAcrossDevices,
	advancedRendering,
	options: THEME_OPTIONS,
})

export function useTheme() {
	return theme
}
