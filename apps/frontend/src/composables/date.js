import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";

dayjs.extend(relativeTime); // eslint-disable-line import/no-named-as-default-member

export const useCurrentDate = () => useState("currentDate", () => Date.now());

export const updateCurrentDate = () => {
  const currentDate = useCurrentDate();

  currentDate.value = Date.now();
};

export const fromNow = (date) => {
  const currentDate = useCurrentDate();
  return dayjs(date).from(currentDate.value);
};
