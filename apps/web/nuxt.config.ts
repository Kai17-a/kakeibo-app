// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: ["@nuxt/ui"],

  // Served as a static SPA bundle by the Rust API (apps/api); all data is
  // fetched client-side, so no Node/Nitro server runtime is needed in production.
  ssr: false,

  devtools: {
    enabled: true,
  },

  css: ["~/assets/css/main.css"],

  // Fonts are bundled locally via @fontsource; no remote font downloads.
  ui: { fonts: false },

  compatibilityDate: "2026-06-30",

  nitro: {
    devProxy: {
      "/api": { target: "http://localhost:8000/api", changeOrigin: true },
    },
  },
});
