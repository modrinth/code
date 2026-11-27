<script setup lang="ts">
import { SkinButton } from '@modrinth/ui'
import { useIntersectionObserver } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import type { Cape, Skin } from '@/helpers/skins'
import { renderSkinPreview } from '@/platform/app-lib/skins/commands'

const props = defineProps<{
	skin: Skin
	capes: Cape[]
	selected: boolean
}>()
const button = ref<InstanceType<typeof SkinButton>>()
const visible = ref(false)
const url = ref<string>()
const capeTexture = computed(
	() => props.capes.find((cape) => cape.id === props.skin.cape_id)?.texture,
)
const key = computed(
	() => `${props.skin.texture_key}+${props.skin.variant}+${capeTexture.value ?? 'no-cape'}`,
)

useIntersectionObserver(
	button,
	([entry]) => {
		visible.value = entry?.isIntersecting ?? false
	},
	{ rootMargin: '200px' },
)

watch(
	[visible, key],
	([isVisible], _, onCleanup) => {
		url.value = undefined
		if (!isVisible) return
		let active = true
		onCleanup(() => {
			active = false
		})
		void renderSkinPreview(props.skin, capeTexture.value)
			.then((result) => {
				if (active) url.value = result
			})
			.catch((error) => console.warn('Could not load skin preview', error))
	},
	{ immediate: true },
)
</script>

<template>
	<SkinButton ref="button" :selected="selected" :forward-image-src="url">
		<template v-for="(_, name) in $slots" #[name]="slotProps">
			<slot :name="name" v-bind="slotProps ?? {}" />
		</template>
	</SkinButton>
</template>
