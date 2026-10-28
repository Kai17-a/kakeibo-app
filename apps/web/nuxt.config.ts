import tailwindcss from "@tailwindcss/vite";
export default defineNuxtConfig({
  vite: { plugins: [tailwindcss()] },

  // Served as a static SPA bundle by the Rust API (apps/api); all data is
  // fetched client-side, so no Node/Nitro server runtime is needed in production.
  ssr: false,

  devtools: {
    enabled: true,
  },

  css: ["~/assets/css/main.css"],

  compatibilityDate: "2026-06-30",

  nitro: {
    devProxy: {
      "/api": { target: "http://localhost:8000/api", changeOrigin: true },
    },
  },
});
