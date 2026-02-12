import { createPinia } from 'pinia';
import { createApp, defineAsyncComponent } from 'vue';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import 'vuetify/styles';

// PDF Viewer App
const PDFViewerApp = {
  components: {
    PDFViewer: defineAsyncComponent(() => import('./components/pdf/PDFViewer.vue')),
  },
  template: `
    <v-app>
      <PDFViewer />
    </v-app>
  `,
};

const vuetify = createVuetify({
  components,
  directives,
});

const pinia = createPinia();
const app = createApp(PDFViewerApp);
app.use(pinia);
app.use(vuetify);
app.mount('#pdf-root');
