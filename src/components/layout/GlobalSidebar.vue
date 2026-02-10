<script setup lang="ts">
import { useRouter } from "vue-router";
import { useI18n } from "@/lib/i18n";

const router = useRouter();
const { t } = useI18n();

const menuItems = [
  { icon: "mdi-file-document", value: "papers", title: "navigation.papers" },
  { icon: "mdi-content-cut", value: "clips", title: "navigation.clips" },
  { icon: "mdi-pencil", value: "writing", title: "navigation.writing" },
  { icon: "mdi-rss", value: "subscriptions", title: "navigation.subscriptions" },
];

function navigateTo(path: string) {
  router.push(path);
}
</script>

<template>
  <v-navigation-drawer permanent rail width="72" class="global-sidebar">
    <v-list density="compact">
      <!-- User avatar placeholder -->
      <v-list-item class="user-avatar" rounded="lg">
        <template #prepend>
          <v-avatar color="primary">
            <span class="text-h6">U</span>
          </v-avatar>
        </template>
      </v-list-item>

      <v-divider class="my-2" />

      <!-- Navigation menu -->
      <v-list-item
        v-for="item in menuItems"
        :key="item.value"
        :prepend-icon="item.icon"
        :value="item.value"
        :title="t(item.title)"
        rounded="lg"
        @click="navigateTo(`/${item.value}`)"
      />
    </v-list>

    <template #append>
      <v-list density="compact">
        <v-list-item
          prepend-icon="mdi-cog"
          value="settings"
          :title="t('navigation.settings')"
          rounded="lg"
          @click="navigateTo('/settings')"
        />
      </v-list>
    </template>
  </v-navigation-drawer>
</template>

<style scoped>
.global-sidebar {
  border-right: 1px solid rgba(255, 255, 255, 0.12);
}

.user-avatar {
  margin: 8px 0;
}
</style>
