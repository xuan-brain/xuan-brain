import { createApp, watch } from "vue";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import { createVuetify } from "vuetify";
import * as components from "vuetify/components";
import * as directives from "vuetify/directives";
import "vuetify/styles";
import "@mdi/font/css/materialdesignicons.css";
import "@/assets/styles/main.css";

// VxeTable
import VxeTable from "vxe-table";
import "vxe-table/lib/style.css";

import App from "./App.vue";
import { i18n } from "./lib/i18n";
import { useAppStore } from "./stores/useAppStore";

const app = createApp(App);

// Register VxeTable globally
app.use(VxeTable);

// Pinia with persist plugin
const pinia = createPinia();
pinia.use(piniaPluginPersistedstate);
app.use(pinia);

// Get app store for theme
const appStore = useAppStore();

// Vuetify
const vuetify = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: "dark",
    themes: {
      dark: {
        colors: {
          primary: appStore.accentColor,
          "on-primary": "#1a2332",
          surface: "#1f1f1f",
          background: "#141414",
        },
      },
      light: {
        colors: {
          primary: appStore.accentColor,
          surface: "#ffffff",
          background: "#f5f5f5",
        },
      },
    },
  },
  // Global density and sizing defaults
  defaults: {
    VBtn: {
      density: "compact",
      size: "small",
    },
    VTextField: {
      density: "compact",
      variant: "outlined",
    },
    VSelect: {
      density: "compact",
      variant: "outlined",
    },
    VTextarea: {
      density: "compact",
      variant: "outlined",
    },
    VListItem: {
      density: "compact",
    },
    VCard: {
      density: "compact",
    },
    VChip: {
      density: "comfortable",
    },
    VTooltip: {
      density: "compact",
    },
    VMenu: {
      density: "compact",
    },
  },
  // Disable all transitions globally
  display: {
    mobileBreakpoint: "md",
    thresholds: {
      xs: 0,
      sm: 340,
      md: 540,
      lg: 800,
      xl: 1280,
    },
  },
});
app.use(vuetify);

// Watch accent color changes and update Vuetify theme
watch(
  () => appStore.accentColor,
  (newColor) => {
    vuetify.theme.themes.value.dark.colors.primary = newColor;
    vuetify.theme.themes.value.light.colors.primary = newColor;
  },
  { immediate: false },
);

// Vue Router
import router from "./router";
app.use(router);

// i18n
app.use(i18n);

app.mount("#app");
