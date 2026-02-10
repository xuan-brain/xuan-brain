import { createApp } from "vue";
import { createVuetify } from "vuetify";
import * as components from "vuetify/components";
import * as directives from "vuetify/directives";
import "vuetify/styles";
import PDFViewerApp from "./PDFViewerApp.vue";

const app = createApp(PDFViewerApp);

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
    },
  },
});

app.use(vuetify);

app.mount("#pdf-app");
