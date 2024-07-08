import dayjs from "dayjs";

// eslint-disable-next-line no-undef
export default defineNuxtPlugin(() => {
  return {
    provide: {
      dayjs,
    },
  };
});
