<template>
	<span class="inline-flex align-middle mr-1 pointer-events-none">
		<Checkbox :model-value="checked" />
	</span>
</template>

<script setup lang="ts">
import Checkbox from '../Checkbox.vue'

defineOptions({
	inheritAttrs: false,
})

defineProps<{
	checked?: boolean
}>()
</script>
