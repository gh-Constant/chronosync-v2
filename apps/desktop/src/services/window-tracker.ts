import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export const appUsage = ref<{ window: string; duration: number }[]>([]);
export const appIcons = ref<Record<string, string>>({});

const activeWindow = ref('');
const startTime = ref(Date.now());

const fetchAppImage = async (title: string) => {
  if (!title) return;
  if (appIcons.value[title]) return; // already cached
  try {
    const dataUrl = (await invoke('get_app_image', { title })) as string;
    if (dataUrl) appIcons.value = { ...appIcons.value, [title]: dataUrl };
  } catch (e) {
    // ignore errors for now
    console.warn('failed to fetch app image', e);
  }
};

const trackWindow = async () => {
  const currentWindow: string = await invoke('get_active_window');

  if (currentWindow !== activeWindow.value) {
    const endTime = Date.now();
    const duration = (endTime - startTime.value) / 1000;

    if (activeWindow.value) {
      const existingApp = appUsage.value.find(
        (app) => app.window === activeWindow.value
      );
      if (existingApp) {
        existingApp.duration += duration;
        // ensure icon is fetched
        fetchAppImage(existingApp.window);
      } else {
        appUsage.value.push({
          window: activeWindow.value,
          duration,
        });
        // fetch icon for the newly added app
        fetchAppImage(activeWindow.value);
      }
    }

    // also prefetch icon for the new current window
    fetchAppImage(currentWindow);

    activeWindow.value = currentWindow;
    startTime.value = Date.now();
  }
};

export const startTracking = () => {
  setInterval(trackWindow, 1000);
};