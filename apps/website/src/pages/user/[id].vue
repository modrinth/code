<script setup lang="ts">
import { UserPage } from '@modrinth/layouts'

const { id } = useRoute().params
</script>
<template>
	<UserPage :user-id="id" />
</template>
