<template>
	<div class="flex w-full flex-col gap-4">
		<!-- TODO: implement vanilla version selection UI -->
		<p class="text-secondary">Select compatible vanilla Minecraft versions.</p>
	</div>
</template>

<script setup lang="ts">
import { injectServerCompatibilityContext } from '../server-compatibility-modal'

const ctx = injectServerCompatibilityContext()
</script>
