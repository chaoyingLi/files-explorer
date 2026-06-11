import { ref } from "vue";

export function useToast() {
  const message = ref("");
  const isError = ref(false);
  let timer: ReturnType<typeof setTimeout> | null = null;

  function show(msg: string, error = false) {
    message.value = msg;
    isError.value = error;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      message.value = "";
    }, error ? 4000 : 2000);
  }

  return { message, isError, show };
}
