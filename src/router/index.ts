import { createRouter, createWebHistory } from "vue-router";
import MainLayout from "@/layouts/MainLayout.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      component: MainLayout,
      redirect: "/papers",
      children: [
        {
          path: "papers",
          name: "papers",
          component: () => import("@/pages/PapersPage.vue"),
        },
        {
          path: "papers/:paperId",
          name: "paper-reader",
          component: () => import("@/pages/PaperReaderPage.vue"),
        },
        {
          path: "clips",
          name: "clips",
          component: () => import("@/pages/ClipsPage.vue"),
        },
        {
          path: "writing",
          name: "writing",
          component: () => import("@/pages/WritingPage.vue"),
        },
        {
          path: "subscriptions",
          name: "subscriptions",
          component: () => import("@/pages/SubscriptionPage.vue"),
        },
        {
          path: "settings",
          name: "settings",
          component: () => import("@/pages/SettingsPage.vue"),
        },
      ],
    },
  ],
});

export default router;
