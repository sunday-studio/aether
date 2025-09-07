import { co, z } from 'jazz-tools';

export const Activity = co.map({
  name: z.string(),
  description: z.string(),
  type: z.string(),
  duration: z.number(),
});
