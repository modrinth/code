<script setup lang="ts">
import { useVIntl, defineMessages } from '@vintl/vintl'
import type { VersionChannel } from '@modrinth/utils'

const { formatMessage } = useVIntl()

withDefaults(
  defineProps<{
    channel: VersionChannel
    large?: boolean
  }>(),
  {
    large: false,
  },
)

const messages = defineMessages({
  releaseSymbol: {
    id: 'project.versions.channel.release.symbol',
    defaultMessage: 'R',
  },
  betaSymbol: {
    id: 'project.versions.channel.beta.symbol',
    defaultMessage: 'B',
  },
  alphaSymbol: {
    id: 'project.versions.channel.alpha.symbol',
    defaultMessage: 'A',
  },
})
</script>

<template>
  <div
    :class="`flex ${large ? 'text-lg w-10.5 h-10.5' : 'text-sm w-9 h-9'} font-bold justify-center items-center rounded-full ${channel === 'release' ? 'bg-bg-green text-green' : channel === 'beta' ? 'bg-bg-orange text-orange' : 'bg-bg-red text-red'}`"
  >
    {{ channel ? formatMessage(messages[`${channel}Symbol`]) : '?' }}
  </div>
</template>
