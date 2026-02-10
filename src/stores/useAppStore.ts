import { defineStore } from "pinia";
import { ref, computed } from "vue";

export interface Document {
  id: number;
  title: string;
  authors: string[];
  year: number;
  abstract?: string;
  keywords?: string[];
  fileType?: string;
  fileSize?: string;
  addedDate?: string;
  tags?: Tag[];
}

export interface Tag {
  id: number;
  name: string;
  color: string;
}

export interface LLMProvider {
  id: string;
  name: string;
  base_url: string;
  api_key: string;
  model_name: string;
  is_default?: boolean;
}

export interface GrobidServer {
  id: string;
  name: string;
  url: string;
  is_default?: boolean;
}

export const useAppStore = defineStore(
  "app",
  () => {
    // State
    const isDark = ref(true);
    const accentColor = ref("#3b82f6");
    const selectedDocument = ref<Document | null>(null);
    const selectedLLMProvider = ref<string | null>(null);
    const selectedGrobidServer = ref<string | null>(null);
    const llmProviders = ref<LLMProvider[]>([]);
    const grobidServers = ref<GrobidServer[]>([]);

    // Getters
    const currentTheme = computed(() => (isDark.value ? "dark" : "light"));
    const currentLLMProvider = computed(() =>
      llmProviders.value.find((p) => p.id === selectedLLMProvider.value),
    );
    const currentGrobidServer = computed(() =>
      grobidServers.value.find((s) => s.id === selectedGrobidServer.value),
    );

    // Actions
    function toggleTheme() {
      isDark.value = !isDark.value;
    }

    function setTheme(value: boolean) {
      isDark.value = value;
    }

    function setAccentColor(color: string) {
      accentColor.value = color;
    }

    function setSelectedDocument(doc: Document | null) {
      selectedDocument.value = doc;
    }

    function setSelectedLLMProvider(providerId: string | null) {
      selectedLLMProvider.value = providerId;
    }

    function setSelectedGrobidServer(serverId: string | null) {
      selectedGrobidServer.value = serverId;
    }

    function setLLMProviders(providers: LLMProvider[]) {
      llmProviders.value = providers;
      // Auto-select default or first provider
      if (!selectedLLMProvider.value && providers.length > 0) {
        const defaultProvider = providers.find((p) => p.is_default);
        selectedLLMProvider.value = defaultProvider
          ? defaultProvider.id
          : providers[0].id;
      }
    }

    function setGrobidServers(servers: GrobidServer[]) {
      grobidServers.value = servers;
      // Auto-select default or first server
      if (!selectedGrobidServer.value && servers.length > 0) {
        const defaultServer = servers.find((s) => s.is_default);
        selectedGrobidServer.value = defaultServer
          ? defaultServer.id
          : servers[0].id;
      }
    }

    return {
      isDark,
      accentColor,
      selectedDocument,
      selectedLLMProvider,
      selectedGrobidServer,
      llmProviders,
      grobidServers,
      currentTheme,
      currentLLMProvider,
      currentGrobidServer,
      toggleTheme,
      setTheme,
      setAccentColor,
      setSelectedDocument,
      setSelectedLLMProvider,
      setSelectedGrobidServer,
      setLLMProviders,
      setGrobidServers,
    };
  },
  {
    persist: {
      key: "xuan-brain-app-storage",
      storage: localStorage,
      pick: [
        "isDark",
        "accentColor",
        "selectedLLMProvider",
        "selectedGrobidServer",
      ],
    },
  },
);
