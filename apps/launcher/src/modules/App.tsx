import { Box, Button, Container, Flex, Heading, Separator, Text } from '@radix-ui/themes';
import { open } from '@tauri-apps/plugin-shell';

export default function App() {
  const openHosts = async () => {
    // 通过自定义 scheme 深链唤起子应用（后续在子应用注册 tlfsuite://open?app=hosts 解析）
    await open('tlfsuite://open?app=hosts');
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
