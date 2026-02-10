import { createApp } from "vue";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import { createVuetify } from "vuetify";
import * as components from "vuetify/components";
import * as directives from "vuetify/directives";
import "vuetify/styles";
import "@mdi/font/css/materialdesignicons.css";
import "@/assets/styles/main.css";

import App from "./App.vue";
import { i18n } from "./lib/i18n";

const app = createApp(App);

// Pinia with persist plugin
const pinia = createPinia();
pinia.use(piniaPluginPersistedstate);
app.use(pinia);

// Vuetify
const vuetify = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: "dark",
    themes: {
      dark: {
        colors: {
          primary: "#90caf9",
          surface: "#1f1f1f",
          background: "#141414",
        },
      },
      light: {
        colors: {
          primary: "#1976d2",
          surface: "#ffffff",
          background: "#f5f5f5",
        },
      },
    },
  },
});
app.use(vuetify);

// Vue Router
import router from "./router";
app.use(router);

// i18n
app.use(i18n);

app.mount("#app");
