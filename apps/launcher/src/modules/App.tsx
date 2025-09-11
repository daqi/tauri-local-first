import { Box, Button, Container, Flex, Heading, Separator, Text } from '@radix-ui/themes';
import { open } from '@tauri-apps/plugin-shell';
import { useEffect } from 'react';
import { getCurrent, onOpenUrl } from '@tauri-apps/plugin-deep-link';

export default function App() {
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      // 监听 tlfsuite deep link
      await getCurrent();
  unlisten = await onOpenUrl(async (urls: string[]) => {
        try {
          const u = new URL(urls[0]);
          if (u.hostname === 'open') {
            const app = u.searchParams.get('app');
            const args = u.searchParams.get('args') || '';
            if (app === 'hostsManager') {
              await open(`hostsmanager://open?args=${encodeURIComponent(args)}`);
            }
          }
        } catch {}
      });
    })();
    return () => { if (unlisten) unlisten(); };
  }, []);
  const openHosts = async () => {
    // 通过子应用专属 scheme 唤起
    await open('hostsmanager://open');
  };

  return (
    <Container size="2">
      <Flex direction="column" gap="3" py="3">
        <Heading>Launcher</Heading>
        <Text color="gray">统一入口 · 命令面板 · 插件化</Text>
        <Separator size="4" my="2" />
        <Box>
          <Button onClick={openHosts}>打开 Hosts 管理器</Button>
        </Box>
      </Flex>
    </Container>
  );
}
