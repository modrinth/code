declare module '*.vue' {
	import type { defineComponent } from 'vue'

	const component: ReturnType<typeof defineComponent>
	export default component
}

declare module '*.glsl' {
	const value: string
	export default value
}
