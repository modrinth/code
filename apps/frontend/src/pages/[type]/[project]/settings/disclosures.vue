<script setup lang="ts">
import { SmartClickable, Toggle } from '@modrinth/ui'
</script>

<template>
	<SmartClickable class="flex w-full">
		<template #clickable>
			<label for="test" class="flex h-full w-full cursor-pointer" />
		</template>
		<div
			class="grid w-full grid-cols-[1fr_auto] items-center gap-6 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4"
		>
			<div>
				<h3 class="mb-1 mt-0 text-base font-semibold text-contrast">
					Contains AI-generated content
				</h3>
				<p class="mb-2 mt-0 text-sm leading-normal">
					You must enable this if this project contains a substantial amount of AI-generated code,
					any assets that are substantially AI-generated, the project's design relies on the use of
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
			</div>
			<div>
				<Toggle id="test" class="smart-clickable:allow-pointer-events" />
			</div>
		</div>
	</SmartClickable>
</template>
