<script setup lang="ts">
import type { Ref } from 'vue'
import { computed, onBeforeUnmount, toRef, watch } from 'vue'

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

const loadingState = injectLoadingState(null)
let token: symbol | null = null

function release() {
	if (token && loadingState) {
		loadingState.end(token)
	}
	token = null
}

if (loadingState && !props.silent) {
	watch(
		resolvedPending,
		(now) => {
			if (typeof window === 'undefined') return
			if (now) {
				if (!token) token = loadingState.begin()
			} else {
				release()
			}
		},
		{ immediate: true },
	)
}

onBeforeUnmount(release)
</script>

<template>
	<Transition name="ready-fade" mode="out-in" :duration="props.duration">
		<div v-if="!resolvedPending" key="content" class="ready-transition-content">
			<slot />
		</div>
		<div v-else key="pending" aria-hidden="true" class="ready-transition-pending" />
	</Transition>
</template>

<style lang="scss" scoped>
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
