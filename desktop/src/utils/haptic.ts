import {
  isSupported,
  perform,
  HapticFeedbackPattern,
  PerformanceTime,
} from "tauri-plugin-macos-haptics-api";

export async function triggerHapticFeedback(pattern: HapticFeedbackPattern = HapticFeedbackPattern.Generic, time: PerformanceTime = PerformanceTime.Now) {
  if (await isSupported()) {
    await perform(pattern, time);
  }
}