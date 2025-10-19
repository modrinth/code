<template>
	<div class="flex flex-col gap-4">
		<p>Completion Stage</p>
		<!-- TBD: Implement completion confirmation display -->
		<!-- Show summary of what was done/selected -->
		<!-- Reference: https://www.figma.com/design/9bI55sqgicwUezmRXBo9bd/Design-System?node-id=524-3174&m=dev -->
		<ConfettiExplosion v-if="showConfetti" />
	</div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import ConfettiExplosion from 'vue-confetti-explosion'

const showConfetti = ref(false)

onMounted(() => {
	// Show confetti when completion stage is mounted
	showConfetti.value = true
	// Hide confetti after 5 seconds like in moderation page
	setTimeout(() => {
		showConfetti.value = false
	}, 5000)
})

// TBD: Implement completion summary display
// TBD: Show selected provider, method, amounts, etc.
</script>
