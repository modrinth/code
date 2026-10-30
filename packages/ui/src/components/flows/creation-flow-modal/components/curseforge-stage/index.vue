<template>
	<ModpackPicker />
</template>

<script setup lang="ts">
import { injectCreationFlowContext } from '../../creation-flow-context'
import { provideCurseForgeSelection } from '../../curseforge'
import ModpackPicker from './modpack-picker.vue'

provideCurseForgeSelection(injectCreationFlowContext())
</script>
