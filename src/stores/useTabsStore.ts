import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface Tab {
  id: string;
  paperId: number;
  title: string;
  path: string;
  isActive: boolean;
}

interface TabsState {
  tabs: Tab[];
  activeTabId: string | null;

  addTab: (paperId: number, title: string, path: string) => void;
  removeTab: (tabId: string) => void;
  setActiveTab: (tabId: string) => void;
  closeOtherTabs: (tabId: string) => void;
  closeAllTabs: () => void;
  getTabByPaperId: (paperId: number) => Tab | undefined;
}

export const useTabsStore = create<TabsState>()(
  persist(
    (set, get) => ({
      tabs: [],
      activeTabId: null,

      addTab: (paperId: number, title: string, path: string) => {
        const existingTab = get().tabs.find((t) => t.paperId === paperId);
        if (existingTab) {
          // Tab already exists, just activate it
          set({ activeTabId: existingTab.id });
          return;
        }

        const newTab: Tab = {
          id: `tab-${Date.now()}-${paperId}`,
          paperId,
          title,
          path,
          isActive: true,
        };

        set((state) => ({
          tabs: [...state.tabs.map((t) => ({ ...t, isActive: false })), newTab],
          activeTabId: newTab.id,
        }));
      },

      removeTab: (tabId: string) => {
        set((state) => {
          const newTabs = state.tabs.filter((t) => t.id !== tabId);
          const wasActive = state.activeTabId === tabId;
          const newActiveId =
            wasActive && newTabs.length > 0
              ? newTabs[newTabs.length - 1].id
              : null;

          return {
            tabs: newTabs.map((t) => ({
              ...t,
              isActive: t.id === newActiveId,
            })),
            activeTabId: newActiveId,
          };
        });
      },

      setActiveTab: (tabId: string) => {
        set((state) => ({
          tabs: state.tabs.map((t) => ({ ...t, isActive: t.id === tabId })),
          activeTabId: tabId,
        }));
      },

      closeOtherTabs: (tabId: string) => {
        set((state) => ({
          tabs: state.tabs
            .map((t) => ({ ...t, isActive: t.id === tabId }))
            .filter((t) => t.id === tabId),
          activeTabId: tabId,
        }));
      },

      closeAllTabs: () => {
        set({ tabs: [], activeTabId: null });
      },

      getTabByPaperId: (paperId: number) => {
        return get().tabs.find((t) => t.paperId === paperId);
      },
    }),
    {
      name: "xuan-brain-tabs-storage",
      partialize: (state) => ({
        tabs: state.tabs,
        activeTabId: state.activeTabId,
      }),
    },
  ),
);
