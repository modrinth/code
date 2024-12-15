import FloatingVue from "floating-vue";
import "floating-vue/dist/style.css";

export default defineNuxtPlugin((nuxtApp) => {
  nuxtApp.vueApp.use(FloatingVue, {
    themes: {
      dropdown: {
        placement: "bottom-end",
        instantMove: true,
        distance: 8,
      },
    },
  });
});
