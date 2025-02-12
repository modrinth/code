import dayjs from "dayjs";
import quarterOfYear from "dayjs/plugin/quarterOfYear";
import advanced from "dayjs/plugin/advancedFormat";

dayjs.extend(quarterOfYear);
dayjs.extend(advanced);

export default defineNuxtPlugin(() => {
  return {
    provide: {
      dayjs,
    },
  };
});
