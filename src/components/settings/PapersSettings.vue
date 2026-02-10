<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invokeCommand } from "@/lib/tauri";
import { useI18n } from "@/lib/i18n";

const { t } = useI18n();

// Config data
const config = ref<any>(null);

// GROBID Servers
const grobidServers = ref<any[]>([]);
const grobidDialog = ref(false);
const editingGrobidServer = ref<any>(null);
const grobidForm = ref({
  name: "",
  url: "https://kermitt2-grobid.hf.space",
});
const testingConnection = ref(false);
const testingServerId = ref<string | null>(null);

// Emit event for parent to refresh config
const emit = defineEmits<{
  configUpdated: [];
}>();

onMounted(() => {
  loadConfig();
});

// Load config from backend
async function loadConfig() {
  try {
    const data = await invokeCommand<any>("get_app_config");
    config.value = data;
    if (data?.paper?.grobid?.servers) {
      grobidServers.value = data.paper.grobid.servers;
    }
  } catch (error) {
    console.error("Failed to load config:", error);
  }
}

// Save config to backend
async function saveConfig(newConfig: any) {
  try {
    await invokeCommand("save_app_config", { config: newConfig });
    config.value = newConfig;
    // Update local servers list to refresh UI
    if (newConfig?.paper?.grobid?.servers) {
      grobidServers.value = newConfig.paper.grobid.servers;
    }
    console.info("Configuration saved");
    emit("configUpdated");
  } catch (error) {
    console.error("Failed to save config:", error);
    alert(`保存设置失败: ${error}`);
  }
}

// Test GROBID connection from list
async function testGrobidConnection(server: any) {
  testingServerId.value = server.id;
  testingConnection.value = true;
  try {
    const testUrl = `${server.url.replace(/\/$/, "")}/api/isalive`;
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 5000);
    const response = await fetch(testUrl, { signal: controller.signal });
    clearTimeout(timeoutId);

    if (response.ok) {
      const text = await response.text();
      if (text.trim() === "true") {
        alert("服务器可用");
      } else {
        alert(`服务器响应: ${text}`);
      }
    } else {
      alert(`连接失败: ${response.status}`);
    }
  } catch (error) {
    alert(`连接失败: ${String(error)}`);
  } finally {
    testingConnection.value = false;
    testingServerId.value = null;
  }
}

// Test GROBID connection from dialog
async function testCurrentConnection() {
  if (!grobidForm.value.url) {
    alert("请先输入服务器 URL");
    return;
  }

  testingConnection.value = true;
  try {
    const testUrl = `${grobidForm.value.url.replace(/\/$/, "")}/api/isalive`;
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 5000);
    const response = await fetch(testUrl, { signal: controller.signal });
    clearTimeout(timeoutId);

    if (response.ok) {
      const text = await response.text();
      if (text.trim() === "true") {
        alert("服务器可用");
      } else {
        alert(`服务器响应: ${text}`);
      }
    } else {
      alert(`连接失败 (${response.status}): ${response.statusText}`);
    }
  } catch (error) {
    alert(`连接失败: ${error.message || String(error)}`);
  } finally {
    testingConnection.value = false;
  }
}

// Check if form is valid for testing
const canTestConnection = computed(() => {
  return !!grobidForm.value.url;
});

// Check if a specific server is being tested
function isServerTesting(serverId: string) {
  return testingConnection.value && testingServerId.value === serverId;
}

// GROBID Server functions
function openAddGrobidServer() {
  editingGrobidServer.value = null;
  grobidForm.value = {
    name: "",
    url: "https://kermitt2-grobid.hf.space",
  };
  grobidDialog.value = true;
}

function openEditGrobidServer(server: any) {
  editingGrobidServer.value = server;
  grobidForm.value = { ...server };
  grobidDialog.value = true;
}

async function deleteGrobidServer(id: string) {
  if (!confirm("确定要删除此 GROBID Server 吗？")) return;
  const newServers = grobidServers.value.filter((s) => s.id !== id);
  await saveConfig({
    ...config.value,
    paper: {
      ...config.value?.paper,
      grobid: { ...config.value?.paper?.grobid, servers: newServers },
    },
  });
}

async function setActiveGrobidServer(id: string) {
  const newServers = grobidServers.value.map((s) => ({
    ...s,
    is_active: s.id === id,
  }));
  await saveConfig({
    ...config.value,
    paper: {
      ...config.value?.paper,
      grobid: { ...config.value?.paper?.grobid, servers: newServers },
    },
  });
}

async function saveGrobidServer() {
  if (!grobidForm.value.name.trim()) {
    alert("请输入名称");
    return;
  }

  let newServers = grobidServers.value ? [...grobidServers.value] : [];
  if (editingGrobidServer.value) {
    newServers = newServers.map((s) =>
      s.id === editingGrobidServer.value.id
        ? { ...grobidForm.value, id: s.id }
        : s,
    );
  } else {
    newServers.push({
      ...grobidForm.value,
      id: crypto.randomUUID(),
      is_active: newServers.length === 0,
    });
  }

  await saveConfig({
    ...config.value,
    paper: {
      ...config.value?.paper,
      grobid: { ...config.value?.paper?.grobid, servers: newServers },
    },
  });
  grobidDialog.value = false;
}
</script>

<template>
  <v-card>
    <v-card-title>
      {{ t("settings.grobidServers") }}
      <v-spacer />
      <v-btn
        color="primary"
        size="small"
        variant="outlined"
        @click="openAddGrobidServer"
      >
        <v-icon start>mdi-plus</v-icon>
        {{ t("settings.addServer") }}
      </v-btn>
    </v-card-title>

    <v-card-text>
      <v-list
        v-if="grobidServers.length === 0"
        class="text-center text-grey pa-4"
      >
        {{ t("settings.noServers") }}
      </v-list>
      <v-list v-else>
        <v-list-item v-for="server in grobidServers" :key="server.id">
          <v-list-item-title>
            {{ server.name }}
            <v-chip
              v-if="server.is_active"
              size="x-small"
              color="success"
              class="ml-2"
            >
              {{ t("settings.active") }}
            </v-chip>
          </v-list-item-title>

          <v-list-item-subtitle>
            <div class="text-caption text-grey">
              {{ server.url }}
            </div>
          </v-list-item-subtitle>

          <template #append>
            <v-btn
              icon="mdi-api"
              size="small"
              variant="tonal"
              color="primary"
              :loading="isServerTesting(server.id)"
              :disabled="testingConnection && testingServerId !== server.id"
              @click="testGrobidConnection(server)"
            />
            <v-btn
              v-if="!server.is_active"
              icon="mdi-check"
              size="small"
              variant="text"
              @click="setActiveGrobidServer(server.id)"
            />
            <v-btn
              icon="mdi-pencil"
              size="small"
              variant="text"
              @click="openEditGrobidServer(server)"
            />
            <v-btn
              icon="mdi-delete"
              size="small"
              color="error"
              variant="text"
              @click="deleteGrobidServer(server.id)"
            />
          </template>
        </v-list-item>
      </v-list>
    </v-card-text>
  </v-card>

  <!-- GROBID Server Dialog -->
  <v-dialog v-model="grobidDialog" max-width="500">
    <v-card>
      <v-card-title
        >{{
          editingGrobidServer ? t("dialog.edit") : t("dialog.add")
        }}
        Server</v-card-title
      >

      <v-card-text>
        <v-text-field
          v-model="grobidForm.name"
          :label="t('dialog.categoryName')"
          variant="outlined"
          placeholder="Local GROBID"
        />

        <v-text-field
          v-model="grobidForm.url"
          label="Server URL"
          variant="outlined"
          placeholder="http://localhost:8070"
        />

        <!-- Test Connection Button -->
        <div class="mt-4 d-flex justify-end">
          <v-btn
            variant="outlined"
            color="primary"
            :disabled="!canTestConnection"
            :loading="testingConnection"
            @click="testCurrentConnection"
          >
            <v-icon start>mdi-api</v-icon>
            Test Connection
          </v-btn>
        </div>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn @click="grobidDialog = false">{{ t("dialog.cancel") }}</v-btn>
        <v-btn color="primary" @click="saveGrobidServer">{{
          t("dialog.save")
        }}</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
