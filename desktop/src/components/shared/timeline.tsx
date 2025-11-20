import { ReactNode } from "react";

import { clsx } from "clsx";
// import { format } from "date-fns";

export function Root({
	children,
	className,
}: {
	children: ReactNode;
	className?: string;
}) {
	return (
		<ol
			role="list"
			className={clsx("relative", className)}
			aria-label="Timeline"
		>
			{children}
		</ol>
	);
}

const Item = ({ children }: { children: ReactNode }) => {
	return (
		<div className="group relative flex min-h-14 gap-4 items-start">
			{children}
		</div>
	);
};

const Indicator = ({
	children,
	className,
	onClick = () => {},
}: {
	children?: ReactNode;
	className?: string;
	onClick?: () => void;
}) => {
	// The key: Indicator must NOT rely on vertical alignment from parent
	return (
		<div className="relative flex flex-col items-center self-stretch justify-start min-h-14">
			<div
				aria-hidden="true"
				className="flex h-full w-7 flex-col items-center justify-start"
			>
				<div
					role="button"
					onClick={onClick}
					style={{
						clipPath:
							"polygon(50% 0%, 90% 20%, 100% 60%, 75% 100%, 25% 100%, 0% 60%, 10% 20%)", // custom shape
					}}
					className={clsx(
						"flex h-4.5 w-4.5 items-center justify-center bg-neutral-200",
						className,
					)}
				>
					{children}
				</div>
				<div className="my-1 w-[2px] flex-1 shrink-0 bg-neutral-200 group-last:hidden" />
			</div>
		</div>
	);
};

const Content = ({
	children,
	timeline,
	className,
}: {
	children: ReactNode;
	timeline?: string;
	className?: string;
}) => {
	return (
		<div className={clsx("flex w-full gap-10 ", className)}>{children}</div>
	);
};

export const Timeline = Object.assign(Root, {
	Item,
	Indicator,
	Content,
});
