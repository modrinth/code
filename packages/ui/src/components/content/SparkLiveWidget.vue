<script setup lang="ts">
import { VideoIcon } from '@modrinth/assets'

withDefaults(
	defineProps<{
		embed?: boolean
	}>(),
	{
		embed: false,
	},
)
</script>

<template>
	<div class="rounded-2xl border border-solid border-surface-4 bg-surface-3 overflow-hidden">
		<div class="p-4 flex flex-col gap-3">
			<div class="flex items-center gap-2">
				<VideoIcon class="size-6 text-brand" />
				<span class="text-contrast font-medium">
					We're hosting a live chat with Spark later today to answer all of your questions!
				</span>
			</div>
			<span>
				Ask questions for us to answer in our
				<a
					href="https://discord.modrinth.com"
					target="_blank"
					class="text-brand font-semibold hover:underline"
					>Discord server</a
				>.
			</span>
			<span
				>Tune in live on June 15, 11am PST / 2pm EST / 7pm BST / 8pm CEST over on our
				<a
					href="https://www.youtube.com/live/p1Dg-fud0TQ"
					target="_blank"
					class="text-brand font-semibold hover:underline"
					>YouTube channel</a
				>.</span
			>
		</div>
		<div
			v-if="embed"
			style="left: 0; width: 100%; height: 0; position: relative; padding-bottom: 56.25%"
		>
			<iframe
				src="https://www.youtube.com/embed/p1Dg-fud0TQ"
				style="top: 0; left: 0; width: 100%; height: 100%; position: absolute; border: 0"
				allowfullscreen
				scrolling="no"
				allow="
					accelerometer *;
					clipboard-write *;
					encrypted-media *;
					gyroscope *;
					picture-in-picture *;
					web-share *;
				"
				referrerpolicy="strict-origin"
			></iframe>
		</div>
	</div>
</template>
