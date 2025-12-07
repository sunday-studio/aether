import { toast as sonnerToast } from "sonner";

interface ToastProps {
	id: string | number;
	title: string;
	// description?: string;
	// button: {
	// 	label: string;
	// 	onClick: () => void;
	// };
}

export function showToast(toast: Omit<ToastProps, "id">) {
	return sonnerToast.custom((id) => (
		<ToastComponent id={id} title={toast.title} />
	));
}

function ToastComponent(props: ToastProps) {
	const { title } = props;

	return (
		<div className="flex rounded-full bg-neutral-900 shadow-md inset-ring-2 inset-ring-neutral-600 items-center w-full py-2 px-5">
			<div className="flex flex-1 items-center w-full">
				<div className="w-full">
					<p className="text-sm text-neutral-200 select-none pointer-events-none">
						{title}
					</p>
				</div>
			</div>
		</div>
	);
}
