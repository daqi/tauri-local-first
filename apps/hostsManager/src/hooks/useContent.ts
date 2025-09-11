import { useEffect, useRef, useState } from 'react';
import commands from '@/commands';
import writeHostsToSystem from '@/utils/writeHostsToSystem';
import useEvent from './useEvent';
import { EVENTS } from '@/events';
import { SYSTEM_HOSTS_ID } from '@/constants';

export default function useContent(id?: string) {
  const [[contentId, content], setContent] = useState<
    [string | null, string | null]
  >([null, null]);
  const contentMapRef = useRef<Map<string, string>>(new Map());
  const contentIdRef = useRef(contentId);
  contentIdRef.current = contentId;

  useEvent<any>(EVENTS.SYSTEM_HOSTS_UPDATED, (event) => {
    contentMapRef.current.set(SYSTEM_HOSTS_ID, event.payload);
    if (contentIdRef.current === SYSTEM_HOSTS_ID)
      setContent([SYSTEM_HOSTS_ID, event.payload]);
  });

  const getHostsContent = async (id2?: string) => {
    if (!id2) return;
    if (contentMapRef.current.has(id2)) {
      const next = contentMapRef.current.get(id2)!;
      setContent([id2, next]);
      return;
    }
    if (id2 === SYSTEM_HOSTS_ID) {
      const next: string = await commands.getSystemHosts();
      contentMapRef.current.set(id2, next);
      if (id2 === id) setContent([id, next]);
    } else {
      const next: string = await commands.getHostsContent(id2);
      contentMapRef.current.set(id2, next);
      if (id2 === id) setContent([id, next]);
    }
  };

  useEffect(() => {
    getHostsContent(id);
  }, [id]);

  return {
    content,
    contentId,
    updateContent: async (id: string, content: string) => {
      await commands.setHostsContent(id, content);
      contentMapRef.current.set(id, content);
      await writeHostsToSystem();
    },
  };
}

export type UseContentReturn = ReturnType<typeof useContent>;
