<template>
  <span v-if="typeOnly" class="environment">
    <InfoIcon aria-hidden="true" />
    A {{ type }}
  </span>
  <span
    v-else-if="
      !['resourcepack', 'shader'].includes(type) &&
      !(type === 'plugin' && search) &&
      !categories.some((x) => tags.loaderData.dataPackLoaders.includes(x))
    "
    class="environment"
  >
    <template v-if="clientSide === 'optional' && serverSide === 'optional'">
      <GlobeIcon aria-hidden="true" />
      Client or server
    </template>
    <template v-else-if="clientSide === 'required' && serverSide === 'required'">
      <GlobeIcon aria-hidden="true" />
      Client and server
    </template>
    <template
      v-else-if="
        (clientSide === 'optional' || clientSide === 'required') &&
        (serverSide === 'optional' || serverSide === 'unsupported')
      "
    >
      <ClientIcon aria-hidden="true" />
      Client
    </template>
    <template
      v-else-if="
        (serverSide === 'optional' || serverSide === 'required') &&
        (clientSide === 'optional' || clientSide === 'unsupported')
      "
    >
      <ServerIcon aria-hidden="true" />
      Server
    </template>
    <template v-else-if="serverSide === 'unsupported' && clientSide === 'unsupported'">
      <GlobeIcon aria-hidden="true" />
      Unsupported
    </template>
    <template v-else-if="alwaysShow">
      <InfoIcon aria-hidden="true" />
      A {{ type }}
    </template>
  </span>
</template>
<script setup>
import InfoIcon from '~/assets/images/utils/info.svg?component'
import ClientIcon from '~/assets/images/utils/client.svg?component'
import GlobeIcon from '~/assets/images/utils/globe.svg?component'
import ServerIcon from '~/assets/images/utils/server.svg?component'

defineProps({
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
})

const tags = useTags()
</script>
<style lang="scss" scoped>
.environment {
  display: flex;
  color: var(--color-text) !important;
  font-weight: bold;
  svg {
    margin-right: 0.2rem;
  }
}
</style>
