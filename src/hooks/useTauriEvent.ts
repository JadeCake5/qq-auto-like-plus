import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

export function useTauriEvent<T>(
  eventName: string,
  handler: (payload: T) => void,
) {
  useEffect(() => {
    const unlisten = listen<T>(eventName, (event) => {
      handler(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [eventName, handler]);
}
