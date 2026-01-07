import { Trash } from "lucide-react";
import { motion, useMotionValue, useSpring } from "motion/react";
import { useRef, useState } from "react";
import { Tooltip } from "~/components/shared/tooltip";
import { cn } from "~/utils/cn";
import { useOptimisticDeleteTask } from "../../use-optimistic-update-task";
import { TaskActionButton } from "./task-shared-components";

const HOLD_DURATION = 1000;
const SCALE_TARGET = 1.5;

export const TaskItemDelete = ({ taskId }: { taskId: string }) => {
	const [isFirstClick, setIsFirstClick] = useState(true);
	const { mutate: deleteTask } = useOptimisticDeleteTask();

	const holdTimerRef = useRef<NodeJS.Timeout | null>(null);
	const progressTimerRef = useRef<NodeJS.Timeout | null>(null);
	const isPressActiveRef = useRef(false);

	const scale = useMotionValue(1);
	const animatedScale = useSpring(scale, {
		stiffness: 300,
		damping: 20,
		restDelta: 0.001,
	});

	const cleanupTimers = () => {
		if (holdTimerRef.current) {
			clearTimeout(holdTimerRef.current);
			holdTimerRef.current = null;
		}
		if (progressTimerRef.current) {
			clearTimeout(progressTimerRef.current);
			progressTimerRef.current = null;
		}
	};

	const handleClick = () => {
		// If a press was active, ignore the click
		if (isPressActiveRef.current) {
			isPressActiveRef.current = false;
			return;
		}

		// Handle double-click confirmation
		if (isFirstClick) {
			setIsFirstClick(false);
			return;
		}

		deleteTask({ id: taskId });
	};

	const handlePressStart = (e: React.PointerEvent) => {
		e.preventDefault();
		isPressActiveRef.current = true;
		cleanupTimers();

		const startTime = Date.now();

		const animateScale = () => {
			const elapsed = Date.now() - startTime;
			const progress = Math.min(elapsed / HOLD_DURATION, 1);
			const scaleValue = 1 + progress * (SCALE_TARGET - 1);
			scale.set(scaleValue);

			if (progress < 1) {
				progressTimerRef.current = setTimeout(animateScale, 16);
			}
		};

		animateScale();

		holdTimerRef.current = setTimeout(() => {
			deleteTask({ id: taskId });
			isPressActiveRef.current = false;
		}, HOLD_DURATION);
	};

	const handlePressEnd = () => {
		cleanupTimers();
		scale.set(1);

		// Keep isPressActiveRef true briefly to prevent click from firing
		setTimeout(() => {
			isPressActiveRef.current = false;
		}, 50);
	};

	return (
		<Tooltip
			content={isFirstClick ? "Delete task" : "Confirm delete task"}
			trigger={
				<motion.button
					type="button"
					onClick={handleClick}
					onPointerDown={handlePressStart}
					onPointerUp={handlePressEnd}
					onPointerLeave={handlePressEnd}
					onPointerCancel={handlePressEnd}
					style={{ scale: animatedScale }}
					className="touch-none select-none"
				>
					<TaskActionButton
						className={cn("hover:text-red-400 hover:bg-red-500/20")}
					>
						<Trash size={14} strokeWidth={3} />
					</TaskActionButton>
				</motion.button>
			}
		/>
	);
};
