<script setup lang="ts">
import { commonProjectSettingsMessages } from '@modrinth/ui'

import MembersSettings from '~/components/project-settings/members/index.vue'

useProjectSettingsHeadTitle(commonProjectSettingsMessages.members)
</script>

<template>
	<MembersSettings />
</template>
