<script setup lang="ts">
import { ref, onMounted } from "vue";

// App version info
const appVersion = ref("");
const tauriVersion = ref("");
const vueVersion = ref("");

onMounted(async () => {
  await loadVersionInfo();
});

// Load app version info
async function loadVersionInfo() {
  try {
    const { getVersion } = await import("@tauri-apps/api/app");

    appVersion.value = await getVersion();

    // Get versions from package.json
    const packageRes = await fetch("/package.json");
    if (packageRes.ok) {
      const packageData = await packageRes.json();
      vueVersion.value = packageData.dependencies?.vue || "3.x";
      tauriVersion.value =
        packageData.devDependencies?.["@tauri-apps/api"] || "2.x";
    }
  } catch (error) {
    console.error("Failed to load version info:", error);
    // Fallback values
    tauriVersion.value = "2.x";
    vueVersion.value = "3.x";
  }
}
</script>

<template>
  <v-card>
    <v-card-text>
      <div class="about-section">
        <!-- App Logo/Icon -->
        <div class="about-logo">
          <v-icon size="80" color="primary">mdi-brain</v-icon>
        </div>

        <!-- App Name -->
        <h2 class="text-h4 text-center mt-4 mb-2">璇玑 (xuan-brain)</h2>

        <!-- Description -->
        <p class="text-center text-grey mb-6">AI 驱动的科研文献管理桌面应用</p>

        <v-divider class="my-4" />

        <!-- Version Info -->
        <div class="version-info">
          <v-list>
            <v-list-item>
              <template #prepend>
                <v-icon>mdi-application</v-icon>
              </template>
              <v-list-item-title>应用版本</v-list-item-title>
              <v-list-item-subtitle>{{
                appVersion || "0.1.0"
              }}</v-list-item-subtitle>
            </v-list-item>

            <v-list-item>
              <template #prepend>
                <img
                  src="@/assets/tech-icons/tauri.png"
                  class="tech-icon"
                  alt="Tauri"
                />
              </template>
              <v-list-item-title>Tauri 版本</v-list-item-title>
              <v-list-item-subtitle>{{
                tauriVersion || "2.x"
              }}</v-list-item-subtitle>
            </v-list-item>

            <v-list-item>
              <template #prepend>
                <img
                  src="@/assets/tech-icons/vue.png"
                  class="tech-icon"
                  alt="Vue"
                />
              </template>
              <v-list-item-title>Vue 版本</v-list-item-title>
              <v-list-item-subtitle>{{
                vueVersion || "3.x"
              }}</v-list-item-subtitle>
            </v-list-item>

            <v-list-item>
              <template #prepend>
                <img
                  src="@/assets/tech-icons/vuetify.png"
                  class="tech-icon"
                  alt="Vuetify"
                />
              </template>
              <v-list-item-title>Vuetify 版本</v-list-item-title>
              <v-list-item-subtitle>3.7.0</v-list-item-subtitle>
            </v-list-item>

            <v-list-item>
              <template #prepend>
                <img
                  src="@/assets/tech-icons/sqlite.svg"
                  class="tech-icon"
                  alt="SQLite"
                />
              </template>
              <v-list-item-title>数据库</v-list-item-title>
              <v-list-item-subtitle>SQLite</v-list-item-subtitle>
            </v-list-item>

            <v-list-item>
              <template #prepend>
                <img
                  src="@/assets/tech-icons/seaorm.png"
                  class="tech-icon"
                  alt="SeaORM"
                />
              </template>
              <v-list-item-title>ORM</v-list-item-title>
              <v-list-item-subtitle>SeaORM 2.0</v-list-item-subtitle>
            </v-list-item>

            <v-list-item>
              <template #prepend>
                <v-icon>mdi-license</v-icon>
              </template>
              <v-list-item-title>License</v-list-item-title>
              <v-list-item-subtitle>MIT</v-list-item-subtitle>
            </v-list-item>
          </v-list>
        </div>

        <v-divider class="my-4" />

        <!-- Links -->
        <div class="about-links">
          <v-btn
            variant="outlined"
            color="primary"
            href="https://github.com/yourusername/xuan-brain"
            target="_blank"
            class="mr-2"
          >
            <v-icon start>mdi-github</v-icon>
            GitHub
          </v-btn>
          <v-btn
            variant="outlined"
            color="primary"
            href="https://github.com/yourusername/xuan-brain/issues"
            target="_blank"
          >
            <v-icon start>mdi-bug-report</v-icon>
            Report Issue
          </v-btn>
        </div>
      </div>
    </v-card-text>
  </v-card>
</template>

<style scoped>
.about-section {
  text-align: center;
}

.about-logo {
  display: flex;
  justify-content: center;
  align-items: center;
}

.version-info {
  text-align: left;
  max-width: 400px;
  margin: 0 auto;
}

.tech-icon {
  width: 24px;
  height: 24px;
  object-fit: contain;
}

.about-links {
  display: flex;
  justify-content: center;
  flex-wrap: wrap;
  gap: 8px;
}
</style>
