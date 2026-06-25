import { ref } from "vue";

interface ToastMsg {
  id: number;
  text: string;
  isError: boolean;
}

let nextId = 0;
const TOAST_DURATION = 2500;
const TOAST_ERROR_DURATION = 4500;

export function useToast() {
  const messages = ref<ToastMsg[]>([]);

  function show(text: string, isError = false) {
    const id = ++nextId;
    messages.value.push({ id, text, isError });
    const duration = isError ? TOAST_ERROR_DURATION : TOAST_DURATION;
    setTimeout(() => {
      messages.value = messages.value.filter((m) => m.id !== id);
    }, duration);
  }

  return { messages, show };
}
