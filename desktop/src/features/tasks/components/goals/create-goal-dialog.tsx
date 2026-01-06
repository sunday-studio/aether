/** biome-ignore-all lint/correctness/noChildrenProp: <explanation> */
import { useForm, useStore } from "@tanstack/react-form";
import { useQueryClient } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { useState } from "react";
import {
	Button,
	Dialog,
	DialogTrigger,
	Form,
	type Key,
} from "react-aria-components";
import { z } from "zod";
import { getGetGoalsQueryKey, useCreateGoal } from "~/aether-sdk";
import { Modal, modalContentStyles } from "~/components/shared/modal";
import { Select, SelectItem } from "~/components/shared/select";
import { Spinner } from "~/components/shared/spinner";
import { TextAreaField, TextField } from "~/components/shared/text-field";
import { Tooltip } from "~/components/shared/tooltip";
import { RecurrenceType } from "../../tasks.domain";
import { TaskActionButton } from "../task-item/task-shared-components";

const createGoalSchema = z.object({
	name: z.string().min(1, { message: "Name is required" }),
	description: z.string(),
	recurrenceType: z.enum([
		RecurrenceType.BI_WEEKLY,
		RecurrenceType.WEEKLY,
		RecurrenceType.MONTHLY,
		RecurrenceType.YEARLY,
		RecurrenceType.CUSTOM,
	]),
	recurrenceInterval: z.number(),
	recurrenceAnchor: z.date(),
	// recurrenceMeta: z.string().optional(),
});

export const CreateGoalDialog = () => {
	const queryClient = useQueryClient();
	const [isOpen, setIsOpen] = useState(false);
	const { mutate, isPending, reset } = useCreateGoal({});

	const form = useForm({
		defaultValues: {
			name: "",
			description: "",
			recurrenceType: "",
			recurrenceInterval: 1,
			recurrenceAnchor: new Date(),
		},
		validators: {
			onChange: createGoalSchema,
			onMount: createGoalSchema,
		},
		onSubmit: async ({ value }) => {
			mutate(
				{
					data: {
						name: value.name,
						description: value.description,
						recurrenceType: value.recurrenceType,
						recurrenceInterval: value.recurrenceInterval,
						recurrenceAnchor: value.recurrenceAnchor,
						// recurrenceMeta: value.recurrenceMeta,
					},
				},

				{
					onError: (error) => {
						console.log("error ->", error);
					},
					onSuccess: () => {
						queryClient.invalidateQueries({ queryKey: getGetGoalsQueryKey() });
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

	return (
		<DialogTrigger isOpen={isOpen} onOpenChange={setIsOpen}>
			<Button>
				<Tooltip
					trigger={
						<TaskActionButton className="bg-transparent hover:bg-neutral-200 cursor-pointer">
							<Plus size={14} strokeWidth={3} />
						</TaskActionButton>
					}
					content="Create a new goal"
					shortcuts={["⌘", "G"]}
				/>
			</Button>
			<Modal isDismissable isOpen={isOpen} onOpenChange={setIsOpen}>
				<Dialog className={modalContentStyles}>
					{/* <p>Let's create a new goal</p> */}
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
						<form.Field name="recurrenceType">
							{(field) => (
								<div className="">
									<Select
										label="Recurrence type"
										placeholder="Select recurrence type"
										value={field.state.value}
										onChange={(value: Key | null) => {
											handleRecurrenceTypeChange(value);
										}}
									>
										<SelectItem id="weekly">Weekly</SelectItem>
										<SelectItem id="bi-weekly">Bi-weekly</SelectItem>
										<SelectItem id="monthly">Monthly</SelectItem>
										<SelectItem id="yearly">Yearly</SelectItem>
										<SelectItem id="custom">Custom</SelectItem>
									</Select>
									{/* {field.state.meta.errors[0] && (
										<p className="text-xs text-red-600 mt-1">
											{field.state.meta.errors[0]}
										</p>
									)} */}
								</div>
							)}
						</form.Field>

						<div className="flex gap-2">
							<form.Field name="recurrenceAnchor">
								{(field) => (
									<TextField
										type="date"
										name={field.name}
										label="Start date"
										placeholder="Select date"
										className="flex-1"
										value={
											field.state.value instanceof Date
												? field.state.value.toISOString().slice(0, 10)
												: field.state.value
										}
										onChange={(value) => {
											field.handleChange(new Date(value));
										}}
										errorMessage={field.state.meta.errors[0]?.message}
									/>
								)}
							</form.Field>

							<form.Field name="recurrenceInterval">
								{(field) => (
									<TextField
										type="number"
										name={field.name}
										label="Interval"
										placeholder="e.g. 1"
										isDisabled={!isCustomRecurrenceType}
										value={field.state.value.toString()}
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
								isDisabled={form.state.isSubmitting || isPending}
							>
								Cancel
							</Button>
							<form.Subscribe
								selector={(state) => [state.isValid, state.isSubmitting]}
								children={([isValid, isSubmitting]) => {
									return (
										<Button
											type="submit"
											className="bg-linear-to-b from-green-800 to-green-900 text-neutral-200 p-1 rounded-lg px-2 text-[13px] flex items-center gap-1 disabled:opacity-50 disabled:cursor-not-allowed"
											isDisabled={!isValid || isSubmitting || isPending}
										>
											{isPending ||
												(form.state.isSubmitting && (
													<Spinner className="size-3" strokeWidth={3} />
												))}
											Create Goal
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
