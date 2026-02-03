import { useForm, useStore } from "@tanstack/react-form";
import { useQueryClient } from "@tanstack/react-query";
import { format } from "date-fns";
import { useState } from "react";
import {
	Button,
	Dialog,
	DialogTrigger,
	Form,
	type Key,
} from "react-aria-components";
import { z } from "zod";
import {
	getGetGoalByIdQueryKey,
	getGetGoalsQueryKey,
	useCreateGoal,
	useUpdateGoal,
} from "~/aether-sdk";
import type { Goal } from "~/aether-sdk/models";
import { Checkbox } from "~/components/shared/checkbox";
import { DateTimePicker } from "~/components/shared/datepicker";
import { Label } from "~/components/shared/field";
import { Modal, modalContentStyles } from "~/components/shared/modal";
import { Select, SelectItem } from "~/components/shared/select";
import { Spinner } from "~/components/shared/spinner";
import { TextAreaField, TextField } from "~/components/shared/text-field";
import { cn } from "~/utils/cn";
import { convertCalendarDateToIsoString, getDateValue } from "~/utils/date";
import { RecurrenceType } from "../../tasks.domain";

const createGoalSchema = z
	.object({
		name: z.string().min(1, { message: "Name is required" }),
		description: z.string(),
		recurrenceType: z.preprocess(
			(val) => (val === "" ? undefined : val),
			z
				.enum([
					RecurrenceType.BI_WEEKLY,
					RecurrenceType.WEEKLY,
					RecurrenceType.MONTHLY,
					RecurrenceType.YEARLY,
					RecurrenceType.CUSTOM,
				])
				.optional(),
		),
		recurrenceInterval: z.number().optional(),
		recurrenceAnchor: z.preprocess(
			(val) => (val === "" ? undefined : val),
			z.string().datetime().optional(),
		),
		isNonRecurring: z.boolean(),
	})
	.superRefine((data, ctx) => {
		// If non-recurring, all recurring fields are optional - skip validation
		if (data.isNonRecurring) {
			return;
		}
		// If recurring, all recurring fields are required
		if (!data.recurrenceType) {
			ctx.addIssue({
				code: z.ZodIssueCode.custom,
				message: "Recurrence type is required when goal is recurring",
				path: ["recurrenceType"],
			});
		}
		if (
			data.recurrenceInterval === undefined ||
			data.recurrenceInterval === null
		) {
			ctx.addIssue({
				code: z.ZodIssueCode.custom,
				message: "Recurrence interval is required when goal is recurring",
				path: ["recurrenceInterval"],
			});
		}
		if (!data.recurrenceAnchor) {
			ctx.addIssue({
				code: z.ZodIssueCode.custom,
				message: "Recurrence anchor is required when goal is recurring",
				path: ["recurrenceAnchor"],
			});
		}
	});

interface CreateGoalFormProps {
	goal?: Goal;
	trigger: React.ReactNode;
}

export const GoalFormDialog = ({ goal, trigger }: CreateGoalFormProps) => {
	const queryClient = useQueryClient();
	const [isOpen, setIsOpen] = useState(false);
	const {
		mutate: createGoal,
		isPending: isCreatingGoal,
		reset,
	} = useCreateGoal({});
	const { mutate: updateGoal, isPending: isUpdatingGoal } = useUpdateGoal({});

	const goalQueryKey = getGetGoalByIdQueryKey(goal?.id ?? "");

	const isEditMode = !!goal;

	const form = useForm({
		defaultValues: {
			name: goal?.name ?? "",
			description: goal?.description ?? "",
			recurrenceType: goal?.recurrenceType ?? undefined,
			recurrenceInterval: goal?.recurrenceInterval ?? undefined,
			recurrenceAnchor: goal?.recurrenceAnchor ?? undefined,
			isNonRecurring: goal?.isNonRecurring ?? false,
		},
		validators: {
			// biome-ignore lint/suspicious/noExplicitAny: Type mismatch between Zod schema and TanStack Form types
			onChange: createGoalSchema as any,
			// biome-ignore lint/suspicious/noExplicitAny: Type mismatch between Zod schema and TanStack Form types
			onMount: createGoalSchema as any,
		},
		onSubmit: async ({ value }) => {
			const data = {
				name: value.name,
				description: value.description,
				recurrenceType: value.recurrenceType,
				recurrenceInterval: value.recurrenceInterval,
				recurrenceAnchor: value.recurrenceAnchor,
				isNonRecurring: value.isNonRecurring,
			};

			const goalsQueryKey = getGetGoalsQueryKey();

			if (isEditMode) {
				updateGoal(
					{
						id: goal?.id ?? "",
						data,
					},
					{
						onSuccess: () => {
							queryClient.invalidateQueries({ queryKey: goalQueryKey });
							queryClient.invalidateQueries({ queryKey: goalsQueryKey });
							setIsOpen(false);
						},
					},
				);
				return;
			}

			createGoal(
				{
					data,
				},
				{
					onSuccess: () => {
						queryClient.invalidateQueries({ queryKey: goalsQueryKey });
						setIsOpen(false);
					},
				},
			);
		},
	});

	const handleDialogClose = () => {
		form.reset();
		reset();
	};

	const handleRecurrenceTypeChange = (value: Key | null) => {
		form.setFieldValue("recurrenceType", value?.toString() ?? "");

		switch (value?.toString()) {
			case RecurrenceType.DAILY:
				form.setFieldValue("recurrenceInterval", 1);
				break;
			case RecurrenceType.WEEKLY:
				form.setFieldValue("recurrenceInterval", 7);
				break;
			case RecurrenceType.BI_WEEKLY:
				form.setFieldValue("recurrenceInterval", 14);
				break;
			case RecurrenceType.MONTHLY:
				form.setFieldValue("recurrenceInterval", 30);
				break;
			case RecurrenceType.YEARLY:
				form.setFieldValue("recurrenceInterval", 365);
				break;

			default:
				form.setFieldValue("recurrenceInterval", 1);
				break;
		}
	};

	const isCustomRecurrenceType = useStore(
		form.store,
		(state) => state.values.recurrenceType === RecurrenceType.CUSTOM,
	);

	const isNonRecurring = useStore(
		form.store,
		(state) => state.values.isNonRecurring,
	);

	const hasEditFormChanged = useStore(
		form.store,
		(state) =>
			state.values.name !== goal?.name ||
			state.values.description !== goal?.description,
	);

	return (
		<DialogTrigger isOpen={isOpen} onOpenChange={setIsOpen}>
			<Button>{trigger}</Button>
			<Modal isDismissable isOpen={isOpen} onOpenChange={setIsOpen}>
				<Dialog className={modalContentStyles}>
					<Form
						className="space-y-4"
						onSubmit={(e) => {
							e.preventDefault();
							form.handleSubmit();
						}}
					>
						<form.Field name="name">
							{(field) => (
								<TextField
									autoFocus
									name={field.name}
									label="Name"
									placeholder="What do you wanna track?"
									value={field.state.value}
									onChange={field.handleChange}
									errorMessage={field.state.meta.errors[0]?.message}
								/>
							)}
						</form.Field>
						<form.Field name="description">
							{(field) => (
								<TextAreaField
									name={field.name}
									label="Description"
									placeholder="Enter a description (optional)"
									value={field.state.value}
									onChange={field.handleChange}
									errorMessage={field.state.meta.errors[0]?.message}
								/>
							)}
						</form.Field>

						<form.Field name="isNonRecurring">
							{(field) => (
								<div className="px-2">
									<Checkbox
										isDisabled={isEditMode}
										name={field.name}
										isSelected={field.state.value}
										onChange={(checked) => {
											field.handleChange(checked);
											// Clear recurring fields when non-recurring is enabled
											if (checked) {
												form.setFieldValue("recurrenceType", undefined);
												form.setFieldValue("recurrenceInterval", undefined);
												form.setFieldValue("recurrenceAnchor", undefined);
											}
										}}
									>
										<p className="text-sm">Non-recurring</p>
									</Checkbox>
								</div>
							)}
						</form.Field>

						<form.Field name="recurrenceType">
							{(field) => (
								<div className="">
									<Select
										label="Recurrence type"
										placeholder="Select recurrence type"
										value={field.state.value ?? ""}
										onChange={(value: Key | null) => {
											if (value === null || value === "") {
												form.setFieldValue("recurrenceType", undefined);
											} else {
												handleRecurrenceTypeChange(value);
											}
										}}
										isDisabled={isEditMode || isNonRecurring}
									>
										<SelectItem id="weekly">Weekly</SelectItem>
										<SelectItem id="bi-weekly">Bi-weekly</SelectItem>
										<SelectItem id="monthly">Monthly</SelectItem>
										<SelectItem id="yearly">Yearly</SelectItem>
										<SelectItem id="custom">Custom</SelectItem>
									</Select>
								</div>
							)}
						</form.Field>

						<div className="flex gap-2 items-start">
							<form.Field name="recurrenceAnchor">
								{(field) => {
									return (
										<DateTimePicker
											isDisabled={isEditMode || isNonRecurring}
											className="flex-1  w-1/2"
											value={getDateValue(field.state.value ?? undefined)}
											onChange={(value) => {
												const dateString =
													convertCalendarDateToIsoString(value);
												field.handleChange(dateString);
											}}
											trigger={
												<div className="flex-1 flex flex-col gap-1">
													<Label>Start date</Label>
													<div
														className={cn([
															"px-3 py-0 min-h-9 flex-1 min-w-0 items-center flex ",
															"border-0 outline-0",
															"bg-neutral-100 text-sm",
															"placeholder:text-neutral-500 rounded-xl",
															{
																"opacity-50": isEditMode || isNonRecurring,
															},
														])}
													>
														<p>
															{field.state.value
																? format(field.state.value, "dd/MM/yyyy")
																: "Select date"}
														</p>
													</div>
												</div>
											}
										/>
									);
								}}
							</form.Field>

							<form.Field name="recurrenceInterval">
								{(field) => (
									<TextField
										className="w-1/2"
										type="number"
										name={field.name}
										label="Interval"
										placeholder="e.g. 1"
										isDisabled={!isCustomRecurrenceType || isNonRecurring}
										value={field.state.value?.toString() ?? ""}
										onChange={(value) => field.handleChange(Number(value))}
										errorMessage={field.state.meta.errors[0]?.message}
									/>
								)}
							</form.Field>
						</div>

						<div className="flex gap-2 self-end mt-4 w-full justify-end">
							<Button
								slot="close"
								type="button"
								className="bg-neutral-200 p-1 rounded-lg px-2 text-sm"
								onPress={handleDialogClose}
								isDisabled={
									form.state.isSubmitting || isCreatingGoal || isUpdatingGoal
								}
							>
								Cancel
							</Button>
							<form.Subscribe
								selector={(state) => [state.isValid, state.isSubmitting]}
								// biome-ignore lint/correctness/noChildrenProp: TanStack Form Subscribe requires children prop
								children={([isValid, isSubmitting]) => {
									const isDisabled = isEditMode
										? !hasEditFormChanged ||
											isSubmitting ||
											isUpdatingGoal ||
											!isValid
										: !isValid || isSubmitting || isCreatingGoal;

									return (
										<Button
											type="submit"
											className="bg-linear-to-b from-green-800 to-green-900 text-neutral-200 p-1 rounded-lg px-2 text-[13px] flex items-center gap-1 disabled:opacity-50 disabled:cursor-not-allowed"
											isDisabled={isDisabled}
										>
											{isCreatingGoal ||
												isUpdatingGoal ||
												(form.state.isSubmitting && (
													<Spinner className="size-3" strokeWidth={3} />
												))}
											{isEditMode ? "Save" : "Create Goal"}
										</Button>
									);
								}}
							/>
						</div>
					</Form>
				</Dialog>
			</Modal>
		</DialogTrigger>
	);
};
