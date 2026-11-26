<script setup lang="ts">
import DisclosuresSettings from '~/components/project-settings/disclosures.vue'
import { disclosureTitle } from '~/components/ui/project-settings/disclosures/messages'

useProjectSettingsHeadTitle(disclosureTitle)
</script>

<template>
	<DisclosuresSettings />
</template>
