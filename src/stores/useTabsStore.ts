import { defineStore } from "pinia";
import { ref } from "vue";

export interface Tab {
  id: string;
  paperId: number;
  title: string;
  path: string;
  isActive: boolean;
}

export const useTabsStore = defineStore(
  "tabs",
  () => {
    const tabs = ref<Tab[]>([]);
    const activeTabId = ref<string | null>(null);

    function addTab(paperId: number, title: string, path: string) {
      const existingTab = tabs.value.find((t) => t.paperId === paperId);
      if (existingTab) {
        setActiveTab(existingTab.id);
        return;
      }

      const newTab: Tab = {
        id: `tab-${Date.now()}`,
        paperId,
        title,
        path,
        isActive: true,
      };

      // Deactivate all other tabs
      tabs.value.forEach((t) => (t.isActive = false));
      tabs.value.push(newTab);
      activeTabId.value = newTab.id;
    }

    function removeTab(tabId: string) {
      const index = tabs.value.findIndex((t) => t.id === tabId);
      if (index === -1) return;

      const wasActive = tabs.value[index].isActive;
      tabs.value.splice(index, 1);

      if (wasActive && tabs.value.length > 0) {
        const newIndex = Math.min(index, tabs.value.length - 1);
        setActiveTab(tabs.value[newIndex].id);
      } else if (tabs.value.length === 0) {
        activeTabId.value = null;
      }
    }

    function setActiveTab(tabId: string) {
      tabs.value.forEach((t) => (t.isActive = t.id === tabId));
      activeTabId.value = tabId;
    }

    function closeOtherTabs(tabId: string) {
      tabs.value = tabs.value.filter((t) => t.id === tabId);
      setActiveTab(tabId);
    }

    function closeAllTabs() {
      tabs.value = [];
      activeTabId.value = null;
    }

    return {
      tabs,
      activeTabId,
      addTab,
      removeTab,
      setActiveTab,
      closeOtherTabs,
      closeAllTabs,
    };
  },
  {
    persist: {
      key: "xuan-brain-tabs-storage",
      storage: localStorage,
    },
  },
);
