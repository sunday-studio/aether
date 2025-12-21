import {
	ModalOverlay,
	type ModalOverlayProps,
	Modal as RACModal,
} from "react-aria-components";
import { cn, tv } from "tailwind-variants";

const overlayStyles = tv({
	base: "absolute top-0 left-0 w-full h-(--page-height) isolate z-20 bg-black/[50%] text-center backdrop-blur-xs",
	variants: {
		isEntering: {
			true: "animate-in fade-in duration-200 ease-out",
		},
		isExiting: {
			true: "animate-out fade-out duration-200 ease-in",
		},
	},
});

const modalStyles = tv({
	base: "p-1 w-full max-w-[min(90vw,450px)] bg-neutral-300 max-h-[calc(var(--visual-viewport-height)*.9)] rounded-xl text-left align-middle shadow-2xl bg-clip-padding",
	variants: {
		isEntering: {
			true: "animate-in zoom-in-105 ease-out duration-200",
		},
		isExiting: {
			true: "animate-out zoom-out-95 ease-in duration-200",
		},
	},
});

export const modalContentStyles = cn(`
  p-4 w-full rounded-lg bg-white text-left align-middle text-neutral-700 shadow-2xl bg-clip-padding bg-white
`);

export function Modal(props: ModalOverlayProps) {
	return (
		<ModalOverlay {...props} className={overlayStyles}>
			<div className="sticky top-0 left-0 w-full h-(--visual-viewport-height) flex items-center justify-center box-border">
				<RACModal {...props} className={modalStyles} />
			</div>
		</ModalOverlay>
	);
}
