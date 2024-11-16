<script setup>
const token = defineModel();

useHead({
  script: [
    {
      src: "https://js.hcaptcha.com/1/api.js",
      async: true,
      defer: true,
    },
  ],
});

function updateToken(newToken) {
  token.value = newToken;
}

onMounted(() => {
  window.updateCatpchaToken = updateToken;
});

defineExpose({
  reset: () => {
    token.value = null;
    window.hcaptcha.reset();
  },
});
</script>

<template>
  <div
    id="h-captcha"
    class="h-captcha"
    data-sitekey="4a7a2c80-68f2-4190-9d52-131c76e0c14e"
    :data-theme="$theme.active === 'light' ? 'light' : 'dark'"
    data-callback="updateCatpchaToken"
  ></div>
</template>

<style lang="scss">
.h-captcha {
  display: flex;
  justify-content: center;
  overflow: hidden;
  border-radius: var(--radius-md);
  border: 2px solid var(--color-button-bg);
  height: 78px;

  iframe {
    margin: -1px;
  }
}
</style>
