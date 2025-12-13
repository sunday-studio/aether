import { create } from "zustand";

type EditorFocusState = {
	focusedId: string | null;
	requestFocus: (id: string) => void;
	clearFocus: () => void;
};

export const useEditorFocusStore = create<EditorFocusState>((set) => ({
	focusedId: null,
	requestFocus: (id) => set({ focusedId: id }),
	clearFocus: () => set({ focusedId: null }),
}));
