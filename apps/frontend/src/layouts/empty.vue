<template>
	<TooltipDirective />
	<slot id="main" />
</template>

<script setup lang="ts">
import { TooltipDirective } from '@modrinth/ui'

useFavicon()
</script>

<style lang="scss">
@import '~/assets/styles/global.scss';
</style>
