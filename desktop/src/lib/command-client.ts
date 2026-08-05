/**
 * Platform-neutral contract for invoking Aether's Rust commands.
 *
 * Desktop and mobile shells can provide their own implementation while sharing
 * the frontend's command names, argument shapes, and response handling.
 */
export type CommandArgs = Record<string, unknown>;

export type CommandInvoker = <Result>(
	command: string,
	args?: CommandArgs,
) => Promise<Result>;

export interface CommandClient {
	invoke<Result>(command: string, args?: CommandArgs): Promise<Result>;
}

export function createCommandClient(invoke: CommandInvoker): CommandClient {
	return { invoke };
}
