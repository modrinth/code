<script setup lang="ts">
import { ToggleCard } from '@modrinth/ui'

const aiDisclosure = ref(false)
</script>

<template>
	<ToggleCard v-model="aiDisclosure">
		<h3 class="mb-1 mt-0 text-base font-semibold text-contrast">Contains AI-generated content</h3>
		<p class="mb-2 mt-0 text-sm leading-normal">
			You must enable this if this project contains a substantial amount of AI-generated code, any
			assets that are substantially AI-generated, the project's design relies on the use of
			generative AI, or if any element of your project's page such as description or publishing
			relies on generative AI.
		</p>
		<p class="m-0 text-sm leading-normal">
			Please refer to Section 6 of Modrinth's
			<nuxt-link
				to="/legal/rules#generative-ai"
				target="_blank"
				class="smart-clickable:allow-pointer-events underline hover:text-contrast"
				>Content Rules</nuxt-link
			>
			for more information.
		</p>
		<template #expanded>
			<p class="mb-2 mt-0 text-sm leading-normal">the contents within</p>
		</template>
	</ToggleCard>
</template>
