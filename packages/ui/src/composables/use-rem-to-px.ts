import {
computed, type ComputedRef, type MaybeRefOrGetter,
onMounted, onUnmounted, type Ref, 	ref, 	toValue
} from 'vue'

export interface RemCompute {
	remToPx: (rem: MaybeRefOrGetter<number>) => ComputedRef<number>;
	rootFontSize: Ref<number>;
}

export function useRemToPx(): RemCompute {
	const rootFontSize = ref(16)

	const updateRootFontSize = () => {
		if (typeof window !== 'undefined') {
			rootFontSize.value = parseFloat(getComputedStyle(document.documentElement).fontSize) || 16
		}
	}

	onMounted(() => {
		updateRootFontSize()
		window.addEventListener('resize', updateRootFontSize)
	})

	onUnmounted(() => {
		window.removeEventListener('resize', updateRootFontSize)
	})

	const remToPx = (rem: MaybeRefOrGetter<number>) => computed(() => toValue(rem) * rootFontSize.value)

	return { remToPx, rootFontSize }
}
