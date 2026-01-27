import { create } from "zustand";
import { persist } from "zustand/middleware";

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
  tags?: { id: number; name: string; color: string }[];
}

interface AppState {
  isDark: boolean;
  accentColor: string;
  selectedDocument: Document | null;
  toggleTheme: () => void;
  setTheme: (isDark: boolean) => void;
  setAccentColor: (color: string) => void;
  setSelectedDocument: (doc: Document | null) => void;
}

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      isDark: true, // Default to dark
      accentColor: "#3b82f6", // Default accent color
      selectedDocument: null,

      toggleTheme: () => set((state) => ({ isDark: !state.isDark })),
      setTheme: (isDark) => set({ isDark }),
      setAccentColor: (accentColor) => set({ accentColor }),
      setSelectedDocument: (selectedDocument) => set({ selectedDocument }),
    }),
    {
      name: "app-storage",
      partialize: (state) => ({
        isDark: state.isDark,
        accentColor: state.accentColor,
      }), // Only persist theme settings
    },
  ),
);
