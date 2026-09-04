import { defineStore } from "pinia";
import {
  clearHistory,
  deleteHistoryItem,
  listHistory,
  type HistoryItemDto,
} from "../services/history";

export const useHistoryStore = defineStore("history", {
  state: () => ({
    items: [] as HistoryItemDto[],
    query: "",
    loading: false,
  }),
  actions: {
    async load() {
      this.loading = true;
      try {
        this.items = await listHistory(this.query);
      } finally {
        this.loading = false;
      }
    },
    async remove(id: number) {
      await deleteHistoryItem(id);
      this.items = this.items.filter((item) => item.id !== id);
    },
    async clear() {
      await clearHistory();
      this.items = [];
    },
  },
});
