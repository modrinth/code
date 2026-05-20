<script setup lang="ts">
/**
 * If `pending` is false on mount and never becomes true, the slot renders with no
 * enter transition (cache-hit fast path). After a real pending phase, transitions
 * behave as before for subsequent toggles.
 */
import type { Ref } from 'vue'
import { computed, onBeforeUnmount, ref, toRef, watch } from 'vue'

import { injectLoadingState } from '#ui/providers/loading-state'

const props = withDefaults(
	defineProps<{
		/** True while the wrapped content is still loading. Slot stays blank, loading bar runs. */
		pending: boolean | Ref<boolean>
		/** Fade duration applied to the slot when content reveals. */
		duration?: number
		/** When true, do NOT register a token with the global loading bar — only fade locally. */
		silent?: boolean
	}>(),
	{
		duration: 200,
		silent: false,
	},
)

const pendingRef = toRef(props, 'pending') as Ref<boolean | Ref<boolean>>
const resolvedPending = computed(() => {
	const v = pendingRef.value
	if (typeof v === 'boolean') return v
	return Boolean((v as Ref<boolean>).value)
})

const hasBeenPending = ref(false)
const useShell = computed(() => resolvedPending.value || hasBeenPending.value)

const loadingState = injectLoadingState(null)
let token: symbol | null = null

function release() {
	if (token && loadingState) {
		loadingState.end(token)
	}
	token = null
}

watch(
	resolvedPending,
	(now) => {
		if (now) {
			hasBeenPending.value = true
		}
		if (loadingState && !props.silent && typeof window !== 'undefined') {
			if (now) {
				if (!token) token = loadingState.begin()
			} else {
				release()
			}
		}
	},
	{ immediate: true },
)

onBeforeUnmount(release)
</script>

<template>
	<template v-if="useShell">
		<Transition name="ready-fade" mode="out-in" :duration="props.duration">
			<div v-if="!resolvedPending" key="content" class="ready-transition-content">
				<slot />
			</div>
			<div v-else key="pending" aria-hidden="true" class="ready-transition-pending" />
		</Transition>
	</template>
	<slot v-else />
</template>

<style scoped>
.ready-fade-enter-active,
.ready-fade-leave-active {
	transition: opacity v-bind('`${props.duration}ms`') ease-in-out;
}

.ready-fade-enter-from,
.ready-fade-leave-to {
	opacity: 0;
}

.ready-transition-content {
	width: 100%;
}

.ready-transition-pending {
	width: 100%;
	height: 100%;
}
</style>
