import { createApp } from "vue";
import App from "./App.vue";
import { startTracking } from './services/window-tracker';

createApp(App).mount("#app");

startTracking();
