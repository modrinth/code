<template>
  <div class="avatar-icon">
    <img :src="this.$auth.user.avatar_url" class="icon" />
    <div v-if="notifCount > 0" class="bubble" :class="{ dropdownBg }">
      {{ displayNotifCount }}
    </div>
  </div>
</template>

<script>
export default {
  name: 'AvatarIcon',
  props: {
    notifCount: {
      type: Number,
      default: 0,
    },
    dropdownBg: {
      type: Boolean,
      default: false,
    },
  },
  computed: {
    displayNotifCount() {
      return this.notifCount < 100 ? this.notifCount : '99+'
    },
  },
}
</script>

<style lang="scss" scoped>
.avatar-icon {
  position: relative;
  height: 2rem;
  width: 2rem;
  margin-left: 0.5rem;
  margin-right: 0.25rem;

  .icon {
    height: 100%;
    width: 100%;
    border-radius: 50%;
  }

  .bubble {
    position: absolute;
    bottom: -0.25rem;
    right: -0.3rem;

    border-radius: 0.9rem;
    height: 0.9rem;
    min-width: 0.45rem;
    padding: 0 0.22rem;
    font-size: 0.65rem;

    display: flex;
    justify-content: center;
    align-items: center;

    background-color: #e02914;
    color: white;
    border: 0.15rem solid var(--color-raised-bg);

    &.dropdownBg {
      border-color: var(--color-button-bg);
    }
  }
}
</style>
<style lang="scss">
.dropdown:hover {
  .bubble {
    border-color: var(--color-button-bg);
  }
}
</style>
