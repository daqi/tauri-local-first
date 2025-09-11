import commands from '@/commands';
import message from '@/components/message';
import promptSystemPassword from '@/components/promptSystemPassword';
import debounce from 'lodash.debounce';
import { emit } from '@tauri-apps/api/event';
import { EVENTS } from '@/events';

const CONTENT_START = '# --- SWEETHOSTS_CONTENT_START ---\n';
const CONTENT_START1 = '# --- SWITCHHOSTS_CONTENT_START ---\n';
let pswd_cache = '';

const getOriginContent = async () => {
  const old_content = await commands.getSystemHosts();
  let index = old_content.indexOf(CONTENT_START);
  let index1 = old_content.indexOf(CONTENT_START1);
  if (index === -1 && index1 === -1) {
    return old_content;
  }
  if (index === -1) {
    index = index1;
  } else if (index1 !== -1) {
    index = Math.min(index, index1);
  }
  const origin = old_content.slice(0, index).trimEnd();
  return origin;
};

const writeHostsToSystem = async () => {
  const originContent = await getOriginContent();
  const content = await commands.getContentOfList();
  const newContent =
    originContent + (content ? `\n\n\n\n${CONTENT_START}\n\n${content}` : '\n');
  const res = await commands.setSystemHosts(newContent, pswd_cache);
  if (res.success) {
    emit(EVENTS.SYSTEM_HOSTS_UPDATED, res.new_content);
  } else {
    console.log(res);
    message.error('更新失败');
    const pswd = await promptSystemPassword();
    pswd_cache = pswd || '';
    const res2 = await commands.setSystemHosts(newContent, pswd_cache);
    if (res2.success) {
      emit(EVENTS.SYSTEM_HOSTS_UPDATED, res2.new_content);
    } else {
      message.error('更新失败');
    }
  }
};

const writeHostsToSystemDebounced = debounce(writeHostsToSystem, 600);

export default writeHostsToSystemDebounced;
