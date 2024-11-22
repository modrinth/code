<script setup>
const token = defineModel();
const id = ref(null);

function hCaptchaUpdateToken(newToken) {
  token.value = newToken;
}

function hCaptchaReady() {
  window.hCaptchaUpdateToken = hCaptchaUpdateToken;
  id.value = window.hcaptcha.render("h-captcha");
}

onMounted(() => {
  window.hCaptchaReady = hCaptchaReady;

  useHead({
    script: [
      {
        src: "https://js.hcaptcha.com/1/api.js?render=explicit&onload=hCaptchaReady&hl=zh",
        async: true,
        defer: true,
      },
    ],
  });
});

defineExpose({
  reset: () => {
    token.value = null;
    window.hcaptcha.reset(id.value);
  },
});
</script>

<template>
  <div
    id="h-captcha"
    class="h-captcha"
    data-sitekey="fae429f6-da46-4b72-9b34-5f3011acd844"
    :data-theme="$theme.active === 'light' ? 'light' : 'dark'"
    data-callback="hCaptchaUpdateToken"
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
