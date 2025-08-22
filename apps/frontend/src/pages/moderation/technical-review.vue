<script setup lang="ts">
import RunDelphiModal from '~/components/ui/moderation/delphi/RunDelphiModal.vue'

const modal = ref<InstanceType<typeof RunDelphiModal>>()
function openRunDelphi() {
	modal.value?.show()
}
</script>

<template>
	<RunDelphiModal ref="modal" @started="() => {}" />
	<section class="card">
		<button @click="openRunDelphi">Run Delphi</button>
	</section>
</template>
