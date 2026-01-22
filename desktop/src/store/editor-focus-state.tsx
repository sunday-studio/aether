import { proxy, useSnapshot } from "valtio";

type EditorFocusState = {
	focusedId: string | null;
	requestFocus: (id: string) => void;
	clearFocus: () => void;
};

const editorFocusState = proxy<{ focusedId: string | null }>({
	focusedId: null,
});

function requestFocus(id: string) {
	editorFocusState.focusedId = id;
}

function clearFocus() {
	editorFocusState.focusedId = null;
}

export function useEditorFocusStore(): EditorFocusState {
	const snap = useSnapshot(editorFocusState);
	return {
		focusedId: snap.focusedId,
		requestFocus,
		clearFocus,
	};
}
