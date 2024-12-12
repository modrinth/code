import FloatingVue from "floating-vue";

export default defineNuxtPlugin((nuxtApp) => {
  nuxtApp.vueApp.use(FloatingVue, {
    themes: {
      "ribbit-popout": {
        $extend: "dropdown",
        placement: "bottom-end",
        instantMove: true,
        distance: 8,
        triggers: ["click"],
      },
    },
  });
});
