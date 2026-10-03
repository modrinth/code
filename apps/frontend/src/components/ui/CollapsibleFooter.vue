<template>
	<ModrinthFooter v-if="!collapsible" />
	<template v-else>
		<div class="relative h-0">
			<button
				type="button"
				class="group absolute bottom-0 left-1/2 z-10 flex h-7 -translate-x-1/2 items-center gap-1 rounded-t-xl border border-b-0 border-solid border-surface-4 bg-surface-3 px-2 text-sm font-semibold text-secondary shadow-md transition-[opacity,transform,color] duration-200 hover:text-contrast focus-visible:text-contrast"
				:class="toggleVisible ? 'scale-100 opacity-100' : 'pointer-events-none scale-90 opacity-0'"
				:aria-expanded="expanded"
				:aria-label="label"
				:tabindex="toggleVisible ? 0 : -1"
				@click="expanded ? collapse() : expand()"
			>
				<ChevronDownIcon v-if="expanded" class="size-4 shrink-0" aria-hidden="true" />
				<ChevronUpIcon v-else class="size-4 shrink-0" aria-hidden="true" />
				<span
					class="max-w-0 overflow-hidden whitespace-nowrap transition-[max-width] duration-200 group-hover:max-w-[10rem] group-focus-visible:max-w-[10rem]"
				>
					{{ label }}
				</span>
			</button>
		</div>
		<div v-if="expanded" ref="footerRef">
			<ModrinthFooter />
		</div>
	</template>
</template>

<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { useIntersectionObserver, useScroll } from '@vueuse/core'

import ModrinthFooter from '~/components/ui/ModrinthFooter.vue'
import { useCollapsibleFooter, useFooterRevealIntent } from '~/composables/collapsible-footer'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	showFooter: {
		id: 'layout.footer.show',
		defaultMessage: 'Show footer',
	},
	hideFooter: {
		id: 'layout.footer.hide',
		defaultMessage: 'Hide footer',
	},
})

const BOTTOM_REVEAL_OFFSET = 24

const collapsible = useCollapsibleFooter()
const expanded = ref(false)
const footerRef = ref<HTMLElement | null>(null)

const { arrivedState } = useScroll(import.meta.client ? window : undefined, {
	offset: { bottom: BOTTOM_REVEAL_OFFSET },
})

const toggleVisible = computed(() => expanded.value || arrivedState.bottom)
const label = computed(() =>
	formatMessage(expanded.value ? messages.hideFooter : messages.showFooter),
)

/** Set once the expanded footer has been on screen, so scrolling back away from it collapses it. */
let footerSeen = false

function expand() {
	expanded.value = true
	footerSeen = false
	nextTick(() => footerRef.value?.scrollIntoView({ behavior: 'smooth', block: 'end' }))
}

function collapse() {
	expanded.value = false
	footerSeen = false
}

useIntersectionObserver(footerRef, ([entry]) => {
	if (!expanded.value || !entry) return
	if (entry.isIntersecting) {
		footerSeen = true
	} else if (footerSeen) {
		collapse()
	}
})

useFooterRevealIntent(
	computed(() => collapsible.value && !expanded.value),
	expand,
)

watch(collapsible, collapse)
</script>
