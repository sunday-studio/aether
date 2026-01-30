/**
 * Extended model types that include runtime relationships
 * not present in the auto-generated SDK types.
 * 
 * The API returns these relationships at runtime but they're not
 * defined in the OpenAPI spec as they're joined dynamically.
 */

import type { Entry, Tag, Task, SubTask, GoalInstance } from "~/aether-sdk/models";

/**
 * Entry with tags relationship (loaded at runtime)
 */
export interface EntryWithTags extends Entry {
  tags?: Tag[];
}

/**
 * Task with tags and subtasks relationships (loaded at runtime)
 */
export interface TaskWithRelations extends Task {
  tags?: Tag[];
  subtasks?: SubTask[];
}

/**
 * Goal instance with tasks relationship (loaded at runtime)
 */
export interface GoalInstanceWithTasks extends GoalInstance {
  tasks?: Task[];
}
