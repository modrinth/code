<template>
	<div class="flex w-full flex-col gap-4">
		<!-- TODO: implement published modpack selection UI -->
		<p class="text-secondary">Select a published modpack for server compatibility.</p>
	</div>
</template>

<script setup lang="ts">
import { injectServerCompatibilityContext } from '../server-compatibility-modal'

const ctx = injectServerCompatibilityContext()
</script>
