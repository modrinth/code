<template>
	<LoaderPicker v-model="draftVersion.loaders" :loaders="generatedState.loaders" />
</template>

<script lang="ts" setup>
import { useManageVersion } from '~/composables/versions/manage-version'

import LoaderPicker from '../components/LoaderPicker.vue'

const generatedState = useGeneratedState()

const { draftVersion } = useManageVersion()
</script>
