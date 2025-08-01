import FloatingVue from "floating-vue";
import "floating-vue/dist/style.css";

export default defineNuxtPlugin((nuxtApp) => {
  nuxtApp.vueApp.use(FloatingVue, {
    themes: {
      "ribbit-popout": {
        $extend: "dropdown",
        placement: "bottom-end",
        instantMove: true,
        distance: 8,
      },
      "dismissable-prompt": {
        $extend: "dropdown",
        placement: "bottom-start",
      },
    },
  });
});
