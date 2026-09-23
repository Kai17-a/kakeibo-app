// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: [
    '@nuxt/eslint',
    '@nuxt/ui'
  ],

  devtools: {
    enabled: true
  },

  css: ['~/assets/css/main.css'],

  // Fonts are bundled locally via @fontsource; no remote font downloads.
  ui: { fonts: false },

  routeRules: {
    '/': { prerender: true }
  },

  compatibilityDate: '2026-06-30',

  nitro: {
    devProxy: {
      '/api': { target: 'http://localhost:8000/api', changeOrigin: true }
    }
  },

  eslint: {
    config: {
      stylistic: {
        commaDangle: 'never',
        braceStyle: '1tbs'
      }
    }
  }
})
