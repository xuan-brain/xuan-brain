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

export const useAppStore = defineStore(
  "app",
  () => {
    // State
    const isDark = ref(true);
    const accentColor = ref("#3b82f6");
    const selectedDocument = ref<Document | null>(null);

    // Getters
    const currentTheme = computed(() => (isDark.value ? "dark" : "light"));

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

    return {
      isDark,
      accentColor,
      selectedDocument,
      currentTheme,
      toggleTheme,
      setTheme,
      setAccentColor,
      setSelectedDocument,
    };
  },
  {
    persist: {
      key: "xuan-brain-app-storage",
      storage: localStorage,
      pick: ["isDark", "accentColor"],
    },
  },
);
