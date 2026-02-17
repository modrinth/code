<template>
	<div class="flex w-full flex-col gap-4">
		<!-- TODO: implement custom modpack upload UI -->
		<p class="text-secondary">Upload a custom modpack file for server compatibility.</p>
	</div>
</template>

<script setup lang="ts">
import { injectServerCompatibilityContext } from '../server-compatibility-modal'

const ctx = injectServerCompatibilityContext()
</script>
