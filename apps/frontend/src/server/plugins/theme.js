// eslint-disable-next-line no-undef
export default defineNitroPlugin((nitroApp) => {
  nitroApp.hooks.hook("render:html", (html, { event }) => {
    try {
      // eslint-disable-next-line no-undef
      const cookies = parseCookies(event);

      if (cookies && cookies["color-mode"]) {
        const theme = JSON.parse(cookies["color-mode"]);

        html.htmlAttrs.push(`class="${theme.value}-mode"`);
      } else {
        html.htmlAttrs.push(`class="dark-mode"`);
      }

      // Reset cookie attributes to correct ones
      if (cookies) {
        const opts = {
          maxAge: 60 * 60 * 24 * 365 * 10,
          sameSite: "lax",
          secure: true,
          httpOnly: false,
          path: "/",
        };

        if (cookies["auth-token"]) {
          // eslint-disable-next-line no-undef
          setCookie(event, "auth-token", cookies["auth-token"], opts);
        }
        if (cookies["color-mode"]) {
          // eslint-disable-next-line no-undef
          setCookie(event, "color-mode", cookies["color-mode"], opts);
        }
        if (cookies.cosmetics) {
          // eslint-disable-next-line no-undef
          setCookie(event, "cosmetics", cookies.cosmetics, opts);
        }
      }
    } catch (err) {
      console.error(err);
    }
  });
});
