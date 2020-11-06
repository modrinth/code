<template>
  <div>
    <div
      v-if="!showAlt"
      class="ethical-ad"
      data-ea-publisher="modrinth-com"
      :data-ea-type="type"
      data-ea-manual="true"
    />
    <div v-else class="alt">
      <p>
        A privacy-focused ad used to fund this site would've been here. Please
        disable your content blocker to support Modrinth and it's authors.
      </p>
    </div>
  </div>
</template>

<script>
export default {
  name: 'EthicalAd',
  props: {
    type: {
      type: String,
      default: 'text',
    },
  },
  data() {
    return {
      showAlt: false,
    }
  },
  mounted() {
    try {
      // eslint-disable-next-line no-undef
      if (typeof ethicalads === 'undefined') {
        this.$notify({
          group: 'ads',
          title: 'Please disable your Content Blocker',
          text:
            'Modrinth uses privacy-focused ads, from EthicalAds. Ads are the only way that our site is able to pay modders and support itself. Our ads are non-intrusive and minimal, and we only have one per page. We can assure you that none of your data is sold or used for tracking purposes.',
          type: 'error',
        })

        this.showAlt = true
      } else {
        const element = document.getElementsByClassName('ethical-ad')[0]

        element.className = 'ethical-ad'
        element.innerHTML = ''

        // eslint-disable-next-line no-undef
        ethicalads.load()

        element.className = 'ethical-ad loaded ' + this.$colorMode.preference
      }
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error(err)
    }
  },
}
</script>

<style lang="scss" scoped>
[data-ea-type='text'] {
  min-height: 70px;
}
[data-ea-type='image'] {
  margin: auto 10px;
  min-height: 260px;
}

.alt {
  font-size: 14px;
  border-radius: var(--size-rounded-sm);
  background-color: var(--color-grey-1);
  box-shadow: 0 2px 3px rgba(0, 0, 0, 0.15);
  padding: 0.7em 1em;
  margin: 1em 1em 2em 1em;
}
</style>
