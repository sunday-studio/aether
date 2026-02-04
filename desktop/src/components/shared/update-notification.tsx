import { listen } from "@tauri-apps/api/event";
import { DownloadIcon, XIcon } from "lucide-react";
import { useEffect } from "react";
import { useNavigate } from "react-router";
import { toast } from "sonner";
import type { UpdateInfo } from "~/types/updater";

/**
 * Component that listens for update events and shows toast notifications.
 * Should be placed near the root of the app.
 */
export function UpdateNotificationListener() {
	const navigate = useNavigate();

	// useEffect(() => {
	// 	const unlisten = listen<UpdateInfo>("update-available", (event) => {
	// 		const info = event.payload;

	// 		toast.custom(
	// 			(id) => (
	// 				<UpdateToast
	// 					id={id}
	// 					info={info}
	// 					onViewChanges={() => {
	// 						toast.dismiss(id);
	// 						navigate("/settings");
	// 					}}
	// 					onDismiss={() => toast.dismiss(id)}
	// 				/>
	// 			),
	// 			{
	// 				duration: 15000,
	// 				position: "bottom-right",
	// 			},
	// 		);
	// 	});

	// 	return () => {
	// 		unlisten.then((fn) => fn());
	// 	};
	// }, [navigate]);

	return null;
}

interface UpdateToastProps {
	id: string | number;
	info: UpdateInfo;
	onViewChanges: () => void;
	onDismiss: () => void;
}

function UpdateToast({ info, onViewChanges, onDismiss }: UpdateToastProps) {
	return (
		<div className="flex flex-col rounded-lg bg-neutral-900 shadow-lg border border-neutral-700 p-4 min-w-[300px]">
			<div className="flex items-start justify-between gap-3">
				<div className="flex items-center gap-2">
					<DownloadIcon className="size-4 text-blue-400" />
					<p className="text-sm font-medium text-neutral-100">
						Update Available
					</p>
				</div>
				<button
					type="button"
					onClick={onDismiss}
					className="text-neutral-500 hover:text-neutral-300 transition-colors"
				>
					<XIcon className="size-4" />
				</button>
			</div>

			<p className="text-sm text-neutral-400 mt-1">
				Version {info.latestVersion} is ready to download
			</p>

			<div className="flex items-center gap-2 mt-3">
				<button
					type="button"
					onClick={onViewChanges}
					className="flex-1 text-sm px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-md transition-colors"
				>
					View Changes
				</button>
				<button
					type="button"
					onClick={onDismiss}
					className="text-sm px-3 py-1.5 text-neutral-400 hover:text-neutral-200 transition-colors"
				>
					Later
				</button>
			</div>
		</div>
	);
}
