<template>
  <span v-if="typeOnly" class="environment">
    <InfoIcon aria-hidden="true" />
    {{ formatMessage(messages.typeLabel, { type: type }) }}
  </span>
  <span
    v-else-if="
      !['resourcepack', 'shader'].includes(type) &&
      !(type === 'plugin' && search) &&
      !categories.includes('datapack')
    "
    class="environment"
  >
    <template v-if="clientSide === 'optional' && serverSide === 'optional'">
      <GlobeIcon aria-hidden="true" />
      {{ formatMessage(messages.clientOrServerLabel) }}
    </template>
    <template v-else-if="clientSide === 'required' && serverSide === 'required'">
      <GlobeIcon aria-hidden="true" />
      {{ formatMessage(messages.clientAndServerLabel) }}
    </template>
    <template
      v-else-if="
        (clientSide === 'optional' || clientSide === 'required') &&
        (serverSide === 'optional' || serverSide === 'unsupported')
      "
    >
      <ClientIcon aria-hidden="true" />
      {{ formatMessage(messages.clientLabel) }}
    </template>
    <template
      v-else-if="
        (serverSide === 'optional' || serverSide === 'required') &&
        (clientSide === 'optional' || clientSide === 'unsupported')
      "
    >
      <ServerIcon aria-hidden="true" />
      {{ formatMessage(messages.serverLabel) }}
    </template>
    <template v-else-if="serverSide === 'unsupported' && clientSide === 'unsupported'">
      <GlobeIcon aria-hidden="true" />
      {{ formatMessage(messages.unsupportedLabel) }}
    </template>
    <template v-else-if="alwaysShow">
      <InfoIcon aria-hidden="true" />
      {{ formatMessage(messages.typeLabel, { type: type }) }}
    </template>
  </span>
</template>
<script setup>
import { GlobeIcon, ClientIcon, ServerIcon, InfoIcon } from '@'
import { useVIntl, defineMessages } from '@vintl/vintl'
const messages = defineMessages({
  clientLabel: {
    id: 'omorphia.component.environment-indicator.label.client',
    defaultMessage: 'Client',
  },
  clientAndServerLabel: {
    id: 'omorphia.component.environment-indicator.label.client-and-server',
    defaultMessage: 'Client and server',
  },
  clientOrServerLabel: {
    id: 'omorphia.component.environment-indicator.label.client-or-server',
    defaultMessage: 'Client or server',
  },
  serverLabel: {
    id: 'omorphia.component.environment-indicator.label.server',
    defaultMessage: 'Server',
  },
  typeLabel: {
    id: 'omorphia.component.environment-indicator.label.type',
    defaultMessage: 'A {type}',
  },
  unsupportedLabel: {
    id: 'omorphia.component.environment-indicator.label.unsupported',
    defaultMessage: 'Unsupported',
  },
})
const { formatMessage } = useVIntl()
</script>
<script>
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    type: {
      type: String,
      default: 'mod',
    },
    serverSide: {
      type: String,
      required: false,
      default: '',
    },
    clientSide: {
      type: String,
      required: false,
      default: '',
    },
    typeOnly: {
      type: Boolean,
      required: false,
      default: false,
    },
    alwaysShow: {
      type: Boolean,
      required: false,
      default: false,
    },
    search: {
      type: Boolean,
      required: false,
      default: false,
    },
    categories: {
      type: Array,
      required: false,
      default() {
        return []
      },
    },
  },
})
</script>
<style lang="scss" scoped>
.environment {
  display: flex;
  color: var(--color-text) !important;
  font-weight: bold;
  font-size: 1rem;
  align-items: center;
  svg {
    margin-right: 0.2rem;
  }
}
</style>
