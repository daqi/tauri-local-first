import { invoke } from '@tauri-apps/api/core';
import tryParseJSON from '@/utils/tryParseJson';

type JsonValue = any;

async function invokeCmd<T = any>(
  cmd: string,
  args?: Record<string, any>
): Promise<T> {
  const res = await invoke(cmd, args ?? {});
  console.log('invokeCmd', { cmd, args, res });
  return tryParseJSON(res) as T;
}

// Exported wrappers
export async function ping(): Promise<string> {
  return invokeCmd('ping');
}

export async function getList(): Promise<JsonValue[]> {
  return invokeCmd('get_list');
}
export async function setList(v: JsonValue[]): Promise<boolean> {
  return invokeCmd('set_list', { v });
}

export async function getContentOfList(): Promise<string> {
  return invokeCmd('get_content_of_list');
}

export async function getSystemHosts(): Promise<string> {
  return invokeCmd('get_system_hosts');
}

export async function getHostsContent(id: string): Promise<string> {
  return invokeCmd('get_hosts_content', { id });
}

export async function setHostsContent(
  id: string,
  content: string
): Promise<boolean> {
  return invokeCmd('set_hosts_content', { id, content });
}

export async function setSystemHosts(
  content: string,
  opts?: string | null
): Promise<JsonValue> {
  return invokeCmd('set_system_hosts', { content, opts });
}

export async function closeMainWindow(): Promise<boolean> {
  return invokeCmd('close_main_window');
}

export async function quitApp(): Promise<boolean> {
  return invokeCmd('quit');
}

// default export with snake_case aliases for convenience
const commands = {
  ping,
  getList,
  setList,
  getContentOfList,
  getSystemHosts,
  setSystemHosts,
  getHostsContent,
  setHostsContent,
  closeMainWindow,
  quitApp,
};

export default commands;
