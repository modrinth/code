<script setup lang="ts">
import { FloatingMenu } from '@modrinth/ui'
import { ref } from 'vue'

import { useTechReviewMenu } from './menus'

defineOptions({ inheritAttrs: false })
const emit = defineEmits<{ open: []; close: [] }>()
const menu = ref<InstanceType<typeof FloatingMenu>>()
const panel = ref<HTMLElement>()
const coordination = useTechReviewMenu({
	getTrigger: () => menu.value?.$el,
	getPanel: () => panel.value?.closest<HTMLElement>('[data-popper-placement]'),
	hide: () => menu.value?.hide(),
})

function onOpen() {
	coordination.open()
	emit('open')
}

function onClose() {
	coordination.close()
	emit('close')
}

defineExpose({ show: () => menu.value?.show(), hide: () => menu.value?.hide() })
</script>

<template>
	<FloatingMenu ref="menu" v-bind="$attrs" @open="onOpen" @close="onClose">
		<slot />
		<template #popper="{ hide }">
			<div ref="panel">
				<slot name="popper" :hide="hide" />
			</div>
		</template>
	</FloatingMenu>
</template>
