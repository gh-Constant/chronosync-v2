import { ref, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

type AppUsage = {
  display: string;
  title: string;
  process_name: string;
  duration: number;
};

type ActiveWindowInfo = {
  title: string;
  process_name: string;
  display: string;
};

export const appUsage = ref<AppUsage[]>([]);
export const appIcons = ref<Record<string, string>>({});

let lastActiveWindow: ActiveWindowInfo | null = null;
let lastWindowChangeTime = Date.now();

const intervalId = setInterval(async () => {
  try {
    const activeWindow = await invoke<ActiveWindowInfo>('get_active_window');

    if (activeWindow && activeWindow.display) {
      // If the window has changed
      if (!lastActiveWindow || activeWindow.display !== lastActiveWindow.display) {
        const now = Date.now();
        const duration = (now - lastWindowChangeTime) / 1000;

        // Update duration for the previous window
        if (lastActiveWindow) {
          const existingApp = appUsage.value.find(
            (app) => app.display === lastActiveWindow!.display
          );
          if (existingApp) {
            existingApp.duration += duration;
          } else {
            appUsage.value.push({
              display: lastActiveWindow.display,
              title: lastActiveWindow.title,
              process_name: lastActiveWindow.process_name,
              duration,
            });
          }
        }

        // Reset for the new window
        lastActiveWindow = activeWindow;
        lastWindowChangeTime = now;

        // Fetch icon for the new window if we haven't already
        if (!appIcons.value[activeWindow.process_name]) {
          console.log(`Fetching icon for: ${activeWindow.process_name}`);
          invoke<string>('get_app_icon', { processName: activeWindow.process_name })
            .then((icon) => {
              console.log(`Icon fetched for: ${activeWindow.process_name}`);
              appIcons.value[activeWindow.process_name] = icon;
            })
            .catch((err) => {
              console.error(`Failed to fetch icon for ${activeWindow.process_name}:`, err);
            });
        }
      }
    }
  } catch (error) {
    console.error('Error getting active window:', error);
  }
}, 1000);

onUnmounted(() => {
  clearInterval(intervalId);
});
