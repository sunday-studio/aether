import { createOpenAI } from '@ai-sdk/openai';
import { generateObject, NoObjectGeneratedError } from 'ai';
import { co, z } from 'jazz-tools';

// --- OpenAI client ---
const openaiClient = createOpenAI({
  apiKey:
    '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',
  // process.env.EXPO_PUBLIC_OPENAI_API_KEY,
});

// AI Schemas
export const GymActivityForAI = z.object({
  type: z.literal('gym'),
  exercise: z.string().optional(),
  sets: z.number().optional(),
  reps: z.number().optional(),
  weight: z.number().optional(),
  pr: z.boolean().optional(),
  notes: z.string().optional(),
});

export const OtherActivityForAI = z.object({
  type: z.literal('other'),
  description: z.string().optional(),
  notes: z.string().optional(),
});

const ActivityForAI = z.object({
  type: z.enum(['gym', 'other']),
  gym: GymActivityForAI.partial().optional(),
  other: OtherActivityForAI.partial().optional(),
});

// Jazz Schemas
export const GymActivity = co.map({
  ...GymActivityForAI.shape,
  createdAt: z.date(),
  updatedAt: z.date(),
});

export const OtherActivity = co.map({
  ...OtherActivityForAI.shape,
  createdAt: z.date(),
  updatedAt: z.date(),
});

export const Activity = co.discriminatedUnion('type', [GymActivity, OtherActivity]);

export const ActivityLog = co.map({
  activities: co.list(Activity),
});

export async function parseActivityFromText(text: string) {
  console.log('text -> ', text);
  try {
    const { object: aiResult } = await generateObject({
      model: openaiClient('gpt-4o-mini'),
      schema: GymActivityForAI,
      prompt: `
        You are the activity parser for Aether.
         Read user text and output a JSON object for the correct type with only the relevant fields.
        If the text doesn't match a known type, use "other".
        Text: "${text}"
      `,
    });

    console.log('aiResult -> ', JSON.stringify(aiResult, null, 2));
  } catch (error) {
    console.error('AI parsing failed, falling back to OtherActivity:', error);

    if (error instanceof NoObjectGeneratedError) {
      console.log('NoObjectGeneratedError');
      console.log('Cause:', error.cause);
      console.log('Text:', error.text);
      console.log('Response:', error.response);
      console.log('Usage:', error.usage);
      console.log('Finish Reason:', error.finishReason);
    }
    // const now = new Date();
    // return JazzActivityMap['other'].parse({
    //   type: 'other',
    //   description: text,
    //   createdAt: now,
    //   updatedAt: now,
    // });
  }
}
