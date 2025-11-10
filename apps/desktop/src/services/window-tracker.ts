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

const updateAppUsage = (windowInfo: ActiveWindowInfo, duration: number) => {
  const existingApp = appUsage.value.find(
    (app) => app.display === windowInfo.display
  );

  if (existingApp) {
    existingApp.duration += duration;
  } else {
    appUsage.value.push({
      display: windowInfo.display,
      title: windowInfo.title,
      process_name: windowInfo.process_name,
      duration,
    });
  }
};

const intervalId = setInterval(async () => {
  try {
    const activeWindow = await invoke<ActiveWindowInfo>('get_active_window');

    if (activeWindow && activeWindow.display) {
      const now = Date.now();
      const duration = (now - lastWindowChangeTime) / 1000;
      lastWindowChangeTime = now;

      if (lastActiveWindow && activeWindow.display !== lastActiveWindow.display) {
        // Window has changed, update usage for the previous window
        updateAppUsage(lastActiveWindow, duration);
      } else if (lastActiveWindow && activeWindow.display === lastActiveWindow.display) {
        // Window is the same, update usage for the current window
        updateAppUsage(activeWindow, duration);
      }

      // Update the last active window
      lastActiveWindow = activeWindow;

      // Fetch icon if it's a new process
      if (activeWindow.process_name && !appIcons.value[activeWindow.process_name]) {
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
  } catch (error) {
    console.error('Error getting active window:', error);
  }
}, 1000);

onUnmounted(() => {
  clearInterval(intervalId);
});
