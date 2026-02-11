import { createApp } from "vue";
import { createPinia } from "pinia";
import vuetify from "@/plugins/vuetify";
import App from "./App.vue";

// PDF Viewer App
const PDFViewerApp = {
  components: {
    PDFViewer: () => import("./components/pdf/PDFViewer.vue"),
  },
  template: `
    <v-app>
      <PDFViewer />
    </v-app>
  `,
};

const pinia = createPinia();
const app = createApp(PDFViewerApp);
app.use(pinia);
app.use(vuetify);
app.mount("#pdf-root");
