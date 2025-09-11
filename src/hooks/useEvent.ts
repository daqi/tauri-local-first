import { EVENTS } from '@/events';
import { useEffect } from 'react';
import {
  EventCallback,
  EventName,
  listen,
  UnlistenFn,
} from '@tauri-apps/api/event';

export default function useEvent<T>(
  event: EventName & EVENTS,
  fn: EventCallback<T>
) {
  useEffect(() => {
    const offList: UnlistenFn[] = [];
    const init = async () => {
      const offFn = await listen<T>(event, (...args) => {
        console.log('useEvent', args);
        return fn(...args);
      });
      offList.push(offFn);
    };
    init();
    return () => {
      offList.forEach((offFn) => offFn());
    };
  }, []);
}

export type UseEventReturn = ReturnType<typeof useEvent>;
