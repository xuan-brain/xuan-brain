<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invokeCommand } from "@/lib/tauri";
import { useI18n } from "@/lib/i18n";

const { t } = useI18n();

// Config data
const config = ref<any>(null);

// LLM Providers
const llmProviders = ref<any[]>([]);
const llmDialog = ref(false);
const editingLlmProvider = ref<any>(null);
const llmForm = ref({
  name: "",
  base_url: "https://api.openai.com/v1",
  api_key: "",
  model_name: "",
});
const testingConnection = ref(false);
const testingDialog = ref(false);

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
    if (data?.system?.llm_providers) {
      llmProviders.value = data.system.llm_providers;
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
    // Update local providers list to refresh UI
    if (newConfig?.system?.llm_providers) {
      llmProviders.value = newConfig.system.llm_providers;
    }
    console.info("Configuration saved");
    emit("configUpdated");
  } catch (error) {
    console.error("Failed to save config:", error);
    alert(`保存设置失败: ${error}`);
  }
}

// Test LLM connection from list
async function testLlmConnection(provider: any) {
  testingConnection.value = true;
  try {
    const response = await fetch(`${provider.base_url}/chat/completions`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${provider.api_key}`,
      },
      body: JSON.stringify({
        model: provider.model_name,
        messages: [{ role: "user", content: "Hello" }],
        max_tokens: 10,
      }),
      signal: AbortSignal.timeout(10000),
    });

    if (response.ok) {
      const data = await response.json();
      if (data.choices && data.choices.length > 0) {
        alert("连接成功！");
      } else {
        alert(
          `连接成功但返回格式异常: ${JSON.stringify(data).substring(0, 100)}`,
        );
      }
    } else {
      const errorText = await response.text();
      alert(`连接失败: ${response.status} - ${errorText.substring(0, 100)}`);
    }
  } catch (error: any) {
    if (error.name === "AbortError") {
      alert("连接超时");
    } else {
      alert(`连接失败: ${error.message || String(error)}`);
    }
  } finally {
    testingConnection.value = false;
  }
}

// Test LLM connection from dialog
async function testCurrentConnection() {
  if (
    !llmForm.value.base_url ||
    !llmForm.value.api_key ||
    !llmForm.value.model_name
  ) {
    alert("请先填写 Base URL、API Key 和 Model Name");
    return;
  }

  testingConnection.value = true;
  try {
    const response = await fetch(`${llmForm.value.base_url}/chat/completions`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${llmForm.value.api_key}`,
      },
      body: JSON.stringify({
        model: llmForm.value.model_name,
        messages: [{ role: "user", content: "Hello" }],
        max_tokens: 10,
      }),
      signal: AbortSignal.timeout(10000),
    });

    if (response.ok) {
      const data = await response.json();
      if (data.choices && data.choices.length > 0) {
        alert("连接成功！");
      } else {
        alert(
          `连接成功但返回格式异常: ${JSON.stringify(data).substring(0, 100)}`,
        );
      }
    } else {
      const errorText = await response.text();
      alert(`连接失败 (${response.status}): ${errorText.substring(0, 200)}`);
    }
  } catch (error: any) {
    if (error.name === "AbortError") {
      alert("连接超时");
    } else {
      alert(`连接失败: ${error.message || String(error)}`);
    }
  } finally {
    testingConnection.value = false;
  }
}

// Check if form is valid for testing
const canTestConnection = computed(() => {
  return !!(
    llmForm.value.base_url &&
    llmForm.value.api_key &&
    llmForm.value.model_name
  );
});

// LLM Provider functions
function openAddLlmProvider() {
  editingLlmProvider.value = null;
  llmForm.value = {
    name: "",
    base_url: "https://api.openai.com/v1",
    api_key: "",
    model_name: "",
  };
  llmDialog.value = true;
}

function openEditLlmProvider(provider: any) {
  editingLlmProvider.value = provider;
  llmForm.value = { ...provider };
  llmDialog.value = true;
}

async function deleteLlmProvider(id: string) {
  if (!confirm("确定要删除此 LLM Provider 吗？")) return;
  const newProviders = llmProviders.value.filter((p) => p.id !== id);
  await saveConfig({
    ...config.value,
    system: { ...config.value?.system, llm_providers: newProviders },
  });
}

async function setDefaultLlmProvider(id: string) {
  const newProviders = llmProviders.value.map((p) => ({
    ...p,
    is_default: p.id === id,
  }));
  await saveConfig({
    ...config.value,
    system: { ...config.value?.system, llm_providers: newProviders },
  });
}

async function saveLlmProvider() {
  if (!llmForm.value.name.trim()) {
    alert("请输入名称");
    return;
  }

  let newProviders = [...llmProviders.value];
  if (editingLlmProvider.value) {
    newProviders = newProviders.map((p) =>
      p.id === editingLlmProvider.value.id ? { ...llmForm.value, id: p.id } : p,
    );
  } else {
    newProviders.push({
      ...llmForm.value,
      id: crypto.randomUUID(),
      is_default: newProviders.length === 0,
    });
  }

  await saveConfig({
    ...config.value,
    system: { ...config.value?.system, llm_providers: newProviders },
  });
  llmDialog.value = false;
}
</script>

<template>
  <v-card>
    <v-card-title>
      {{ t("settings.llmProviders") }}
      <v-spacer />
      <v-btn
        color="primary"
        size="small"
        variant="outlined"
        @click="openAddLlmProvider"
      >
        <v-icon start>mdi-plus</v-icon>
        {{ t("settings.addProvider") }}
      </v-btn>
    </v-card-title>

    <v-card-text>
      <v-list
        v-if="llmProviders.length === 0"
        class="text-center text-grey pa-4"
      >
        {{ t("settings.noProviders") }}
      </v-list>
      <v-list v-else>
        <v-list-item v-for="provider in llmProviders" :key="provider.id">
          <v-list-item-title>
            {{ provider.name }}
            <v-chip
              v-if="provider.is_default"
              size="x-small"
              color="success"
              class="ml-2"
            >
              {{ t("settings.default") }}
            </v-chip>
          </v-list-item-title>

          <v-list-item-subtitle>
            <div>{{ t("settings.model") }}: {{ provider.model_name }}</div>
            <div class="text-caption text-grey">
              {{ provider.base_url }}
            </div>
          </v-list-item-subtitle>

          <template #append>
            <v-btn
              icon="mdi-api"
              size="small"
              variant="tonal"
              color="primary"
              :loading="testingConnection"
              @click="testLlmConnection(provider)"
            />
            <v-btn
              v-if="!provider.is_default"
              icon="mdi-check"
              size="small"
              variant="text"
              @click="setDefaultLlmProvider(provider.id)"
            />
            <v-btn
              icon="mdi-pencil"
              size="small"
              variant="text"
              @click="openEditLlmProvider(provider)"
            />
            <v-btn
              icon="mdi-delete"
              size="small"
              color="error"
              variant="text"
              @click="deleteLlmProvider(provider.id)"
            />
          </template>
        </v-list-item>
      </v-list>
    </v-card-text>
  </v-card>

  <!-- LLM Provider Dialog -->
  <v-dialog v-model="llmDialog" max-width="500">
    <v-card>
      <v-card-title
        >{{
          editingLlmProvider ? t("dialog.edit") : t("dialog.add")
        }}
        Provider</v-card-title
      >

      <v-card-text>
        <v-text-field
          v-model="llmForm.name"
          :label="t('dialog.categoryName')"
          variant="outlined"
          placeholder="e.g. OpenAI, DeepSeek"
        />

        <v-text-field
          v-model="llmForm.base_url"
          label="Base URL"
          variant="outlined"
          placeholder="https://api.openai.com/v1"
        />

        <v-text-field
          v-model="llmForm.api_key"
          label="API Key"
          type="password"
          variant="outlined"
          placeholder="sk-..."
        />

        <v-text-field
          v-model="llmForm.model_name"
          :label="t('settings.model')"
          variant="outlined"
          placeholder="gpt-4o, deepseek-chat"
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
        <v-btn @click="llmDialog = false">{{ t("dialog.cancel") }}</v-btn>
        <v-btn color="primary" @click="saveLlmProvider">{{
          t("dialog.save")
        }}</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
