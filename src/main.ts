import { createApp } from "vue";
import { createPinia } from "pinia";
import i18n from "./i18n";
import App from "./App.vue";
import "./style.css";

// Detect platform and set as data attribute for platform-specific CSS
const isMac = /Mac|iPod|iPhone|iPad/.test(navigator.platform);
if (isMac) {
  document.documentElement.setAttribute("data-platform", "macos");
} else if (/Win/.test(navigator.platform)) {
  document.documentElement.setAttribute("data-platform", "windows");
} else {
  document.documentElement.setAttribute("data-platform", "linux");
}

const app = createApp(App);
app.use(createPinia());
app.use(i18n);
app.mount("#app");
