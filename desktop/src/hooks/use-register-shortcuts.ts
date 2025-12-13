import { useHotkeys } from "react-hotkeys-hook";
import { useNavigate } from "react-router";
import { Key } from "ts-key-enum";

// useHotkeys(`${Key.Meta}+d`, () => appState.toggleSidebarOpenState());

export const useRegisterShortcuts = () => {
	const navigate = useNavigate();
	console.log("useRegisterShortcuts");
};
