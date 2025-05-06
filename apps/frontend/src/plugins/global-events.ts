import mitt from "mitt";

export default defineNuxtPlugin(() => {
  const emitter = mitt();
  return {
    provide: {
      event: emitter.emit,
      listen: emitter.on,
    },
  };
});
