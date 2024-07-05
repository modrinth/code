/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./components/**/*.{js,vue,ts}",
    "./layouts/**/*.vue",
    "./pages/**/*.vue",
    "./plugins/**/*.{js,ts}",
    "./app.vue",
    "./error.vue",
    // monorepo
    "../../packages/**/*.{js,vue,ts}",
  ],
  theme: {
    extend: {
      colors: {
        light: {
          secondary: "#6b7280",
          icon: "#6b7280",
          text: {
            DEFAULT: "hsl(221, 39%, 11%)",
            inactive: "hsl(215, 14%, 34%)",
            dark: "#1a202c",
            inverted: "#e5e7eb",
          },
          heading: "#2c313d",
          bg: "#e5e7eb",
          raisedBg: "#ffffff",
          divider: {
            DEFAULT: "hsl(220, 13%, 91%)",
            dark: "#c8cdd3",
          },
          brand: {
            DEFAULT: "#00af5c",
            green: "#00af5c",
            highlight: "rgba(0, 175, 92, 0.25)",
            shadow: "rgba(0, 175, 92, 0.7)",
            inverted: "#ffffff",
          },
          tabUnderlineHovered: "#e2e8f0",
          button: {
            bg: "hsl(220, 13%, 91%)",
            text: "#1a202c",
            bgHover: "#d9dce0",
            textHover: "#1b1e24",
            bgActive: "#c3c6cb",
            textActive: "#1b1e24",
          },
          toggleHandle: "#6b7280",
          dropdown: {
            bg: "hsl(220, 13%, 91%)",
            text: "#1a202c",
          },
          tooltip: {
            bg: "#1a202c",
            text: "#e5e7eb",
          },
          code: {
            bg: "#e5e7eb",
            text: "#1a202c",
          },
          kbdShadow: "rgba(0, 0, 0, 0.25)",
          ad: {
            DEFAULT: "#d6e6f9",
            raised: "#b1c8e4",
            contrast: "#1a202c",
            highlight: "#088cdb",
          },
          greyLink: {
            DEFAULT: "#1a202c",
            hover: "#2c313d",
            active: "#1a202c",
          },
          link: {
            DEFAULT: "#0d60bb",
            hover: "#1a76e7",
            active: "#146fd7",
          },
          redBg: "rgba(204, 35, 69, 0.1)",
          warning: {
            bg: "hsl(355, 70%, 88%)",
            text: "hsl(342, 70%, 35%)",
            banner: {
              text: "hsl(0, 11%, 16%)",
              bg: "hsl(0, 100%, 95%)",
              side: "hsl(357, 78%, 40%)",
            },
          },
          infoBanner: {
            text: "#1a202c",
            bg: "#d6e6f9",
            side: "#088cdb",
          },
          blockQuote: "#1a202c",
          headerUnderline: "#c8cdd3",
          hr: "#1a202c",
          table: {
            border: "#dfe2e5",
            alternateRow: "#f2f4f7",
          },
          shadow: {
            insetLg: "inset 0px -2px 2px hsla(221, 39%, 11%, 0.1)",
            inset: "inset 0px -2px 2px hsla(221, 39%, 11%, 0.05)",
            insetSm: "inset 0px -1px 2px hsla(221, 39%, 11%, 0.15)",
            raisedLg: "0px 2px 4px hsla(221, 39%, 11%, 0.2)",
            raised:
              "0.3px 0.5px 0.6px hsla(221, 39%, 11%, 0.15), 1px 2px 2.2px -1.7px hsla(221, 39%, 11%, 0.12), 4.4px 8.8px 9.7px -3.4px hsla(221, 39%, 11%, 0.09)",
            floating:
              "hsla(0, 0%, 0%, 0) 0px 0px 0px 0px, hsla(0, 0%, 0%, 0) 0px 0px 0px 0px, hsla(0, 0%, 0%, 0.1) 0px 4px 6px -1px, hsla(0, 0%, 0%, 0.1) 0px 2px 4px -1px",
            card: "rgba(50, 50, 100, 0.1) 0px 2px 4px 0px",
          },
          landing: {
            mazeBg: "url('https://cdn.modrinth.com/landing-new/landing-light.webp')",
            mazeGradientBg: "url('https://cdn.modrinth.com/landing-new/landing-lower-light.webp')",
            mazeOuterBg: "linear-gradient(180deg, #f0f0f0 0%, #ffffff 100%)",
            colorHeading: "#000",
            colorSubheading: "#3a3f45",
            transitionGradientStart: "rgba(255, 255, 255, 0)",
            transitionGradientEnd: "#ffffff",
            hoverCardGradient:
              "radial-gradient(50% 50% at 50% 50%, #fff 0%, rgba(204, 204, 204, 0.77) 100%)",
            borderGradient:
              "linear-gradient(to bottom right, rgba(129, 137, 175, 0.75) 0%, rgba(66, 71, 97, 0.34) 100%)",
            borderColor: "rgba(129, 137, 175, 0.55)",
            creatorGradient: "linear-gradient(180deg, #f8f8f8 0%, #f8f8f8 63.19%)",
            blobGradient:
              "radial-gradient(50% 50% at 50% 50%, rgba(255, 255, 255, 0.35) 0%, rgba(255, 255, 255, 0.2695) 100%)",
            blobShadow:
              "2px 2px 12px rgba(0, 0, 0, 0.16), inset 2px 2px 64px rgba(255, 255, 255, 0.45)",
            cardBg: "rgba(255, 255, 255, 0.8)",
            cardShadow: "2px 2px 12px rgba(0, 0, 0, 0.16)",
            blueLabel: "#0098ba",
            blueLabelBg: "rgba(0, 177, 216, 0.15)",
            greenLabel: "#00a936",
            greenLabelBg: "rgba(0, 216, 69, 0.15)",
            rawBg: "#fff",
          },
        },
        dark: {
          secondary: "#96a2b0",
          icon: "#96a2b0",
          // todo finish this
        },
      },
    },
  },
  plugins: [],
};
