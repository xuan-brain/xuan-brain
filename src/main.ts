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

// Register VxeTable plugins (optional, for additional features)
// Note: loading functionality is built-in, no separate plugin needed for v4

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

// Watch theme changes and update Vuetify and VxeTable themes
watch(
  () => appStore.isDark,
  (isDark) => {
    // Update Vuetify theme
    vuetify.theme.global.name.value = isDark ? "dark" : "light";
    // Update VxeTable theme (v4.6.17+)
    if (VxeTable.setTheme) {
      VxeTable.setTheme(isDark ? "dark" : "light");
    }
    // Update VxeTable CSS variables for better integration
    updateVxeTableTheme(isDark);
  },
  { immediate: true },
);

// Helper function to update VxeTable CSS variables
function updateVxeTableTheme(isDark: boolean) {
  const root = document.documentElement;
  if (isDark) {
    // Dark theme
    root.style.setProperty("--vxe-font-color", "rgba(255, 255, 255, 0.87)");
    root.style.setProperty("--vxe-table-body-background-color", "#1f1f1f");
    root.style.setProperty(
      "--vxe-table-header-font-color",
      "rgba(255, 255, 255, 0.7)",
    );
    root.style.setProperty("--vxe-table-header-background-color", "#141414");
    root.style.setProperty("--vxe-toolbar-background-color", "#141414");
    root.style.setProperty("--vxe-button-default-background-color", "#2a2a2a");
    root.style.setProperty("--vxe-input-border-color", "#424242");
    root.style.setProperty("--vxe-table-border-color", "#424242");
    root.style.setProperty(
      "--vxe-table-row-hover-background-color",
      "rgba(255, 255, 255, 0.04)",
    );
    root.style.setProperty(
      "--vxe-table-row-striped-background-color",
      "rgba(255, 255, 255, 0.02)",
    );
    root.style.setProperty("--vxe-table-popup-border-color", "#424242");
    root.style.setProperty(
      "--vxe-loading-background-color",
      "rgba(0, 0, 0, 0.5)",
    );
  } else {
    // Light theme
    root.style.setProperty("--vxe-font-color", "#606266");
    root.style.setProperty("--vxe-table-body-background-color", "#ffffff");
    root.style.setProperty("--vxe-table-header-font-color", "#606266");
    root.style.setProperty("--vxe-table-header-background-color", "#f8f8f9");
    root.style.setProperty("--vxe-toolbar-background-color", "#fff");
    root.style.setProperty("--vxe-button-default-background-color", "#fff");
    root.style.setProperty("--vxe-input-border-color", "#dcdfe6");
    root.style.setProperty("--vxe-table-border-color", "#e8eaec");
    root.style.setProperty("--vxe-table-row-hover-background-color", "#f5f7fa");
    root.style.setProperty(
      "--vxe-table-row-striped-background-color",
      "#fafafa",
    );
    root.style.setProperty("--vxe-table-popup-border-color", "#DADCE0");
    root.style.setProperty(
      "--vxe-loading-background-color",
      "rgba(255, 255, 255, 0.5)",
    );
  }
}

// Vue Router
import router from "./router";
app.use(router);

// i18n
app.use(i18n);

app.mount("#app");
