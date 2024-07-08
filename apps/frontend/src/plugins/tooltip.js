import FloatingVue from "floating-vue";
import "floating-vue/dist/style.css";

// eslint-disable-next-line no-undef
export default defineNuxtPlugin(({ vueApp }) => {
  vueApp.use(FloatingVue);
});
