<script setup lang="ts">
import ModerationKeybind from "~/components/ui/moderation/settings/ModerationKeybind.vue";

const keybinds = useModerationKeybinds()
</script>

<template>
	<div class="universal-card">
		<h2 class="text-2xl">Keybinds</h2>
		<div class="grid grid-cols-2 gap-x-12 gap-y-4">
			<ModerationKeybind
				v-for="[id, keybind] in keybinds"
				:key="id"
				:title="keybind.description"
				:global="keybind.scope === 'project'"
				:definitions="keybind.keybind"
				:default="keybind.defaultKeybind"
				:on-change="
							(definitions) => {
								keybinds.set(id, definitions)
								saveModerationOptions()
							}
						"
			/>
		</div>
	</div>
</template>
